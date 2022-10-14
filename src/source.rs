// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

/// A source code repository
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Source {
    pub kind: String,
    pub url: String,
}

impl Source {
    /// Download the source code repository to the given directory
    //TODO: More documentation, code example
    pub fn download<P: AsRef<Path>>(&self, directory: P) -> io::Result<u64> {
        match self.kind.as_str() {
            "dir" => {
                let status = Command::new("cp")
                    .arg("--preserve=all")
                    .arg("--recursive")
                    .arg(&self.url)
                    .arg(directory.as_ref())
                    .spawn()?
                    .wait()?;

                if !status.success() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Copy error: {}", status),
                    ));
                }

                let output = Command::new("find")
                    .arg(directory.as_ref())
                    .arg("-type")
                    .arg("f")
                    .arg("-printf")
                    .arg("%T@\\n")
                    .stdout(Stdio::piped())
                    .spawn()?
                    .wait_with_output()?;

                if !output.status.success() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Find error: {}", output.status),
                    ));
                }

                let stdout = String::from_utf8(output.stdout).map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Find output not UTF-8: {}", err),
                    )
                })?;

                let mut time_opt = None;
                for line in stdout.trim().lines() {
                    let mut parts = line.trim().split('.');

                    let time_str = parts.next().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::Other, "Find time not valid".to_string())
                    })?;

                    let time = time_str.parse::<u64>().map_err(|err| {
                        io::Error::new(
                            io::ErrorKind::Other,
                            format!("Find time not a number: {}", err),
                        )
                    })? as u64;

                    time_opt = match time_opt {
                        Some(old_time) => {
                            if time > old_time {
                                Some(time)
                            } else {
                                Some(old_time)
                            }
                        }
                        None => Some(time),
                    };
                }

                match time_opt {
                    Some(time) => Ok(time),
                    None => Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Find time not found".to_string(),
                    )),
                }
            }
            "git" => {
                let status = Command::new("git")
                    .arg("clone")
                    .arg("--recursive")
                    .arg(&self.url)
                    .arg(directory.as_ref())
                    .spawn()?
                    .wait()?;

                if !status.success() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Git clone error: {}", status),
                    ));
                }

                let output = Command::new("git")
                    .arg("-C")
                    .arg(directory.as_ref())
                    .arg("log")
                    .arg("-1")
                    .arg("--format=%ct")
                    .stdout(Stdio::piped())
                    .spawn()?
                    .wait_with_output()?;

                if !output.status.success() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Git log error: {}", output.status),
                    ));
                }

                let stdout = String::from_utf8(output.stdout).map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Git log output not UTF-8: {}", err),
                    )
                })?;

                let time = stdout.trim().parse::<u64>().map_err(|err| {
                    io::Error::new(
                        io::ErrorKind::Other,
                        format!("Git log time not a number: {}", err),
                    )
                })?;

                Ok(time)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Unknown source kind: {}", self.kind),
            )),
        }
    }
}
