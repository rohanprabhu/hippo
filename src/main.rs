#[macro_use] extern crate log;
#[macro_use] extern crate clap;
extern crate env_logger;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

mod journaling;
mod snap;
mod utils;

use std::env;
use std::path::Path;
use std::fs;
use journaling::journal::Journal;
use clap::{App, SubCommand, Arg};

fn init_hippo() -> Journal {
    let mut home_dir = if let Some(path) = env::home_dir() {
        if let Some(path_str) = path.to_str() {
            path_str.to_owned()
        } else {
            panic!("Found a home directory, but the encoding of the filename was invalid")
        }
    } else {
        panic!("Could not find home directory for current user, specify prefix manually");
    };

    home_dir.push_str("/.hippo");
    Journal::initialize(home_dir)
}

fn main() {
    env_logger::init().unwrap();
    let mut journal = init_hippo();

    let arg_matches = App::new("hippo")
        .version("0.1")
        .author("Rohan Prabhu <rohan@rohanprabhu.com>")
        .about("Variant manager for configuration-like files")
        .subcommand(SubCommand::with_name("snap")
            .version("0.1")
            .about("Create a new snapshot for a file")
            .arg(Arg::with_name("FILE")
                .help("The files to create a snapshot for, could be absolute or relative to the \
                       current working directory")
                .required(true)
                .index(1)
                .multiple(true)
            )
        )
        .get_matches();

    if let Some(matches) = arg_matches.subcommand_matches("snap") {
        let raw_file_paths = values_t!(matches.values_of("FILE"), String).unwrap();
        let absolute_paths = raw_file_paths
            .iter()
            .map(|path: &String|
                    fs::canonicalize(Path::new(path).to_path_buf()).unwrap())
            .collect::<Vec<_>>();

        snap::snap(&mut journal, absolute_paths);
    }
}
