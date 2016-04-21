//! Simple templating functionality through keyword replacement.
//!
//! Replaces `__KEYWORDS__` in Strings.
use std::io;
use std::io::Read;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

use regex::{Regex,Captures};
use std::ops::Deref;

/// Simple template style keyword replacement.
///
/// This allows replacing a known set of keywords looking like `__THIS__`.
/// Here it is implemented for `Deref<Target=str>`.
pub trait IsKeyword {
    /// Checks if the whole string is a keyword
    fn is_keyword(&self) -> bool;
    /// Captures keywords from string.
    fn get_keyword(&self) -> Option<String>;
    /// Well, it lists the keywords in a string, duh!
    fn list_keywords(&self) -> Vec<String>;

    /// This one is usefull.
    ///
    /// Takes a clorsure that replaces keywords.
    /// **Careful**, this replaces either way!
    /// If you get a keywords you don't want to replace,
    /// please place it back where you got it from.
    ///
    /// # Example
    /// ```ignore
    /// .map_keywords|keyword| match data.get(keyword){
    ///     Some(content) => String::from(*content),
    ///     None => format!("__{}__", keyword)
    /// }
    /// ```
    ///
    fn map_keywords<F>(&self, closure: F) -> String where F:Fn(&str) -> String;// -> Option<String>;
}

static REGEX: &'static str = r"__([0-9A-Z-]*)__*";

/// Allows very simplistic `__KEYWORD__` replacement.
impl<U:Deref<Target=str>> IsKeyword for U {

    /// Checks if the whole string is a keyword
    fn is_keyword(&self) -> bool{
        Regex::new(REGEX).expect("broken regex").is_match(self)
    }

    /// Captures keywords from string.
    fn get_keyword(&self) -> Option<String> {
        Regex::new(REGEX).expect("broken regex")
            .captures(&self)
            .and_then(|caps| caps.at(1).map(|c| c.to_owned()))
    }

    /// Well, it lists the keywords in a string, duh!
    fn list_keywords(&self) -> Vec<String>{
        Regex::new(REGEX).expect("broken regex")
            .captures_iter(&self)
            .map(|c|c.at(1).unwrap().to_owned())
            .collect()
    }

    /// This one is usefull.
    ///
    /// Takes a clorsure that replaces keywords.
    /// **Careful**, this replaces either way!
    /// If you get a keywords you don't want to replace,
    /// please place it back where you got it from.
    ///
    /// # Example
    /// ```ignore
    /// .map_keywords|keyword| match data.get(keyword){
    ///     Some(content) => String::from(*content),
    ///     None => format!("__{}__", keyword)
    /// }
    /// ```
    ///
    fn map_keywords<F>(&self, closure: F) -> String
        where F:Fn(&str) -> String{
        Regex::new(REGEX).expect("broken regex")
            .replace_all(&self, |caps:&Captures| {
                closure(caps.at(1).unwrap())
            })
    }
}


/// Simple templating module
#[derive(Debug)]
pub struct Templater{
    /// content of template file after reading
    pub original: String,

    /// content of filled template
    pub filled: String,
}

impl Templater{

    pub fn new(template:&str) -> Templater{
        Templater{
            original:template.to_owned(),
            filled: String::new(),
        }
    }

    pub fn from_file(path:&Path) -> Result<Templater, io::Error> {
        let template = try!(File::open(&path)
            .and_then(|mut file| {
                let mut content = String::new();
                file.read_to_string(&mut content).map(|_| content)
            }));

        Ok(Templater::new(&template))
    }

    pub fn finalize(&mut self) -> Templater {
        self.to_owned()
    }

    /// Creates a finished version of the output.
    ///
    /// If any keywords remain unfilled, `Err` contains a list of left overs.
    pub fn complete(&mut self) -> Result<Templater,Vec<String>> {
        let left_overs = self.filled.list_keywords();

        if left_overs.is_empty(){
            Ok(self.to_owned())
        } else {
            Err(left_overs)
        }
    }

    pub fn fill_in_data(&mut self, data: &HashMap<&str,String>) -> &mut Templater {
        self.fill_template(|keyword| match data.get(keyword){
            Some(content) => content.clone(),
            None => format!("__{}__", keyword)
        })
    }

    pub fn list_keywords(&self) -> Vec<String>{
        self.original.list_keywords()
    }

    pub fn fill_template<F>(&mut self, closure: F) -> &mut Templater
        where F:Fn(&str) -> String {
        self.filled = self.original.map_keywords(closure);
        self
    }
}

use std::borrow::ToOwned;
impl ToOwned for Templater{
    type Owned = Templater;
    fn to_owned(&self) -> Templater {
        Templater{
            //path :    self.path.to_owned(),
            original: self.original.to_owned(),
            filled:   self.filled.to_owned()
        }
    }
}

#[cfg(test)]
mod test{
    use super::Templater;
    const TEMPLATE:&'static str = r##"This tests __TEST__ for __ATTR__ __SUBJ__."##;

   #[test]
   fn complete(){
       let filled_in = Templater::new(TEMPLATE)
           .fill_in_data(&hashmap!{
               "TEST" => String::from("templates"),
               "ATTR" => String::from("complete"),
               "SUBJ" => String::from("replacements"),
           }).complete().unwrap();
       assert_eq!(filled_in.filled, "This tests templates for complete replacements.")
   }

   #[test]
   fn not_complete(){
       let filled_in = Templater::new(TEMPLATE)
           .fill_in_data(&hashmap!{
               "TEST" => String::from("templates"),
           }).complete();
       assert!(filled_in.is_err());
       assert_eq!(&filled_in.unwrap_err(), &["ATTR","SUBJ"])
   }

}
