use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs::File;
use std::io::prelude::*;

lazy_static! {
    pub static ref CONFIG: config = load("config/kly.yaml");
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct config {
    mqtt: mqtt,
    udp: udp,
    prometheus: prometheus,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct mqtt {
    url: String,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct fwd {
    start: u16,
    size: u16,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct udp {
    fwd: fwd,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct prometheus {
    url: String,
}

pub fn load(file: &str) -> config {
    let mut f = File::open(file).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    let res: Result<config, _> = serde_yaml::from_str(&s);
    res.unwrap()
}
