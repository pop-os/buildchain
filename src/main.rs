// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::uninlined_format_args)]

use buildchain::{build, download, BuildArguments, DownloadArguments};
use clap::{Arg, Command};
use std::process;

fn buildchain() -> Result<(), String> {
    let matches = Command::new("buildchain")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("build")
                .about("Build a buildchain project")
                .arg(
                    Arg::new("use_pihsm")
                        .short('p')
                        .long("pihsm")
                        .help("Sign manifest with PiHSM"),
                )
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .num_args(1)
                        .help("Configuration file")
                        .default_value("buildchain.json"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .num_args(1)
                        .help("Output directory")
                        .default_value("buildchain.tar"),
                )
                .arg(
                    Arg::new("project")
                        .long("project")
                        .num_args(1)
                        .help("Tail signature project name")
                        .default_value("default"),
                )
                .arg(
                    Arg::new("branch")
                        .long("branch")
                        .num_args(1)
                        .help("Tail signature branch name")
                        .default_value("master"),
                )
                .arg(
                    Arg::new("remote")
                        .short('r')
                        .long("remote")
                        .num_args(1)
                        .help("Remote LXC server"),
                )
                .arg(
                    Arg::new("source_url")
                        .num_args(1)
                        .help("Source URL")
                        .default_value("."),
                )
                .arg(
                    Arg::new("source_kind")
                        .num_args(1)
                        .help("Source Kind (dir, git)")
                        .default_value("dir"),
                )
                .arg(
                    Arg::new("exclude_source")
                        .long("exclude-source")
                        .help("Exclude the source checkout from the archive"),
                ),
        )
        .subcommand(
            Command::new("download")
                .about("Download from a buildchain project")
                .arg(
                    Arg::new("project")
                        .long("project")
                        .num_args(1)
                        .help("Tail signature project name")
                        .default_value("default"),
                )
                .arg(
                    Arg::new("branch")
                        .long("branch")
                        .num_args(1)
                        .help("Tail signature branch name")
                        .default_value("master"),
                )
                .arg(
                    Arg::new("cert")
                        .long("cert")
                        .num_args(1)
                        .help("Remote URL certificate"),
                )
                .arg(
                    Arg::new("cache")
                        .long("cache")
                        .num_args(1)
                        .help("Local cache"),
                )
                .arg(
                    Arg::new("key")
                        .num_args(1)
                        .required(true)
                        .help("Remote public key"),
                )
                .arg(
                    Arg::new("url")
                        .num_args(1)
                        .required(true)
                        .help("Remote URL"),
                )
                .arg(Arg::new("file").num_args(1).help("Requested file")),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("build") {
        build(BuildArguments {
            config_path: matches.get_one::<String>("config").unwrap(),
            output_path: matches.get_one::<String>("output").unwrap(),
            project_name: matches.get_one::<String>("project").unwrap(),
            branch_name: matches.get_one::<String>("branch").unwrap(),
            remote_opt: matches.get_one::<String>("remote").map(|s| s.as_str()),
            source_url: matches.get_one::<String>("source_url").unwrap(),
            source_kind: matches.get_one::<String>("source_kind").unwrap(),
            use_pihsm: matches.contains_id("use_pihsm"),
            exclude_source: matches.contains_id("exclude_source"),
        })
        .map_err(|err| format!("failed to build: {}", err))
    } else if let Some(matches) = matches.subcommand_matches("download") {
        download(DownloadArguments {
            project: matches.get_one::<String>("project").unwrap(),
            branch: matches.get_one::<String>("branch").unwrap(),
            cert_opt: matches.get_one::<String>("cert").map(|s| s.as_str()),
            cache_opt: matches.get_one::<String>("cache").map(|s| s.as_str()),
            key: matches.get_one::<String>("key").unwrap(),
            url: matches.get_one::<String>("url").unwrap(),
            file_opt: matches.get_one::<String>("file").map(|s| s.as_str()),
        })
    } else {
        Err("no subcommand provided".to_string())
    }
}

fn main() {
    match buildchain() {
        Ok(()) => (),
        Err(err) => {
            eprintln!("buildchain: {}", err);
            process::exit(1);
        }
    }
}
