use atomicwrites::{AtomicFile,AllowOverwrite};
use std::fs::File;
use std::fs;
use std::io::Write;
use std::io;
use std::path::{Path};

pub fn write_file(filename: &Path, content: &String) -> io::Result<()> {
    let af = AtomicFile::new(filename, AllowOverwrite);
    af.write(|f| {
        f.write_all(content.as_bytes())
    })?;
    Ok(())
}

pub fn write_json(filename: &Path, content: &serde_json::Value) -> io::Result<()> {
    // FIXME: use serde_json::to_writeable
    let json = serde_json::to_string(content)?;
    write_file(filename, &json)
}

pub fn read_json(filename: &Path) -> io::Result<serde_json::Value> {
    let file = File::open(filename)?;
    let v = serde_json::from_reader(file)?;
    Ok(v)
}

pub fn read_json_or_empty(filename: &Path) -> serde_json::Value {
    read_json(filename).unwrap_or(json!({}))
}

pub fn read_list(filename: &Path) -> io::Result<Vec<String>> {
    let file = fs::read_to_string(filename)?;
    let list = file.split_whitespace().map(|x| x.to_string()).collect();
    Ok(list)
}

pub fn write_list(filename: &Path, values: Vec<String>) -> io::Result<()> {
    let content = values.join(" ");
    write_file(filename, &content)
}
