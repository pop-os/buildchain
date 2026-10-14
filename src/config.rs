// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};

/// A build configuration
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// The name of this build project
    pub name: String,
    /// The commands to run to generate a build environment
    pub prepare: Vec<Vec<String>>,
    /// The commands to run that build the artifacts in `source/`
    pub build: Vec<Vec<String>>,
    /// The commands to run that publish the artifacts to `artifacts/`
    pub publish: Vec<Vec<String>>,
}
