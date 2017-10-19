extern crate buildchain;
extern crate clap;
extern crate serde_json;

use buildchain::{Config, Location, Manifest};
use clap::{App, Arg};
use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

fn buildchain() -> Result<(), String> {
    let matches = App::new("buildchain")
                    .arg(Arg::with_name("config")
                            .short("c")
                            .long("config")
                            .takes_value(true)
                            .help("Build configuration file"))
                    .arg(Arg::with_name("output")
                            .short("o")
                            .long("output")
                            .takes_value(true)
                            .help("Build output directory"))
                    .arg(Arg::with_name("remote")
                            .short("r")
                            .long("remote")
                            .takes_value(true)
                            .help("Name of remote LXC server"))
                    .get_matches();

    let config_path = matches.value_of("config").unwrap_or("buildchain.json");
    let _output_path = matches.value_of("output").unwrap_or("buildchain.out");
    let remote_opt = matches.value_of("remote");

    let mut file = match File::open(&config_path) {
        Ok(file) => file,
        Err(err) => {
            return Err(format!("failed to open {}: {}", config_path, err));
        }
    };

    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => (),
        Err(err) => {
            return Err(format!("failed to read {}: {}", config_path, err));
        }
    }

    let config = match serde_json::from_str::<Config>(&string) {
        Ok(config) => config,
        Err(err) => {
            return Err(format!("failed to parse {}: {}", config_path, err));
        }
    };

    let location = if let Some(remote) = remote_opt {
        println!("buildchain: building {} on {}", config.name, remote);
        Location::Remote(remote.to_string())
    } else {
        println!("buildchain: building {} locally", config.name);
        Location::Local
    };

    let (time, temp_dir) = match config.run(location) {
        Ok(t) => t,
        Err(err) => {
            return Err(format!("failed to run {}: {}", config_path, err));
        }
    };

    println!("{}", temp_dir.path().display());
    let manifest = match Manifest::new(time, temp_dir.path().join("artifacts")) {
        Ok(manifest) => manifest,
        Err(err) => {
            return Err(format!("failed to generate manifest: {}", err));
        }
    };

    println!("{:?}", manifest);

    match File::create(temp_dir.path().join("manifest.json")) {
        Ok(mut file) => {
            if let Err(err) = serde_json::to_writer_pretty(&mut file, &manifest) {
                return Err(format!("failed to write manifest: {}", err));
            }
            if let Err(err) = file.sync_all() {
                return Err(format!("failed to sync manifest: {}", err));
            }
        },
        Err(err) => {
            return Err(format!("failed to create manifest: {}", err));
        }
    }

    println!("TODO: copy and remove {}", temp_dir.into_path().display());

    Ok(())
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
