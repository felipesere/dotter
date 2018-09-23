use crate::{Command, Context, Explanation, Result};
use std::process::{self};

#[derive(Deserialize, Debug)]
pub struct ShellCommand {
    run: String,
}

impl Command for ShellCommand {
    fn execute(&self, _context: &Context) -> Result<()> {
        process::Command::new("sh")
            .arg("-c")
            .arg(&self.run)
            .status()?;

        Ok(())
    }

    fn rollback(&self, _context: &Context) ->Result<()> {
        Ok(())
    }

    fn explain(&self, _context: &Context) -> Result<Vec<Explanation>> {
        // do something clever to check if target/source exist
        Ok(vec![Explanation::new("this is from the shell script")])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_runs_a_simple_shell_command() {
        let context = Context::default();

        let echo_command = ShellCommand {
            run: "echo \"Hi there\"".to_string(),
        };

        echo_command.execute(&context);
    }
}
