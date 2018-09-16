use crate::{Command, Context, Source};

use symlink::{remove_symlink_file, symlink_file};

#[derive(Deserialize, Debug)]
pub struct Symlink {
    from: String,
    to: String,
}

struct Symlinker {}

impl Source for Symlinker {
    const NAME: &'static str = "symlinks";
    type Item = Symlink;

    fn perform(&self, command: Symlink) -> bool {
        true
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::{tempdir, TempDir};

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
            ..Context::default()
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
