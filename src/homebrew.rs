use crate::{Command, Context};
use std::process::{self, ExitStatus};

fn brew(source: Source) -> process::Command {
    let mut command = process::Command::new("brew");
    match source {
        Source::Cask => { command.arg("cask"); },
        _ => ()
    }
    command
}

fn install<S: AsRef<str>>(name: S, cask: Source) -> ExitStatus {
    brew(cask)
        .arg("install")
        .arg(name.as_ref())
        .status()
        .expect("Homebrew: could not install package")
}

fn remove<S: AsRef<str>>(name: S, cask: Source) -> ExitStatus {
    brew(cask)
        .arg("remove")
        .arg(name.as_ref())
        .status()
        .expect("Homebrew: could not remove package")
}

fn ls<S: AsRef<str>>(name: S, cask: Source) -> BrewStatus {
    let status = brew(cask)
        .arg("ls")
        .arg("--versions")
        .arg(name.as_ref())
        .status()
        .expect("Homebrew: Could not check if package present");

    if status.success() {
        BrewStatus::Installed
    } else {
        BrewStatus::Missing
    }
}

#[derive(Deserialize, Debug)]
pub struct TappedBrew {
    tap: String,
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct CaskBrew {
    cask: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Brew {
    Simple(String),
    FromTap(TappedBrew),
    FromCask(CaskBrew),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Fact<T: Eq + std::fmt::Debug> {
    value: T,
}

#[derive(PartialEq, Eq, Debug)]
pub enum BrewStatus {
    Installed,
    Missing,
}

enum Source {
    Regular,
    Cask
}

use crate::homebrew::Source::Cask;
use crate::homebrew::Source::Regular;


impl Brew {
    fn gather_facts(&self) -> Fact<BrewStatus> {
        match self {
            Brew::Simple(name) => Fact {
                value: ls(name, Regular),
            },
            Brew::FromCask(CaskBrew { cask }) => Fact {
                value: ls(cask, Cask),
            },
            Brew::FromTap(TappedBrew { tap: _, name}) => Fact {
                value: ls(name, Regular),
            },
        }
    }
}

impl Command for Brew {
    fn execute(&self, _context: &Context) {
        match self {
            Brew::Simple(name) => {
                install(name, Regular);
            }
            Brew::FromCask(CaskBrew { cask }) => {
                install(cask, Cask);
            }
            Brew::FromTap(TappedBrew{ tap, name}) => {
                let full_name = format!("{}/{}", tap, name);
                install(full_name, Regular);
            }
        }
    }

    fn rollback(&self, _context: &Context) {
        match self {
            Brew::Simple(name) => {
                remove(name, Regular);
            }
            Brew::FromCask(CaskBrew { cask }) => {
                remove(cask, Cask);
            }
            Brew::FromTap(TappedBrew{tap: _tap, name}) => {
                remove(name, Regular);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[ignore]
    fn it_will_install_things_from_homebrew() {
        let context = Context {
            working_directory: PathBuf::new(),
        };

        let brew = Brew::Simple("parallel".to_string());

        assert_missing(&brew);
        brew.execute(&context);

        assert_installed(&brew);
        brew.rollback(&context);

        assert_missing(&brew);
    }

    #[test]
    #[ignore]
    fn works_for_brew_casks() {
        let context = Context {
            working_directory: PathBuf::new(),
        };

        let brew_cask = Brew::FromCask(CaskBrew {
            cask: "couleurs".to_string(),
        });

        assert_missing(&brew_cask);
        brew_cask.execute(&context);

        assert_installed(&brew_cask);
        brew_cask.rollback(&context);

        assert_missing(&brew_cask);
    }

    #[test]
    #[ignore]
    fn works_for_brew_tap() {
        let context = Context {
            working_directory: PathBuf::new(),
        };

        let brew_cask = Brew::FromTap(TappedBrew{ tap: "brewsci/bio".to_string(), name: "abacas".to_string()});

        assert_missing(&brew_cask);
        brew_cask.execute(&context);

        assert_installed(&brew_cask);
        brew_cask.rollback(&context);

        assert_missing(&brew_cask);
    }

    #[test]
    fn explaining_homebrew_commands_shows_what_needs_installing() {}

    fn assert_installed(brew: &Brew) {
        assert_eq!(
            brew.gather_facts(),
            Fact {
                value: BrewStatus::Installed
            }
        );
    }

    fn assert_missing(brew: &Brew) {
        assert_eq!(
            brew.gather_facts(),
            Fact {
                value: BrewStatus::Missing
            }
        );
    }
}
