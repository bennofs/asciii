use super::spec::*;
use super::Project;
use bill::{Bill, ItemList, Tax};
use util::currency_to_string;

pub trait ExportTarget<T> {
    fn export(&self) -> T;
}

fn opt_str(opt: Option<&str>) -> Option<String> {
    opt.map(|e| e.to_owned())
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Client {
    title: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    full_name: Option<String>,
    address: Option<String>,
    email: Option<String>,
    addressing: Option<String>,
}

impl ExportTarget<Client> for Project {
    fn export(&self) -> Client {
        Client {
            full_name: self.client().full_name(),
            addressing: self.client().addressing(),
            email: opt_str(self.client().email()),
            last_name: opt_str(self.client().last_name()),
            first_name: opt_str(self.client().first_name()),
            title: opt_str(self.client().title()),
            address: opt_str(self.client().address()),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Event {
    name: Option<String>,
    date: Option<String>,
    manager: Option<String>,
}

use chrono::prelude::*;
fn dmy(date: Option<Date<Utc>>) -> Option<String> {
    date.map(|d| d.format("%d.%m.%Y").to_string())
}

impl ExportTarget<Event> for Project {
    fn export(&self) -> Event {
        Event {
            name: IsProject::name(self).map(ToOwned::to_owned),
            date: dmy(self.event_date()),
            manager: self.responsible().map(ToOwned::to_owned),
        }
    }
}


#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Hours {
    time: Option<f64>,
    salary: Option<String>,
}

impl ExportTarget<Hours> for Project {
    fn export(&self) -> Hours {
        Hours {
            time: self.hours().total(),
            salary: self.hours().salary().map(|s| s.postfix().to_string()),
        }
    }
}


#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Sum {
    gross_sum: String,
    has_tax: bool,
    tax_sum: String,
    tax_value: f64,
}

use super::product::Product;
fn sums_from_bill(bill: &Bill<Product>) -> Vec<Sum> {
    bill.iter()
        .map(|(tax, list)| Sum::from_itemlist(tax, list))
        .rev()
        .collect::<Vec<_>>()
}

impl Sum {
    pub fn from_itemlist(tax: &Tax, list: &ItemList<Product>) -> Sum {
        let gross_sum = list.gross_sum();
        let tax_sum = list.tax_sum();
        Sum {
            tax_value: (tax.into_inner() * 100.0),
            gross_sum: currency_to_string(&gross_sum),
            tax_sum: currency_to_string(&tax_sum),
            has_tax: (tax.into_inner() > 0f64),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Offer {
    // appendix: Option<i64>,
    date: Option<String>,
    number: Option<String>,
    sums: Vec<Sum>,
    net_total: String,
    gross_total: String,
}


impl ExportTarget<Offer> for Project {
    fn export(&self) -> Offer {
        let (offer, _) = self.bills().unwrap();
        Offer {
            // appendix: self.offer().appendix(),
            date: dmy(self.offer().date()),
            number: self.offer().number(),
            sums: sums_from_bill(&offer),
            net_total: currency_to_string(&offer.net_total()),
            gross_total: currency_to_string(&offer.gross_total()),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Invoice {
    date: Option<String>,
    number: Option<String>,
    number_long: Option<String>,
    official: Option<String>,
    sums: Vec<Sum>,
    net_total: String,
    gross_total: String,
}


impl ExportTarget<Invoice> for Project {
    fn export(&self) -> Invoice {
        let (_, invoice) = self.bills().unwrap();

        Invoice {
            date: dmy(self.invoice().date()),
            number: self.invoice().number_str(),
            number_long: self.invoice().number_long_str(),
            official: self.invoice().official(),
            sums: sums_from_bill(&invoice),
            net_total: currency_to_string(&invoice.net_total()),
            gross_total: currency_to_string(&invoice.gross_total()),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
struct ExportProduct {
    name: String,
    price: String,
    unit: String,
    amount: f64,
    cost: String,
    tax: f64,
}

fn bill_products(bill: &Bill<Product>) -> Vec<ExportProduct> {
    bill.as_items_with_tax()
        .into_iter()
        .map(|(tax, item)| {
        ExportProduct {
            name: item.product.name.to_string(),
            price: currency_to_string(&item.product.price),
            unit: item.product.unit.unwrap_or_else(|| "").to_string(),
            amount: item.amount,
            cost: currency_to_string(&item.gross()),
            tax: tax.value(),
        }
    })
        .collect()
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Bills {
    offer: Vec<ExportProduct>,
    invoice: Vec<ExportProduct>,
}


impl ExportTarget<Bills> for Project {
    fn export(&self) -> Bills {
        let (offer, invoice) = self.bills().unwrap();

        Bills {
            offer: bill_products(&offer),
            invoice: bill_products(&invoice),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize))]
pub struct Complete {
    client: Client,
    event: Event,
    hours: Hours,
    offer: Offer,
    invoice: Invoice,
    bills: Bills,
}


impl ExportTarget<Complete> for Project {
    fn export(&self) -> Complete {
        Complete {
            client: self.export(),
            event: self.export(),
            hours: self.export(),
            offer: self.export(),
            invoice: self.export(),
            bills: self.export(),
        }
    }
}
