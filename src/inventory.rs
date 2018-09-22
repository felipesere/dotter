use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

use crate::homebrew::{is_homebrew_installed, install_homebrew, Brew};
use crate::shell::ShellCommand;
use crate::symlinks::Symlink;
use crate::{Command, Context, Explanation};

#[derive(Deserialize, Debug)]
pub struct Inventory(HashMap<String, Group>);

impl Inventory {
    fn count(&self) -> usize {
        self.0.len()
    }

    pub fn group<S: Into<String>>(&self, group: S) -> Option<&Group> {
        self.0.get(&group.into())
    }
}

#[derive(Deserialize, Debug)]
pub struct Group {
    #[serde(default)]
    brew: Vec<Brew>,

    #[serde(default)]
    symlinks: Vec<Symlink>,

    #[serde(default)]
    shell: Vec<ShellCommand>,
}

impl Command for Group {
    fn execute(&self, context: &Context) {
        if !is_homebrew_installed() {
            install_homebrew();
        }
        self.brew.execute(&context);
    }

    fn rollback(&self, context: &Context) {
        self.brew.rollback(&context);
    }

    fn explain(&self, context: &Context) -> Vec<Explanation> {
        // do something clever to check if target/source exist
        self.brew.explain(&context)
    }
}

pub fn read_inventory<P: AsRef<Path>>(path: P) -> Result<Inventory, Box<Error>> {
    let file = File::open(path)?;

    let i = serde_json::from_reader(file)?;

    Ok(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_read_the_inventory() {
        let inventory: Inventory = read_inventory("samples/inventory.json").unwrap();

        println!("{:#?}", inventory);
        assert_eq!(2, inventory.count());
    }
}
