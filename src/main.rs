#![allow(dead_code)]
#![allow(unused_imports)]
#![cfg(not(test))]
extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate tempdir;
extern crate term;
extern crate git2;
extern crate currency;
#[macro_use] extern crate prettytable;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
#[macro_use] extern crate clap;

mod util;
mod config;

mod project;
mod manager;
mod repo;

mod templater;
mod cli;

use clap::{App, SubCommand, Arg};
use manager::LuigiDir;
use cli::SortOptions;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}

// TODO: add logging
// TODO: make better use of io::ErrorKind
// TODO: remove: to_owned() and unwrap()s, stupid :D

fn main(){
    cli::app();
}

