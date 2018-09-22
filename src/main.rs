#![feature(uniform_paths)]

extern crate serde;
extern crate serde_json;
extern crate symlink;

#[macro_use] extern crate serde_derive;

mod homebrew;
mod inventory;
mod shell;
mod symlinks;

use clap::{Arg, App};
use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::path::{Path, PathBuf};

pub struct Context {
    working_directory: PathBuf,
    environment: HashMap<String, String>,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            working_directory: env::current_dir().expect("Could not get current directory"),
            environment: env::vars().collect()
        }
    }
}


impl Context {
    fn current_dir(&self) -> &Path {
        &self.working_directory
    }
}

pub trait Command {
    fn execute(&self, context: &Context);

    fn rollback(&self, context: &Context);
}

pub trait Source {
    const NAME: &'static str;
    type Item: Command;

    fn is_installed() -> bool {
        true
    }

    fn install() {}

    fn perform(&self, command: Self::Item) -> bool;
}

impl<T: Command> Command for Vec<T> {
    fn execute(&self, context: &Context) {
        for command in self {
            command.execute(context);
        }
    }

    fn rollback(&self, context: &Context) {
        for command in self {
            command.rollback(context);
        }
    }
}

fn main() {
    let m = App::new("Welcome to Dotter")
        .version("0.1")
        .author("Felipe Sere <felipesere@gmail.com>")
        .about("Think of a minimal subset of anisble, without any dependencies")
        .arg(Arg::with_name("direction")
             .help("Wheather to execute or rollback")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("inventory")
             .help("What inventory file to use")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("group")
             .help("Only run a single group")
             .long("group")
             .required(false)
             .takes_value(true))
        .get_matches();


    let command = m.value_of("direction").expect("No direction was given");
    let inventory = m.value_of("inventory").expect("No inventory was given");
    let group = m.value_of("group").expect("No group was chosen");

    let inv = inventory::read_inventory(inventory)
        .expect(&format!("Could not read inventory from {}", inventory));
    let group = inv.group(group)
        .expect("Did not find that group");

    let context = Context::default();

    if command == "run" {
        group.execute(&context);
    } else if command == "rollback" {
        group.rollback(&context);
    } else {
        println!("Unrecognized command {}", command);
    }
}
