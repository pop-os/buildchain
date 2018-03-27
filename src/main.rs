extern crate buildchain;
extern crate clap;

use buildchain::{build, BuildArguments, download, DownloadArguments};
use clap::{App, Arg, SubCommand};
use std::io::{self, Write};
use std::process;

fn buildchain() -> Result<(), String> {
    let matches = App::new("buildchain")
        .subcommand(
            SubCommand::with_name("build")
                .about("Build a buildchain project")
                .arg(
                    Arg::with_name("use_pihsm")
                        .short("p")
                        .long("pihsm")
                        .help("Sign manifest with PiHSM")
                )
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .takes_value(true)
                        .help("Configuration file")
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .takes_value(true)
                        .help("Output directory")
                )
                .arg(
                    Arg::with_name("project")
                        .long("project")
                        .takes_value(true)
                        .help("Tail signature project name")
                )
                .arg(
                    Arg::with_name("branch")
                        .long("branch")
                        .takes_value(true)
                        .help("Tail signature branch name")
                )
                .arg(
                    Arg::with_name("remote")
                        .short("r")
                        .long("remote")
                        .takes_value(true)
                        .help("Remote LXC server")
                )
                .arg(
                    Arg::with_name("source_url")
                        .takes_value(true)
                        .help("Source URL")
                )
                .arg(
                    Arg::with_name("source_kind")
                        .takes_value(true)
                        .help("Source Kind (dir, git)")
                )
        )
        .subcommand(
            SubCommand::with_name("download")
                .about("Download from a buildchain project")
                .arg(
                    Arg::with_name("project")
                        .long("project")
                        .takes_value(true)
                        .help("Tail signature project name")
                )
                .arg(
                    Arg::with_name("branch")
                        .long("branch")
                        .takes_value(true)
                        .help("Tail signature branch name")
                )
                .arg(
                    Arg::with_name("key")
                        .takes_value(true)
                        .required(true)
                        .help("Remote public key")
                )
                .arg(
                    Arg::with_name("url")
                        .takes_value(true)
                        .required(true)
                        .help("Remote URL")
                )
                .arg(
                    Arg::with_name("cache")
                        .takes_value(true)
                        .required(true)
                        .help("Local cache")
                )
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
        })
    } else if let Some(matches) = matches.subcommand_matches("download") {
        download(DownloadArguments {
            project_name: matches.value_of("project").unwrap_or("default"),
            branch_name: matches.value_of("branch").unwrap_or("master"),
            key: matches.value_of("key").unwrap(),
            url: matches.value_of("url").unwrap(),
            cache: matches.value_of("cache").unwrap(),
        })
    } else {
        Err(format!("no subcommand provided"))
    }
}

fn main() {
    match buildchain() {
        Ok(()) => (),
        Err(err) => {
            writeln!(io::stderr(), "buildchain: {}", err).unwrap();
            process::exit(1);
        }
    }
}
