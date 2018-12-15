extern crate serde;
extern crate serde_json;
extern crate symlink;
extern crate clap;


#[macro_use] extern crate failure;
#[macro_use] extern crate serde_derive;

mod homebrew;
mod inventory;
mod shell;
mod symlinks;
mod group;

use crate::homebrew::{is_homebrew_installed, install_homebrew};
use std::collections::HashMap;
use std::default::Default;
use std::{env, result};
use std::path::PathBuf;
use clap::{App, Arg, ArgMatches, ArgGroup};
use std::str::FromStr;

pub type Result<T> = result::Result<T, failure::Error>;

fn main() -> Result<()> {
    let matches = App::new("dotter")
        .author("Felipe Sere <felipesere@gmail.com>")
        .about("Think of a minimal subset of anisble, without any dependencies")
        .group(
            ArgGroup::with_name("execution")
            .args(&["direction", "inventory", "explain", "only"])
            .multiple(true)
            .requires_all(&["direction", "inventory"])
            .required(false))
        .arg(Arg::with_name("direction")
             .takes_value(true)
             .index(1)
             .possible_values(&["run", "rollback"]))
        .arg(Arg::with_name("inventory")
             .index(2)
             .takes_value(true))
        .arg(Arg::with_name("explain").short("e").long("explain").requires("execution"))
        .arg(
            Arg::with_name("only")
            .short("o")
            .long("only")
            .takes_value(true)
            .requires("execution"))
        .arg(
            Arg::with_name("version").short("v").long("version").conflicts_with("execution")
            )
        .get_matches();

    if matches.is_present("version") {
        println!(env!("VERSION"));
        return Ok(());
    }


    if !is_homebrew_installed() {
        install_homebrew();
    }

    let mut inv = inventory::read_inventory(&matches.value_of("inventory").unwrap())?;

    let target: Box<dyn Command> = if let Some(name) = &matches.value_of("only") {
        let group = inv.group(name.as_ref()).expect("did not find group.");
        Box::new(group)
    } else {
        Box::new(inv)
    };

    let context = Context::from(matches);
    if context.explain {
        for explanation in target.explain(&context)? {
            println!("{}", explanation.message);
        }
        Ok(())
    } else {
        target.dispatch(&context)
    }
}

pub struct Context {
    direction: Direction,
    environment: HashMap<String, String>,
    explain: bool,
    working_directory: PathBuf
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


impl <'a> std::convert::From<ArgMatches<'a>> for Context {
    fn from(options: ArgMatches<'a>) -> Self {
        let direction = options.value_of("direction").and_then(|dir| Direction::from_str(dir).ok()).unwrap_or(Direction::Execute);

        Context {
            direction: direction,
            explain: options.is_present("explain"),
            ..Context::default()
        }
    }
}


#[derive(Fail, Debug)]
#[fail(display = "Error occured: {}", _0)]
struct MyError(&'static str);

macro_rules! simple_error {
    ($expresssion:expr) => (
        result::Result::Err(failure::Error::from(MyError($expresssion)))
    )
}

#[derive(Debug)]
pub enum Direction {
    Execute,
    Rollback
}

impl std::str::FromStr for Direction {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "run" {
            Ok(Direction::Execute)
        } else if s == "rollback" {
            Ok(Direction::Rollback)
        } else {
            simple_error!("Did not match either 'run' or 'rollback'")
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
    fn dispatch(&self, context: &Context) -> Result<()> {
        match context.direction {
            Direction::Execute => self.execute(&context),
            Direction::Rollback => self.rollback(&context),
        }
    }

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
