use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub enum Facts {}

pub struct Context;

pub trait Command {
    fn gather_facts(&self, context: &Context) -> Facts;

    fn execute(&self, context: &Context);

    fn rollback(&self, context: &Context);
}

#[derive(Deserialize, Debug)]
pub struct Inventory(HashMap<String, Group>);

impl Inventory {
    fn count(&self) -> usize {
        self.0.len()
    }
}

#[derive(Deserialize, Debug)]
struct Group {
    #[serde(default)]
    brew: Vec<String>,

    #[serde(default)]
    symlinks: Vec<Symlink>,

    #[serde(default)]
    shell: Vec<ShellCommand>,
}

#[derive(Deserialize, Debug)]
struct Symlink {
    from: String,
    to: String,
}

#[derive(Deserialize, Debug)]
struct ShellCommand {
    run: String,
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
