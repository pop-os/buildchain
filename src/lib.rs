// SPDX-License-Identifier: GPL-3.0-only

//! Buildchain creates and manages a distributed and reproducible chain of builds

#![allow(clippy::uninlined_format_args)]

pub use lxd::Location;

pub use crate::block::Block;
pub use crate::build::{build, BuildArguments};
pub use crate::config::Config;
pub use crate::download::{download, DownloadArguments, Downloader};
pub use crate::manifest::Manifest;
pub use crate::pihsm::sign_manifest;
pub use crate::sha384::Sha384;
pub use crate::source::Source;
pub use crate::store::Store;

mod block;
mod build;
mod config;
mod download;
mod manifest;
mod pihsm;
mod sha384;
mod source;
mod store;

// Helper function for errors
pub(crate) fn err_str<E: ::std::error::Error>(err: E) -> String {
    format!("{}: {:?}", err, err)
}
