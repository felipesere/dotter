#![feature(uniform_paths)]

extern crate serde;
extern crate serde_json;
extern crate symlink;

#[macro_use]
extern crate serde_derive;

mod homebrew;
mod inventory;
mod shell;

use std::path::{Path, PathBuf};

pub struct Context {
    working_directory: PathBuf,
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

fn main() {
    println!("Hello, world!");
}
