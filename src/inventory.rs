use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use symlink::{remove_symlink_file, symlink_file};

use crate::homebrew::Brew;
use crate::{Command, Context};

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
    brew: Vec<Brew>,

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

impl Command for Symlink {
    fn execute(&self, context: &Context) {
        let current_dir = context.current_dir();
        symlink_file(current_dir.join(&self.from), current_dir.join(&self.to))
            .expect("Could not create symlink");
    }

    fn rollback(&self, context: &Context) {
        let current_dir = context.current_dir();
        remove_symlink_file(current_dir.join(&self.to)).expect("Could not remove symlink");
    }
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
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use tempfile::TempDir;

    #[test]
    fn it_can_read_the_inventory() {
        let inventory: Inventory = read_inventory("samples/inventory.json").unwrap();

        println!("{:#?}", inventory);
        assert_eq!(2, inventory.count());
    }

    fn given_a_file_exists(name: &'static str) -> TempDir {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(name);
        let mut tmp_file = File::create(file_path).unwrap();
        writeln!(tmp_file, "The original text").unwrap();

        dir
    }

    #[test]
    fn it_creats_a_symmlink_when_executed() {
        let dir = given_a_file_exists("original.txt");

        let linker = Symlink {
            from: "original.txt".to_string(),
            to: "the_copy.txt".to_string(),
        };

        let context = Context {
            working_directory: dir.into_path(),
        };

        linker.execute(&context);

        let paths = std::fs::read_dir(&context.working_directory).unwrap();
        assert_eq!(paths.count(), 2);

        linker.rollback(&context);

        let after = std::fs::read_dir(&context.working_directory).unwrap();
        assert_eq!(after.count(), 1);
    }

    #[test]
    fn it_will_interpolate_home_directory() {}
}
