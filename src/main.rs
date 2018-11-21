extern crate atomicwrites;
extern crate clap;
extern crate serde_json;

use atomicwrites::{AtomicFile,AllowOverwrite};
use clap::{Arg, App};
use std::fs::File;
use std::io::Write;
use std::io;
use std::path::{Path,PathBuf};

fn update_file(filename: &Path, content: &String) -> io::Result<()> {
    let af = AtomicFile::new(filename, AllowOverwrite);
    af.write(|f| {
        f.write_all(content.as_bytes())
    })?;
    Ok(())
}

fn write_json(filename: &Path, content: &serde_json::Value) -> io::Result<()> {
    let json = serde_json::to_string(content)?;
    update_file(filename, &json)
}

fn read_json(filename: &Path) -> io::Result<serde_json::Value> {
    let file = File::open(filename)?;
    let v = serde_json::from_reader(file)?;
    Ok(v)
}

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
    let spec = read_json(Path::new(input));
}
