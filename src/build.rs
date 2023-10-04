// SPDX-License-Identifier: GPL-3.0-only

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use lxd::{Container, Image, Location};
use tempfile::TempDir;

use crate::{sign_manifest, Config, Sha384, Source, Store};

/// A temporary structure used to generate a unique build environment
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
struct BuildEnvironmentConfig {
    /// The LXC base to use
    pub base: String,
    /// The commands to run to generate a build environment
    pub prepare: Vec<Vec<String>>,
}

fn prepare(config: &Config, location: &Location) -> io::Result<String> {
    let build_json = serde_json::to_string(&BuildEnvironmentConfig {
        base: config.base.clone(),
        prepare: config.prepare.clone(),
    })
    .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    let build_sha = Sha384::new(&mut build_json.as_bytes())
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    let build_sha_str = serde_json::to_string(&build_sha)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    let container_name = format!("buildchain-{}-prepare", config.name);
    let build_image = format!(
        "buildchain-{}-{}",
        config.name,
        build_sha_str.trim_matches('"')
    );

    if Image::new(location.clone(), &build_image).is_ok() {
        println!("Build environment cached as {}", build_image);
    } else {
        let mut container = if config.privileged {
            println!(
                "Create privileged container {} from {}",
                container_name, &config.base
            );
            unsafe { Container::new_privileged(location.clone(), &container_name, &config.base)? }
        } else {
            println!("Create container {} from {}", container_name, &config.base);
            Container::new(location.clone(), &container_name, &config.base)?
        };

        for command in config.prepare.iter() {
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

    Ok(build_image)
}

fn run<P: AsRef<Path>, Q: AsRef<Path>>(
    config: &Config,
    location: &Location,
    build_image: &str,
    source_path: P,
    temp_path: Q,
) -> io::Result<()> {
    let source_path = source_path.as_ref();
    let temp_path = temp_path.as_ref();

    let container_name = format!("buildchain-{}-build", config.name);

    let mut container = if config.privileged {
        println!(
            "Create privileged container {} from {}",
            container_name, build_image
        );
        unsafe { Container::new_privileged(location.clone(), &container_name, build_image)? }
    } else {
        println!("Create container {} from {}", container_name, build_image);
        Container::new(location.clone(), &container_name, build_image)?
    };

    println!("Push source");
    container.push(source_path, "/root", true)?;

    for command in config.build.iter() {
        let mut args = Vec::new();
        for arg in command.iter() {
            args.push(arg.as_str());
        }

        println!("Build command {:?}", args);
        container.exec(&args)?;
    }

    println!("Create artifact directory");
    container.exec(&["mkdir", "/root/artifacts"])?;

    for command in config.publish.iter() {
        let mut args = Vec::new();
        for arg in command.iter() {
            args.push(arg.as_str());
        }

        println!("Publish command {:?}", args);
        container.exec(&args)?;
    }

    println!("Pull artifacts");
    container.pull("/root/artifacts", temp_path, true)?;

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
    pub remote_opt: Option<&'a str>,
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

    let location = if let Some(remote) = args.remote_opt {
        println!("buildchain: building {} on {}", config.name, remote);
        Location::Remote(remote.to_string())
    } else {
        println!("buildchain: building {} locally", config.name);
        Location::Local
    };

    let build_image = prepare(&config, &location)?;

    run(
        &config,
        &location,
        &build_image,
        &source_path,
        temp_dir.path(),
    )?;

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
