use crate::homebrew::{is_homebrew_installed, install_homebrew, Brew};
use crate::shell::ShellCommand;
use crate::symlinks::Symlink;
use crate::{Command, Context, Explanation};

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
        self.symlinks.execute(&context);
        self.shell.execute(&context);
    }

    fn rollback(&self, context: &Context) {
        self.brew.rollback(&context);
        self.symlinks.rollback(&context);
        self.shell.rollback(&context);
    }

    fn explain(&self, context: &Context) -> Vec<Explanation> {
        let mut explanations = Vec::new();
        explanations.append(&mut self.brew.explain(&context));
        explanations.append(&mut self.symlinks.explain(&context));
        explanations.append(&mut self.shell.explain(&context));

        explanations
    }
}
