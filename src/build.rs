// SPDX-License-Identifier: GPL-3.0-only

use std::fs;
use std::io;
use std::env;
use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

use crate::{sign_manifest, Config, Source, Store};

fn prepare(config: &Config) -> io::Result<()> {
    for command in config.prepare.iter() {
        let mut args = Vec::new();
        for arg in command.iter().skip(1) {
            args.push(arg.as_str());
        }

        println!("Prepare command: {} {:?}", &command[0], args);
        Command::new(&command[0]).args(&args).status()?;
    }

    Ok(())
}

fn run(config: &Config) -> io::Result<()> {
    for command in config.build.iter() {
        let mut args = Vec::new();
        for arg in command.iter().skip(1) {
            args.push(arg.as_str());
        }

        println!("Build command: {} {:?}", &command[0], args);
        Command::new(&command[0]).args(&args).status()?;
    }

    println!("Create artifact directory");
    fs::create_dir_all("artifacts")?;

    for command in config.publish.iter() {
        let mut args = Vec::new();
        for arg in command.iter().skip(1) {
            args.push(arg.as_str());
        }

        println!("Publish command: {} {:?}", &command[0], args);
        Command::new(&command[0]).args(&args).status()?;
    }

    Ok(())
}

fn archive<P: AsRef<Path>, Q: AsRef<Path>>(
    source_path: P,
    dest_path: Q,
    exclude_source: bool,
) -> io::Result<()> {
    let source_path = source_path.as_ref();
    let dest_path = dest_path.as_ref();

    let mut args = vec![
        "--create",
        "--verbose",
        "--sort=name",
        "--owner=0",
        "--group=0",
        "--numeric-owner",
        "--exclude-vcs",
    ];

    if exclude_source {
        args.push("--exclude=./source")
    }

    let status = Command::new("tar")
        .args(args)
        .arg("--file")
        .arg(dest_path)
        .arg("--directory")
        .arg(source_path)
        .arg(".")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("tar failed with status: {}", status),
        ))
    }
}

pub struct BuildArguments<'a> {
    pub config_path: &'a str,
    pub output_path: &'a str,
    pub project_name: &'a str,
    pub branch_name: &'a str,
    pub source_url: &'a str,
    pub source_kind: &'a str,
    pub use_pihsm: bool,
    pub exclude_source: bool,
}

pub fn build(args: BuildArguments) -> io::Result<()> {
    let config_path = args.config_path;

    let temp_dir = TempDir::with_prefix("buildchain.")?;

    let source = Source {
        kind: args.source_kind.to_string(),
        url: args.source_url.to_string(),
    };

    let source_path = temp_dir.path().join("source");

    let source_time = source.download(&source_path)?;

    let string = fs::read_to_string(source_path.join(config_path))?;
    let config = serde_json::from_str::<Config>(&string)?;

    println!("buildchain: building {}", config.name);

    // Run all commands from the context of the buildroot.
    let cwd = env::current_dir()?;
    env::set_current_dir(&temp_dir)?;

    prepare(&config)?;
    run(&config)?;

    env::set_current_dir(cwd)?;

    let store = Store::new(&temp_dir);
    let manifest = store.import_artifacts(source_time)?;
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;

    store.write_manifest(&manifest_bytes)?;
    if args.use_pihsm {
        let response = sign_manifest(&manifest_bytes)?;
        store.write_tail(args.project_name, args.branch_name, &response)?;
    }
    store.remove_tmp_dir()?;

    archive(&temp_dir, args.output_path, args.exclude_source)?;

    println!("buildchain: placed results in {}", args.output_path);

    Ok(())
}
