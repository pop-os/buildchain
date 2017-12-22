use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

use lxd::{Container, Image, Location};
use serde_json;
use tempdir::TempDir;

use {Config, Manifest, Sha384, Source};
use pihsm::sign_manifest;
use store::Store;

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
    }).map_err(|err| {
        io::Error::new(io::ErrorKind::Other, err)
    })?;

    let build_sha = Sha384::new(&mut build_json.as_bytes()).map_err(|err| {
        io::Error::new(io::ErrorKind::Other, err)
    })?;

    let build_sha_str = serde_json::to_string(&build_sha).map_err(|err| {
        io::Error::new(io::ErrorKind::Other, err)
    })?;

    let container_name = format!("buildchain-{}-prepare", config.name);
    let build_image = format!("buildchain-{}-{}", config.name, build_sha_str.trim_matches('"'));

    if Image::new(location.clone(), &build_image).is_ok() {
        println!("Build environment cached as {}", build_image);
    } else {
        let mut container = if config.privileged {
            println!("Create privileged container {} from {}", container_name, &config.base);
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

fn run<P: AsRef<Path>, Q: AsRef<Path>>(config: &Config, location: &Location, build_image: &str, source_path: P, temp_path: Q) -> io::Result<()> {
    let source_path = source_path.as_ref();
    let temp_path = temp_path.as_ref();

    let container_name = format!("buildchain-{}-build", config.name);

    let mut container = if config.privileged {
        println!("Create privileged container {} from {}", container_name, build_image);
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

fn archive<P: AsRef<Path>, Q: AsRef<Path>>(source_path: P, dest_path: Q) -> io::Result<()> {
    let source_path = source_path.as_ref();
    let dest_path = dest_path.as_ref();

    let status = Command::new("tar")
        .arg("--create")
        .arg("--verbose")
        .arg("--sort=name")
        .arg("--owner=0")
        .arg("--group=0")
        .arg("--numeric-owner")
        .arg("--file").arg(dest_path)
        .arg("--directory").arg(source_path)
        .arg(".")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("tar failed with status: {}", status)
        ))
    }
}

pub struct BuildArguments<'a> {
    pub config_path: &'a str,
    pub output_path: &'a str,
    pub remote_opt: Option<&'a str>,
    pub source_url: &'a str,
    pub source_kind: &'a str,
    pub use_pihsm: bool,
}


pub fn build<'a>(args: BuildArguments<'a>) -> Result<(), String> {
    let config_path = args.config_path;

    let temp_dir = match TempDir::new("buildchain") {
        Ok(dir) => dir,
        Err(err) => {
            return Err(format!("failed to create temporary directory: {}", err));
        }
    };

    let source = Source {
        kind: args.source_kind.to_string(),
        url: args.source_url.to_string()
    };

    let source_path = temp_dir.path().join("source");

    let source_time = match source.download(&source_path) {
        Ok(time) => time,
        Err(err) => {
            return Err(format!("failed to download source {:?}: {}", source, err));
        }
    };

    let mut file = match File::open(&source_path.join(&config_path)) {
        Ok(file) => file,
        Err(err) => {
            return Err(format!("failed to open config {}: {}", config_path, err));
        }
    };

    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => (),
        Err(err) => {
            return Err(format!("failed to read config {}: {}", config_path, err));
        }
    }

    let config = match serde_json::from_str::<Config>(&string) {
        Ok(config) => config,
        Err(err) => {
            return Err(format!("failed to parse config {}: {}", config_path, err));
        }
    };

    let location = if let Some(remote) = args.remote_opt {
        println!("buildchain: building {} on {}", config.name, remote);
        Location::Remote(remote.to_string())
    } else {
        println!("buildchain: building {} locally", config.name);
        Location::Local
    };

    let build_image = match prepare(&config, &location) {
        Ok(build_image) => build_image,
        Err(err) => {
            return Err(format!("failed to prepare config {}: {}", config_path, err));
        }
    };

    match run(&config, &location, &build_image, &source_path, &temp_dir.path()) {
        Ok(()) => (),
        Err(err) => {
            return Err(format!("failed to run config {}: {}", config_path, err));
        }
    }

    let manifest = match Manifest::new(source_time, temp_dir.path().join("artifacts")) {
        Ok(manifest) => manifest,
        Err(err) => {
            return Err(format!("failed to generate manifest: {}", err));
        }
    };
    let manifest_bytes = match serde_json::to_vec_pretty(&manifest) {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err(format!("failed to serialize manifest: {}", err));
        }
    };
    {
        let store = Store::new(&temp_dir);
        store.write_manifest(&manifest_bytes)?;
        if args.use_pihsm {
            let response = match sign_manifest(&manifest_bytes) {
                Ok(response) => response,
                Err(err) => {
                    return Err(format!("failed to sign manifest: {}", err));
                }
            };
            store.write_block(&response)?;
        }
        store.remove_tmp_dir()?;
    }

    match archive(&temp_dir, &args.output_path) {
        Ok(()) => {
            println!("buildchain: placed results in {}", args.output_path);
        },
        Err(err) => {
            return Err(format!("failed to move temporary directory {}: {}", temp_dir.as_ref().display(), err));
        }
    }

    Ok(())
}
