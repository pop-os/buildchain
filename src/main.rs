// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::uninlined_format_args)]

use buildchain::{build, download, BuildArguments, DownloadArguments};
use clap::{App, Arg};
use std::process;

fn buildchain() -> Result<(), String> {
    let matches = App::new("buildchain")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            App::new("build")
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
                        .takes_value(true)
                        .help("Configuration file"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .takes_value(true)
                        .help("Output directory"),
                )
                .arg(
                    Arg::new("project")
                        .long("project")
                        .takes_value(true)
                        .help("Tail signature project name"),
                )
                .arg(
                    Arg::new("branch")
                        .long("branch")
                        .takes_value(true)
                        .help("Tail signature branch name"),
                )
                .arg(
                    Arg::new("remote")
                        .short('r')
                        .long("remote")
                        .takes_value(true)
                        .help("Remote LXC server"),
                )
                .arg(
                    Arg::new("source_url")
                        .takes_value(true)
                        .help("Source URL"),
                )
                .arg(
                    Arg::new("source_kind")
                        .takes_value(true)
                        .help("Source Kind (dir, git)"),
                )
                .arg(
                    Arg::new("exclude_source")
                        .long("exclude-source")
                        .help("Exclude the source checkout from the archive"),
                ),
        )
        .subcommand(
            App::new("download")
                .about("Download from a buildchain project")
                .arg(
                    Arg::new("project")
                        .long("project")
                        .takes_value(true)
                        .help("Tail signature project name"),
                )
                .arg(
                    Arg::new("branch")
                        .long("branch")
                        .takes_value(true)
                        .help("Tail signature branch name"),
                )
                .arg(
                    Arg::new("cert")
                        .long("cert")
                        .takes_value(true)
                        .help("Remote URL certificate"),
                )
                .arg(
                    Arg::new("cache")
                        .long("cache")
                        .takes_value(true)
                        .help("Local cache"),
                )
                .arg(
                    Arg::new("key")
                        .takes_value(true)
                        .required(true)
                        .help("Remote public key"),
                )
                .arg(
                    Arg::new("url")
                        .takes_value(true)
                        .required(true)
                        .help("Remote URL"),
                )
                .arg(
                    Arg::new("file")
                        .takes_value(true)
                        .help("Requested file"),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("build") {
        build(BuildArguments {
            config_path: matches.value_of("config").unwrap_or("buildchain.json"),
            output_path: matches.value_of("output").unwrap_or("buildchain.tar"),
            project_name: matches.value_of("project").unwrap_or("default"),
            branch_name: matches.value_of("branch").unwrap_or("master"),
            remote_opt: matches.value_of("remote"),
            source_url: matches.value_of("source_url").unwrap_or("."),
            source_kind: matches.value_of("source_kind").unwrap_or("dir"),
            use_pihsm: matches.is_present("use_pihsm"),
            exclude_source: matches.is_present("exclude_source"),
        })
        .map_err(|err| format!("failed to build: {}", err))
    } else if let Some(matches) = matches.subcommand_matches("download") {
        download(DownloadArguments {
            project: matches.value_of("project").unwrap_or("default"),
            branch: matches.value_of("branch").unwrap_or("master"),
            cert_opt: matches.value_of("cert"),
            cache_opt: matches.value_of("cache"),
            key: matches.value_of("key").unwrap(),
            url: matches.value_of("url").unwrap(),
            file_opt: matches.value_of("file"),
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
