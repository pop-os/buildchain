//! Buildchain creates and manages a distributed and reproducible chain of builds

extern crate hex;
extern crate git2;
extern crate lxd;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate sha2;
extern crate tempdir;

pub use self::config::Config;
pub use self::lxd::Location;
pub use self::manifest::Manifest;
pub use self::sha384::Sha384;
pub use self::source::Source;

mod config;
mod manifest;
mod sha384;
mod source;
