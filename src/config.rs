use lxd::{Location, Container};
use std::io;

/// A build configuration
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// The name of this build project
    pub name: String,
    /// The LXC base to use
    pub base: String,
    /// The commands to run that generate the build artifacts
    pub commands: Vec<Vec<String>>,
    /// A directory of build artifacts
    pub artifacts: Option<String>,
}

impl Config {
    /// Run a build configuration
    ///
    /// # Return
    ///
    /// An empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while running will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use buildchain::{Config, Location};
    /// use std::collections::BTreeMap;
    ///
    /// let config = Config {
    ///     name: "test-config".to_string(),
    ///     base: "ubuntu:16.04".to_string(),
    ///     commands: vec![vec!["echo".to_string(), "hello".to_string()]],
    ///     artifacts: None,
    /// };
    /// config.run(Location::Local, "tests/res/config/buildchain.out").unwrap();
    /// ```
    pub fn run(&self, location: Location, output: &str) -> io::Result<()> {
        let mut container = Container::new(location, &format!("buildchain-{}", self.name), &self.base)?;
        for command in self.commands.iter() {
            let mut args = vec![];
            for arg in command.iter() {
                args.push(arg.as_str());
            }
            container.exec(&args)?;
        }

        if let Some(ref artifacts) = self.artifacts {
            container.pull(&artifacts, output, true)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use std::fs::File;
    use std::io::Read;

    use super::{Config, Location};

    #[test]
    fn test_build() {
        let config = {
            let mut file = File::open("tests/res/config/buildchain.json").unwrap();
            let mut json = String::new();
            file.read_to_string(&mut json).unwrap();
            serde_json::from_str::<Config>(&json).unwrap()
        };

        config.run(Location::Local, "tests/res/config/buildchain.out").unwrap();
    }
}
