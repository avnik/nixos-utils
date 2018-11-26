extern crate clap;
extern crate pgs_files;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate nixos_utils;

use clap::{Arg, App};
use serde::Serialize;
use std::path::{PathBuf, Path};
use std::fs;
use std::fs::File;
use std::io;
use pgs_files::{passwd,group,shadow};
use nixos_utils::*;

const STATE_DIR: &str = "/var/lib/nixos";
const UID_MAP_FILE: &str = "uid-map";
const GID_MAP_FILE: &str = "gid-map";
const DECLARATIVE_USERS: &str = "declarative-users";
const DECLARATIVE_GROUPS: &str = "declarative-groups";

// state, which need to be persisted
struct UsersGroups {
    passwd: Vec<passwd::PasswdEntry>,
    shadow: Vec<shadow::ShadowEntry>,
    group: Vec<group::GroupEntry>,
    uid_map: serde_json::Value,
    gid_map: serde_json::Value,
    declarative_users: serde_json::Value,
    declarative_groups: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpecUsers {
    name: String,
    uid: i32,
    gorup: String,
    description: String,
    home: String,
    createHome: String,
    is_system_dir: bool,
    password: String,
    password_file: String,
    hashed_password: String,
    initial_password: String,
    initial_hashed_password: String
}

#[derive(Deserialize)]
struct SpecGroups {
    name: String,
    gid: i32,
    members: Vec<i32>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Spec {
    users: SpecUsers,
    groups: SpecGroups,
    mutable_users: bool
}

fn push_int_or_empty(s: &mut String, i: i64) {
    if i >= 0 {
        s.push_str(&i.to_string())
    };
    s.push(':')
}

// "fork" of ToString (FIXME: remove when `impl ToString for {shadow,passwd,group}` hit uptream)
trait ToStringXXX {
    fn to_string_xxx(&self) -> String;
}

// FIXME: promote to pgs_files
impl ToStringXXX for shadow::ShadowEntry {
    fn to_string_xxx(&self) -> String {
        let mut accum = self.name.to_string() + ":" + &self.passwd + ":";
        push_int_or_empty(&mut accum, self.last_change);
        push_int_or_empty(&mut accum, self.min);
        push_int_or_empty(&mut accum, self.max);
        push_int_or_empty(&mut accum, self.warning);
        push_int_or_empty(&mut accum, self.inactivity);
        push_int_or_empty(&mut accum, self.expires);
        if self.flag > 0 { 
            accum.push_str(&self.flag.to_string())
        };
        accum 
    }
}

// FIXME: promote to pgs_files
impl ToStringXXX for passwd::PasswdEntry {
    fn to_string_xxx(&self) -> String {
        let uid = self.uid.to_string();
        let gid = self.gid.to_string();
        self.name.to_string() + ":" + &self.passwd + ":" + &uid + ":" + &gid + ":" + &self.gecos + ":" + &self.dir + ":" + &self.shell
    }
}

// FIXME: promote to pgs_files
impl ToStringXXX for group::GroupEntry {
    fn to_string_xxx(&self) -> String {
        let members_as_str: Vec<String> = self.members.iter().map(|x| x.to_string()).collect();
        let members = members_as_str.join(",");
        let gid = self.gid.to_string();
        self.name.to_string() + ":" + &self.passwd + ":" + &gid + ":" + &members
    }
}

impl UsersGroups {
    pub fn save(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path.join(STATE_DIR))?;
        fs::create_dir_all(path.join("/etc"))?; // can we haven't /etc (unless we in tests)
        write_json(&path.join(STATE_DIR).join(UID_MAP_FILE), &self.uid_map)?;
        write_json(&path.join(STATE_DIR).join(GID_MAP_FILE), &self.gid_map)?;
        write_json(&path.join(STATE_DIR).join(DECLARATIVE_USERS), &self.declarative_users)?;
        write_json(&path.join(STATE_DIR).join(DECLARATIVE_GROUPS), &self.declarative_groups)?;

        let new_group_strs: Vec<String> = self.group.iter().map(|x| x.to_string_xxx()).collect();
        let new_group = new_group_strs.join("\n");
        write_file(&path.join("/etc/group"), &new_group)?;

        let new_passwd_strs: Vec<String> = self.passwd.iter().map(|x| x.to_string_xxx()).collect();
        let new_passwd = new_passwd_strs.join("\n");
        write_file(&path.join("/etc/passwd"), &new_passwd)?;

        let new_shadow_strs: Vec<String> = self.passwd.iter().map(|x| x.to_string_xxx()).collect();
        let new_shadow = new_shadow_strs.join("\n");
        write_file(&path.join("/etc/shadow"), &new_shadow)?;
        Ok(()) 
    }

    pub fn from_path(path: &Path) -> UsersGroups {
        UsersGroups{
            passwd: passwd::get_all_entries_from_path(&path.join("/etc/passwd")),
            group: group::get_all_entries_from_path(&path.join("/etc/group")),
            shadow: shadow::get_all_entries_from_path(&path.join("/etc/shadow")),
            uid_map: read_json_or_empty(&path.join(STATE_DIR).join(UID_MAP_FILE)),            
            gid_map: read_json_or_empty(&path.join(STATE_DIR).join(GID_MAP_FILE)),
            declarative_users: read_json_or_empty(&path.join(STATE_DIR).join(DECLARATIVE_USERS)),
            declarative_groups: read_json_or_empty(&path.join(STATE_DIR).join(DECLARATIVE_GROUPS)),
        }
    }
}

fn main() {
    let matches = App::new("update-users-groups")
        .version("0.1.0")
        .args(&[
            Arg::with_name("root")
             .short("r")
             .long("root")
             .takes_value(true),
            Arg::with_name("INPUT")
             .help("SPEC file to use")
             .index(1)
             .required(true)
        ])
        .get_matches();

    let root = matches.value_of("root").unwrap_or("/");
    let input = matches.value_of("INPUT").unwrap().to_owned();
    println!("process input file {}", input);
    let spec_json = File::open(Path::new(&input)).unwrap();
    let spec: Spec = serde_json::from_reader(spec_json).unwrap();
}
