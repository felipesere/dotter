use crate::{Command, Context, Direction, Source, Explanation};
use std::collections::HashMap;
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
        let destination = interpolate(&self.to, &context.environment);

        symlink_file(current_dir.join(&self.from), current_dir.join(&destination))
            .expect("Could not create symlink");
    }

    fn rollback(&self, context: &Context) {
        let current_dir = context.current_dir();
        let destination = interpolate(&self.to, &context.environment);

        remove_symlink_file(current_dir.join(&destination)).expect("Could not remove symlink");
    }

    fn explain(&self, context: &Context) -> Vec<Explanation> {
        match context.direction {
            Direction::Execute => vec![Explanation::new("some symlinks will be applied")],
            Direction::Rollback => vec![Explanation::new("we are rolling back symlinks")],
        }
    }
}

fn interpolate(target: &str, values: &HashMap<String, String>) -> String {
    if !target.contains("$") {
        return target.to_string();
    }

    let mut better_target = target.to_string();
    for (key, value) in values {
        if  target.contains(key) {
            let x = format!("${}", key);
            better_target = better_target.replace(&x, value);
        }
    }
    better_target
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::{tempdir, TempDir};
    use maplit::hashmap;

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
    fn it_will_interpolate_home_directory() {
        let dir = given_a_file_exists("original.txt");
        std::fs::create_dir(dir.path().join("fancy_subdir")).expect("Was not able to create subdirectory");

        let linker = Symlink {
            from: "original.txt".to_string(),
            to: "$SOME_ENV_FLAG/the_copy.txt".to_string(),
        };

        let environment = hashmap! {
            "SOME_ENV_FLAG".to_string() => "./fancy_subdir".to_string()
        };

        let context = Context {
            working_directory: dir.into_path(),
            environment: environment
        };

        linker.execute(&context);

        let paths = std::fs::read_dir(&context.working_directory.join("fancy_subdir")).unwrap();
        assert_eq!(paths.count(), 1);

        linker.rollback(&context);

        let after = std::fs::read_dir(&context.working_directory.join("fancy_subdir")).unwrap();
        assert_eq!(after.count(), 0);
    }
}
