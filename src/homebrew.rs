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

#[cfg(test)]
mod tests {
    #[test]
    fn it_will_install_things_from_homebrew() {}

    #[test]
    fn explaining_homebrew_commands_shows_what_needs_installing() {}

    #[test]
    fn rolling_back_homebrew_will_uninstall_declared_apps() {}
}
