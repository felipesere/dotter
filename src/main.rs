#![feature(uniform_paths)]

extern crate serde;
extern crate serde_json;
extern crate symlink;

#[macro_use]
extern crate serde_derive;

mod homebrew;
mod inventory;
mod shell;
mod symlinks;

use std::path::{Path, PathBuf};
use std::default::Default;
use std::env;
use std::collections::HashMap;

pub struct Context {
    working_directory: PathBuf,
    environment: HashMap<String, String>,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            working_directory: env::current_dir().unwrap(),
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
    let context = Context::default();
    let inv = inventory::read_inventory("samples/inventory.json").unwrap();
    let group = inv.group("test").unwrap();
    group.execute(&context);
    println!("Done!");
    group.rollback(&context);
}
