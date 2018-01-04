//! Buildchain creates and manages a distributed and reproducible chain of builds

extern crate base32;
extern crate lxd;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate tempdir;

pub use self::build::{build, BuildArguments};
pub use self::config::Config;
pub use self::lxd::Location;
pub use self::manifest::Manifest;
pub use self::pihsm::sign_manifest;
pub use self::sha384::Sha384;
pub use self::source::Source;
pub use self::store::Store;

mod build;
mod config;
mod manifest;
mod pihsm;
mod sha384;
mod source;
mod store;
