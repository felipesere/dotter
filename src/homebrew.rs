use crate::{Command, Context};
use std::process;

#[derive(Deserialize, Debug)]
pub struct TappedBrew {
    tap: String,
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Brew {
    Simple(String),
    FromTap(TappedBrew),
    FromCask(String),
}

impl Command for Brew {
    fn execute(&self, _context: &Context) {
        match self {
            Brew::Simple(name) => {
                process::Command::new("brew")
                    .arg("install")
                    .arg(name)
                    .status()
                    .expect("failed to execute process");
            }
            _ => (),
        }
    }

    fn rollback(&self, _context: &Context) {
        match self {
            Brew::Simple(name) => {
                process::Command::new("brew")
                    .arg("remove")
                    .arg(name)
                    .status()
                    .expect("failed to execute process");
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn it_will_install_things_from_homebrew() {
        let context = Context {
            working_directory: PathBuf::new(),
        };

        let brew = Brew::Simple("parallel".to_string());

        brew.execute(&context);

        // could do something around the 'gather_facts' and a brew list
        brew.rollback(&context);
    }

    #[test]
    fn explaining_homebrew_commands_shows_what_needs_installing() {}

    #[test]
    fn rolling_back_homebrew_will_uninstall_declared_apps() {}
}
