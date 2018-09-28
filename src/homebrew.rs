use crate::{Command, Context, Explanation, Direction, Result};
use std::ffi::OsStr;
use std::process::{self, ExitStatus};
use std::path::Path;

pub fn is_homebrew_installed() -> bool {
    Path::new("/usr/local/bin/brew").exists()
}

pub fn install_homebrew() -> bool {
    let body = reqwest::get("https://raw.githubusercontent.com/Homebrew/install/master/install")
        .expect("URL was weird")
        .text()
        .expect("could not get stuff");

    let status = std::process::Command::new("ruby").args(&["-e", &body]).status().expect("it bombed out?");

    status.success()
}

fn brew(source: BrewSoure) -> process::Command {
    let mut command = process::Command::new("brew");
    match source {
        BrewSoure::Cask => {
            command.arg("cask");
        }
        _ => (),
    }
    command
}

// TODO better error reporting here!
fn install<S: AsRef<OsStr>>(name: S, cask: BrewSoure) -> ExitStatus {
    brew(cask)
        .arg("install")
        .arg(name)
        .status()
        .expect("Homebrew: could not install package")
}

// TODO better error reporting here!
fn remove<S: AsRef<OsStr>>(name: S, cask: BrewSoure) -> ExitStatus {
    brew(cask)
        .arg("remove")
        .arg(name.as_ref())
        .status()
        .expect("Homebrew: could not remove package")
}

// TODO better error reporting here!
fn ls<S: AsRef<OsStr>>(name: S, cask: BrewSoure) -> BrewStatus {
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

impl Brew {
    fn name(&self) -> String {
        match self {
            Brew::Simple(name) => name.clone(),
            Brew::FromTap(TappedBrew {tap, name}) => format!("{}/{}", tap, name),
            Brew::FromCask(CaskBrew {cask}) => cask.clone(),
        }
    }

    fn status(&self) -> BrewStatus {
        match self {
            Brew::Simple(name) => ls(name, Regular),
            Brew::FromCask(CaskBrew { cask }) => ls(cask, Cask),
            Brew::FromTap(TappedBrew { tap: _, name }) => ls(name, Regular),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum BrewStatus {
    Installed,
    Missing,
}

enum BrewSoure {
    Regular,
    Cask,
}

use crate::homebrew::BrewSoure::{Cask, Regular};

impl Command for Brew {
    fn execute(&self, _context: &Context) -> Result<()> {
        if self.status() == BrewStatus::Installed {
            return Ok(());
        }

        match self {
            Brew::Simple(name) => {
                install(name, Regular);
            }
            Brew::FromCask(CaskBrew { cask }) => {
                install(cask, Cask);
            }
            Brew::FromTap(TappedBrew { tap, name }) => {
                let full_name = format!("{}/{}", tap, name);
                install(full_name, Regular);
            }
        }

        Ok(())
    }

    fn rollback(&self, _context: &Context) -> Result<()> {
        match self {
            Brew::Simple(name) => {
                remove(name, Regular);
            }
            Brew::FromCask(CaskBrew { cask }) => {
                remove(cask, Cask);
            }
            Brew::FromTap(TappedBrew { tap: _tap, name }) => {
                remove(name, Regular);
            }
        }
        Ok(())
    }

    fn explain(&self, context: &Context) -> Result<Vec<Explanation>> {
        let message = match (self.status(), &context.direction) {
            (BrewStatus::Installed, Direction::Execute)  => format!("{} is already installed, nothing to do", self.name()),
            (BrewStatus::Missing,   Direction::Execute)  => format!("Will install {}", self.name()),
            (BrewStatus::Installed, Direction::Rollback) => format!("WIll uninstall {}.", self.name()),
            (BrewStatus::Missing,   Direction::Rollback)  => format!("{} is not installed, nothing to do.", self.name())
        };

        Ok(vec![Explanation::new(message)])
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn it_will_install_things_from_homebrew() {
        let context = Context::default();

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
        let context = Context::default();

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
        let context = Context::default();

        let brew_cask = Brew::FromTap(TappedBrew {
            tap: "brewsci/bio".to_string(),
            name: "abacas".to_string(),
        });

        assert_missing(&brew_cask);
        brew_cask.execute(&context);

        assert_installed(&brew_cask);
        brew_cask.rollback(&context);

        assert_missing(&brew_cask);
    }

    #[test]
    fn explaining_homebrew_commands_shows_what_needs_installing() {}

    fn assert_installed(brew: &Brew) {
        assert_eq!( brew.status(), BrewStatus::Installed);
    }

    fn assert_missing(brew: &Brew) {
        assert_eq!( brew.status(), BrewStatus::Missing);
    }
}
