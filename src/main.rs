#![feature(uniform_paths)]

extern crate serde;
extern crate serde_json;
extern crate symlink;

#[macro_use] extern crate structopt;

use structopt::StructOpt;

#[macro_use] extern crate failure;

#[macro_use] extern crate serde_derive;

mod homebrew;
mod inventory;
mod shell;
mod symlinks;
mod group;

use std::collections::HashMap;
use std::default::Default;
use std::env;
use std::path::PathBuf;
use std::result;
use crate::homebrew::{is_homebrew_installed, install_homebrew};

pub type Result<T> = result::Result<T, failure::Error>;

#[derive(Fail, Debug)]
#[fail(display = "Error occured: {}", _0)]
struct MyError(&'static str);

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
            Err(failure::Error::from(MyError("Did not match either 'run' or 'rollback'")))
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

#[derive(StructOpt, Debug)]
#[structopt(name = "dotter",
            about = "Think of a minimal subset of anisble, without any dependencies",
            author = "Felipe Sere <felipesere@gmail.com>",
            version = "1.0.0")]
struct Opt {
    #[structopt(name="direction")]
    direction: Direction,

    #[structopt(name="inventory")]
    inventory: String,

    #[structopt(name="explain", long = "explain")]
    explain: bool,

    #[structopt(name="group", long = "group")]
    group: Option<String>
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

impl std::convert::From<Opt> for Context {
    fn from(options: Opt) -> Self {
        Context {
            direction: options.direction,
            explain: options.explain,
            ..Context::default()
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    if !is_homebrew_installed() {
        install_homebrew();
    }

    let mut inv = inventory::read_inventory(&opt.inventory)?;

    let target: Box<dyn Command> = if let Some(name) = &opt.group {
        let group = inv.group(name.as_ref()).expect("did not find group.");
        Box::new(group)
    } else {
        Box::new(inv)
    };

    let context = Context::from(opt);
    if context.explain {
        for explanation in target.explain(&context)? {
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
