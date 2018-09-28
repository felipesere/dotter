use crate::{Command, Context, Direction, Explanation, Result};
use symlink::{remove_symlink_file, symlink_file};
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Symlink {
    from: String,
    to: String,
}

impl Command for Symlink {
    fn execute(&self, context: &Context) -> Result<()> {
        let current_dir = context.current_dir();
        let destination = interpolate(&self.to, context);

        let parent = destination.parent().unwrap();
        std::fs::create_dir_all(parent).expect("Trying to create parents");

        symlink_file(current_dir.join(&self.from), current_dir.join(&destination))?;
        Ok(())
    }

    fn rollback(&self, context: &Context) -> Result<()> {
        let current_dir = context.current_dir();
        let destination = interpolate(&self.to, context);

        remove_symlink_file(current_dir.join(&destination))?;
        Ok(())
    }

    fn explain(&self, context: &Context) -> Result<Vec<Explanation>> {
        let destination = interpolate(&self.to, context);
        let message = match context.direction {
            Direction::Execute => {
                if destination.exists() {
                    format!("Symmlink to {} already exists", destination.display())
                } else {
                    format!("adding a link from {} to {}", self.from, destination.display())
                }
            },
            Direction::Rollback => {
                if destination.exists() {
                    format!("Removing symmlink to {}", destination.display())
                } else {
                    format!("Symmlink to {} did not exist", destination.display())
                }
            },
        };

        Ok(vec![Explanation::new(message)])
    }
}


fn interpolate(target: &str, context: &Context) -> PathBuf {
    if !target.contains("$") {
        return context.current_dir().join(target)
    }

    let mut better_target = target.to_string();
    for (key, value) in context.environment.iter() {
        if  target.contains(key) {
            let x = format!("${}", key);
            better_target = better_target.replace(&x, &value);
        }
    }
    context.current_dir().join(better_target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::{tempdir, TempDir};
    use maplit::hashmap;

    fn given_these_files_exist(names: &[&'static str]) -> TempDir {
        let dir = tempdir().unwrap();
        for name in names {
            let file_path = dir.path().join(name);
            let mut tmp_file = File::create(file_path).unwrap();
            writeln!(tmp_file, "some boring text").unwrap();
        }

        dir
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
            environment: environment,
            ..Context::default()
        };

        linker.execute(&context);

        let paths = std::fs::read_dir(&context.working_directory.join("fancy_subdir")).unwrap();
        assert_eq!(paths.count(), 1);

        linker.rollback(&context);

        let after = std::fs::read_dir(&context.working_directory.join("fancy_subdir")).unwrap();
        assert_eq!(after.count(), 0);
    }

    #[test]
    fn it_will_inform_about_unnecessary_links() {
        let dir = given_these_files_exist(&["original.txt", "the_copy.txt"]);

        let linker = Symlink {
            from: "original.txt".to_string(),
            to: "the_copy.txt".to_string(),
        };

        let context = Context {
            working_directory: dir.into_path(),
            ..Context::default()
        };

        let explanations = linker.explain(&context).unwrap();

        let expected = format!("Symmlink to {}/the_copy.txt already exists", context.working_directory.display());

        assert_eq!(explanations.get(0).unwrap().message, expected);
    }

    #[test]
    fn it_will_inform_about_removing_nonexisting_links() {
        let dir = given_a_file_exists("original.txt");

        let linker = Symlink {
            from: "original.txt".to_string(),
            to: "the_copy.txt".to_string(),
        };

        let context = Context {
            working_directory: dir.into_path(),
            direction: Direction::Rollback,
            ..Context::default()
        };

        let explanations = linker.explain(&context).unwrap();

        let expected = format!("Symmlink to {}/the_copy.txt did not exist", context.working_directory.display());

        assert_eq!(explanations.get(0).unwrap().message, expected);
    }
}
