extern crate buildchain;
extern crate clap;

use buildchain::{build, BuildArguments};
use clap::{App, Arg, SubCommand};
use std::io::{self, Write};
use std::process;

fn buildchain() -> Result<(), String> {
    let matches = App::new("buildchain")
        .subcommand(
            SubCommand::with_name("build")
                .about("Build a buildchain project")
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
    .get_matches();

    if let Some(matches) = matches.subcommand_matches("build") {
        build(BuildArguments {
            config_path: matches.value_of("config").unwrap_or("buildchain.json"),
            output_path: matches.value_of("output").unwrap_or("buildchain.tar"),
            remote_opt: matches.value_of("remote"),
            source_url: matches.value_of("source_url").unwrap_or("."),
            source_kind: matches.value_of("source_kind").unwrap_or("dir"),
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
