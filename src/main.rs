#![feature(uniform_paths)]

extern crate serde;
extern crate serde_json;
extern crate symlink;

#[macro_use]
extern crate serde_derive;

mod homebrew;
mod inventory;

use std::path::{Path, PathBuf};

pub enum Facts {
    None,
}

pub struct Context {
    working_directory: PathBuf,
}

pub struct Explanation {}

impl Context {
    fn current_dir(&self) -> &Path {
        &self.working_directory
    }
}

pub trait Command {
    fn explain(&self, context: &Context) -> Explanation;

    fn gather_facts(&self, context: &Context) -> Facts;

    fn execute(&self, context: &Context);

    fn rollback(&self, context: &Context);
}

fn main() {
    println!("Hello, world!");
}
