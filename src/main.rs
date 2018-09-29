#![feature(uniform_paths)]

extern crate serde;
extern crate serde_json;
extern crate symlink;
extern crate failure;

#[macro_use] extern crate serde_derive;

mod homebrew;
mod inventory;
mod shell;
mod symlinks;
mod group;

use clap::{Arg, App};
use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::path::PathBuf;
use std::result;
use crate::homebrew::{is_homebrew_installed, install_homebrew};

pub type Result<T> = result::Result<T, failure::Error>;

pub struct Context {
    direction: Direction,
    environment: HashMap<String, String>,
    explain: bool,
    working_directory: PathBuf
}

pub enum Direction {
    Execute,
    Rollback
}

impl Default for Context {
    fn default() -> Context {
        Context {
            direction: Direction::Execute,
            environment: env::vars().collect(),
            explain: false,
            working_directory: env::current_dir().expect("Could not get current directory")
        }
    }
}

pub struct Explanation {
    message: String
}

impl Explanation {
    fn new<S: Into<String>>(message: S) -> Explanation {
        Explanation {
            message: message.into()
        }
    }
}

pub trait Command {
    fn execute(&self, context: &Context) -> Result<()>;

    fn rollback(&self, context: &Context) -> Result<()>;

    fn explain(&self, context: &Context) -> Result<Vec<Explanation>>;
}

impl<T: Command> Command for Vec<T> {
    fn execute(&self, context: &Context) -> Result<()> {
        for command in self {
            command.execute(context)?;
        }
        Ok(())
    }

    fn rollback(&self, context: &Context) -> Result<()> {
        for command in self {
            command.rollback(context)?;
        }
        Ok(())
    }

    fn explain(&self, context: &Context) -> Result<Vec<Explanation>> {
        let mut explanations = Vec::new();

        for command in self {
            explanations.append(&mut command.explain(context)?);
        }

        Ok(explanations)
    }
}

fn main() -> Result<()> {
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
        .arg(Arg::with_name("explain")
             .help("Explain what actions will be taken")
             .long("explain")
             .required(false))
        .arg(Arg::with_name("group")
             .help("Only run a single group")
             .long("group")
             .required(false)
             .takes_value(true))
        .get_matches();

    if !is_homebrew_installed() {
        install_homebrew();
    }


    let mut context = Context::default();
    let command = m.value_of("direction").expect("No direction was given");
    let inventory = m.value_of("inventory").expect("No inventory was given");
    let group_name = m.value_of("group");
    let explain = m.is_present("explain");

    let mut inv = inventory::read_inventory(inventory).expect(&format!("Could not read inventory from {}", inventory));

    if command == "rollback" {
        context.direction = Direction::Rollback;
    }
    context.explain = explain;

    let target: Box<dyn Command> = if group_name.is_some() {
        let group = inv.group(group_name.unwrap()).unwrap();
        Box::new(group)
    } else {
        Box::new(inv)
    };

    if explain {
        for explanation in target.explain(&context).unwrap() {
            println!("{}", explanation.message);
        }
        Ok(())
    } else {
        match context.direction {
            Direction::Execute => target.execute(&context),
            Direction::Rollback => target.rollback(&context),
        }
    }
}
