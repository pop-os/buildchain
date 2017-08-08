use std::io;
use std::process::Command;

use super::Location;

fn lxc(args: &[&str]) -> io::Result<()> {
    let mut cmd = Command::new("lxc");
    for arg in args.iter() {
        cmd.arg(arg);
    }

    let status = cmd.spawn()?.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("LXC {:?} failed with {}", args, status)
        ))
    }
}

/// An LXC container
pub struct Lxc(String);

impl Lxc {
    /// Create a new LXC container
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the container. This will be prepended with `buildchain-`
    /// * `base` - The base distribution to use, `ubuntu:16.04` for example
    ///
    /// # Return
    ///
    /// The newly created LXC container
    ///
    /// # Errors
    ///
    /// Errors that are encountered while creating will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use buildchain::{Location, Lxc};
    ///
    /// let mut lxc = Lxc::new(Location::Local, "test-new", "ubuntu:16.04").unwrap();
    /// ```
    pub fn new(location: Location, name: &str, base: &str) -> io::Result<Lxc> {
        let full_name = match location {
            Location::Local => format!("buildchain-{}", name),
            Location::Remote(remote) => format!("{}:buildchain-{}", remote, name)
        };

        lxc(&["launch", base, &full_name, "-e", "-n", "lxdbr0"])?;

        // Hack to wait for network up and running
        lxc(&["exec", &full_name, "--mode=non-interactive", "-n", "--", "dhclient"])?;

        Ok(Lxc(full_name))
    }

    /// Run a command in an LXC container
    ///
    /// # Arguments
    ///
    /// * `command` - An array of command arguments
    ///
    /// # Return
    ///
    /// And empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while executing will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use buildchain::{Location, Lxc};
    ///
    /// let mut lxc = Lxc::new(Location::Local, "test-exec", "ubuntu:16.04").unwrap();
    /// lxc.exec(&["echo", "hello"]).unwrap();
    /// ```
    pub fn exec(&mut self, command: &[&str]) -> io::Result<()> {
        let mut args = vec!["exec", &self.0, "--"];
        for arg in command.as_ref().iter() {
            args.push(arg.as_ref());
        }
        lxc(&args)
    }

    /// Mount a path in an LXC container
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the mount
    /// * `source` - The source path to mount
    /// * `dest` - The destination of the mount
    ///
    /// # Return
    ///
    /// And empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while mounting will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use buildchain::{Location, Lxc};
    ///
    /// let mut lxc = Lxc::new(Location::Local, "test-mount", "ubuntu:16.04").unwrap();
    /// lxc.mount("source", ".", "/root/source").unwrap();
    /// ```
    pub fn mount(&mut self, name: &str, source: &str, dest: &str) -> io::Result<()> {
        lxc(&["config", "device", "add", &self.0, name, "disk", &format!("source={}", source), &format!("path={}", dest)])
    }

    /// Pull a file from the LXC container
    ///
    /// # Arguments
    ///
    /// * `source` - The source of the file in the container
    /// * `dest` - The destination of the file in the host
    ///
    /// # Return
    ///
    /// And empty tuple on success
    ///
    /// # Errors
    ///
    /// Errors that are encountered while mounting will be returned
    ///
    /// # Example
    ///
    /// ```
    /// use buildchain::{Location, Lxc};
    ///
    /// let mut lxc = Lxc::new(Location::Local, "test-pull", "ubuntu:16.04").unwrap();
    /// lxc.pull("/etc/hostname", "target/hostname").unwrap();
    /// ```
    pub fn pull(&mut self, source: &str, dest: &str) -> io::Result<()> {
        lxc(&["file", "pull", &format!("{}/{}", self.0, source), dest])
    }
}

impl Drop for Lxc {
    fn drop(&mut self) {
        let _ = lxc(&["stop", &self.0]);
    }
}
