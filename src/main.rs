extern crate buildchain;
extern crate clap;
extern crate serde_json;

use buildchain::Config;
use clap::{App, Arg};
use std::fs::File;
use std::io::Read;
use std::process;

fn main() {
    let matches = App::new("buildchain")
                    .arg(Arg::with_name("config")
                            .help("Build configuration file"))
                    .get_matches();

    let config_path = matches.value_of("config").unwrap_or("buildchain.json");

    let mut file = match File::open(&config_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("buildchain: failed to open {}: {}", config_path, err);
            process::exit(1)
        }
    };

    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("buildchain: failed to read {}: {}", config_path, err);
            process::exit(1)
        }
    }

    let config = match serde_json::from_str::<Config>(&string) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("buildchain: failed to parse {}: {}", config_path, err);
            process::exit(1)
        }
    };

    match config.run() {
        Ok(_) => (),
        Err(err) => {
            eprintln!("buildchain: failed to run {}: {}", config_path, err);
            process::exit(1)
        }
    }
}
