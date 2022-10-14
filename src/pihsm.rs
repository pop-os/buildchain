// SPDX-License-Identifier: GPL-3.0-only

use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

pub fn sign_manifest(manifest: &[u8]) -> io::Result<[u8; 400]> {
    let mut response = [0u8; 400];
    {
        println!("Calling pihsm-request to sign manifest...");
        let mut child = Command::new("/usr/bin/pihsm-request")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = child.stdin.take().expect("failed to get stdin");
            stdin.write_all(manifest)?;
            stdin.flush()?;
        }
        {
            let stdout = child.stdout.as_mut().expect("failed to get stdout");
            let bytes = stdout.read(&mut response)?;
            if bytes != response.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "pihsm-request: expected {} bytes, got {}",
                        response.len(),
                        bytes
                    ),
                ));
            }
        }
        child.wait()?;
        println!("Successfully signed manifest with pihsm-request.");
    }
    Ok(response)
}
