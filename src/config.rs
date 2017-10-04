use lxd::{Location, Container, Image};
use serde_json;
use std::io;
use tempdir::TempDir;

use super::{Sha384, Source};

/// A build configuration
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// The name of this build project
    pub name: String,
    /// The LXC base to use
    pub base: String,
    /// The source repository (git only, for now)
    pub source: Source,
    /// The commands to run to generate a build environment
    pub prepare: Vec<Vec<String>>,
    /// The commands to run that build the artifacts in /root/source
    pub build: Vec<Vec<String>>,
    /// The commands to run that publish the artifacts to /root/artifacts
    pub publish: Vec<Vec<String>>,
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
    pub fn run(&self, location: Location) -> io::Result<(u64, TempDir)> {
        println!("Create temporary directory");
        let temp_dir = TempDir::new("buildchain")?;

        println!("Download source using {}: {}", self.source.kind, self.source.url);
        let time = self.source.download(temp_dir.path().join("source"))?;

        let container_name = format!("buildchain-{}-{}", self.name, time);

        let prepare_json = serde_json::to_string(&self.prepare).map_err(|err| {
            io::Error::new(io::ErrorKind::Other, err)
        })?;
        let prepare_sha = Sha384::new(&mut prepare_json.as_bytes()).map_err(|err| {
            io::Error::new(io::ErrorKind::Other, err)
        })?;
        let prepare_sha_str = serde_json::to_string(&prepare_sha).map_err(|err| {
            io::Error::new(io::ErrorKind::Other, err)
        })?;
        let build_image = format!("buildchain-{}-{}", self.name, prepare_sha_str.trim_matches('"'));

        if Image::new(location.clone(), &build_image).is_ok() {
            println!("Build environment cached as {}", build_image);
        } else {
            println!("Create container {} from {}", container_name, self.base);
            let mut container = Container::new(location.clone(), &container_name, &self.base)?;

            for command in self.prepare.iter() {
                let mut args = vec![];
                for arg in command.iter() {
                    args.push(arg.as_str());
                }

                println!("Prepare command {:?}", args);
                container.exec(&args)?;
            }

            println!("Snapshot build environment as {}", build_image);
            let snapshot = container.snapshot(&build_image)?;

            println!("Publish build environment as {}", build_image);
            snapshot.publish(&build_image)?;
        }

        {
            println!("Create container {} from {}", container_name, build_image);
            let mut container = Container::new(location, &container_name, &build_image)?;

            println!("Push source");
            container.push(temp_dir.path().join("source"), "/root", true)?;

            for command in self.build.iter() {
                let mut args = vec![];
                for arg in command.iter() {
                    args.push(arg.as_str());
                }

                println!("Build command {:?}", args);
                container.exec(&args)?;
            }

            println!("Create artifact directory");
            container.exec(&["mkdir", "/root/artifacts"])?;

            for command in self.publish.iter() {
                let mut args = vec![];
                for arg in command.iter() {
                    args.push(arg.as_str());
                }

                println!("Publish command {:?}", args);
                container.exec(&args)?;
            }

            println!("Pull artifacts");
            container.pull("/root/artifacts", temp_dir.path(), true)?;
        }

        Ok((time, temp_dir))
    }
}
