#[macro_use] extern crate log;
#[macro_use] extern crate clap;
#[macro_use] extern crate prettytable;
extern crate env_logger;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate colored;

mod journaling;
mod snap;
mod list;
mod utils;

use std::env;
use std::path::Path;
use std::fs;
use journaling::journal::Journal;
use clap::{App, SubCommand, Arg};
use list::HippoList;

use colored::*;

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

struct HippoError;

impl From<list::ListError> for HippoError {
    fn from(_: list::ListError) -> Self {
        HippoError {}
    }
}

impl From<clap::Error> for HippoError {
    fn from(_: clap::Error) -> Self { HippoError {} }
}

fn main_func() -> Result<(), HippoError> {
    let mut journal = init_hippo();

    let mut clap_app = App::new("hippo".magenta().to_string())
        .version("0.1")
        .author("Rohan Prabhu <rohan@rohanprabhu.com>")
        .about("Variant manager for configuration-like files. Hippo keeps track of everything. EVERYTHING.")
        .subcommand(SubCommand::with_name("snap")
            .version("0.1")
            .about("Create a new snapshot for given files")
            .arg(Arg::with_name("FILE")
                .help("The files to create a snapshot for, could be absolute or relative to the \
                       current working directory")
                .required(true)
                .multiple(true)
            )
            .arg(Arg::with_name("name")
                .long("name")
                .short("n")
                .value_name("name")
                .help("Name of the snapshot. If not provided, a default one is generated \n\
                      which is a specially formatted date-time string")
                .required(false)
                .multiple(false)
            )
            .arg(Arg::with_name("comment")
                .long("comment")
                .short("c")
                .value_name("comment")
                .help("A comment for the snapshot. If not provided, a default one is generated \n\
                       based on the time this snapshot was created")
                .required(false)
                .multiple(false)
            )
        )
        .subcommand(SubCommand::with_name("list")
            .version("0.1")
            .about("List all snapshots for given files")
            .arg(Arg::with_name("FILE")
                .help("The files to list the snapshot for, could be absolute or relative to the \
                       current working directory")
                .required(true)
                .index(1)
                .multiple(true)
            )
        );

    let arg_matches = clap_app.to_owned().get_matches();

    if let Some(matches) = arg_matches.subcommand_matches("snap") {
        let raw_file_paths = values_t!(matches.values_of("FILE"), String).unwrap();
        let absolute_paths = raw_file_paths
            .iter()
            .map(|path: &String|
                fs::canonicalize(Path::new(path).to_path_buf()).unwrap()
            )
            .collect::<Vec<_>>();

        snap::snap(&mut journal,
                   value_t!(matches.value_of("name"), String).ok(),
                   value_t!(matches.value_of("comment"), String).ok(),
                   absolute_paths
        );

    } else if let Some(matches) = arg_matches.subcommand_matches("list") {
        let raw_file_paths = values_t!(matches.values_of("FILE"), String).unwrap();
        let absolute_paths = raw_file_paths
            .iter()
            .map(|path: &String|
                fs::canonicalize(Path::new(path).to_path_buf()).unwrap())
            .collect::<Vec<_>>();

        HippoList::new().list(&mut journal, absolute_paths)?;
    } else {
        clap_app.print_help()?;
    }

    Ok(())
}

fn main() {
    env_logger::init().unwrap();
    ::std::process::exit(match main_func() {
        Ok(_) => 0,
        Err(_) => {
            // Global error handler to be called (or whatever is the rustic way is)
            1
        }
    });
}
