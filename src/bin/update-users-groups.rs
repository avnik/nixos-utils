extern crate clap;
extern crate nixos_utils;

use clap::{Arg, App};
use std::path::{Path};

fn main() {
    let matches = App::new("update-users-groups")
        .version("0.1.0")
        .arg(Arg::with_name("root")
             .short("r")
             .long("root")
             .takes_value(true))
        .arg(Arg::with_name("INPUT")
             .required(true)
             .index(1))
        .get_matches();

    let root = matches.value_of("root").unwrap_or("/");
    let input = matches.value_of("INPUT").unwrap();
    let spec = nixos_utils::read_json(Path::new(input));
}
