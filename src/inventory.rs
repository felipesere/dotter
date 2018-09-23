use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::group::Group;
use crate::{Command, Context, Explanation, Result};

#[derive(Deserialize, Debug)]
pub struct Inventory(HashMap<String, Group>);

impl Inventory {
    pub fn group<S: Into<String>>(&mut self, group: S) -> Option<Group> {
        self.0.remove(&group.into())
    }
}

impl Command for Inventory {
    fn execute(&self, context: &Context) -> Result<()> {
        for (_key, value) in self.0.iter() {
            value.execute(&context)?;
        }
        Ok(())
    }

    fn rollback(&self, context: &Context) -> Result<()> {
        for (_key, value) in self.0.iter() {
            value.rollback(&context)?;
        }
        Ok(())
    }

    fn explain(&self, context: &Context) -> Result<Vec<Explanation>> {
        let mut explanations = Vec::new();
        for (_key, value) in self.0.iter() {
            explanations.append(&mut value.explain(&context)?);
        }
        Ok(explanations)
    }
}

pub fn read_inventory<P: AsRef<Path>>(path: P) -> Result<Inventory> {
    let file = File::open(path)?;

    let i = serde_json::from_reader(file)?;

    Ok(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_read_the_inventory() {
        let mut inventory: Inventory = read_inventory("samples/inventory.json").unwrap();

        assert!(inventory.group("vim").is_some());
        assert!(inventory.group("homebrew").is_some());
    }
}
