use serde::{Deserialize, Serialize};

/// A build configuration
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// The name of this build project
    pub name: String,
    /// The LXC base to use
    pub base: String,
    /// True if the LXC container for builds should be privileged
    #[serde(default = "Default::default")]
    pub privileged: bool,
    /// The commands to run to generate a build environment
    pub prepare: Vec<Vec<String>>,
    /// The commands to run that build the artifacts in /root/source
    pub build: Vec<Vec<String>>,
    /// The commands to run that publish the artifacts to /root/artifacts
    pub publish: Vec<Vec<String>>,
}
