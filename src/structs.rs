use std::fs::{File, OpenOptions};

use clap::Parser;
use log::Level;
use serde::{Deserialize, Deserializer};

fn deserialize_path_to_file_ro<'de, D>(deserializer: D) -> Result<File, D::Error>
where
    D: Deserializer<'de>,
{
    return Ok(OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open(String::deserialize(deserializer).unwrap())
        .unwrap());
}
fn deserialize_path_to_file_rw<'de, D>(deserializer: D) -> Result<File, D::Error>
where
    D: Deserializer<'de>,
{
    return Ok(OpenOptions::new()
        .write(true)
        .read(true)
        .create(false)
        .open(String::deserialize(deserializer).unwrap())
        .unwrap());
}

//A customizable fan control program
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    //Path to refan.toml
    pub config_path: String,

    //Verbosity
    pub verbosity: Level,
}

#[derive(Deserialize)]
pub struct Config {
    pub dt: f32,
    pub fans: Vec<Fan>,
}

#[derive(Deserialize)]
pub struct Fan {
    #[serde(deserialize_with = "deserialize_path_to_file_ro")]
    pub temp_sensor_path: File,
    #[serde(deserialize_with = "deserialize_path_to_file_rw")]
    pub pwm_write_path: File,
    #[serde(deserialize_with = "deserialize_path_to_file_rw")]
    pub pwm_mode_path: File,
    pub curve: Vec<TPoint>,
    pub pwm_start: i16,
    pub pwm_stop: i16,
    pub stopped: bool,
    pub pwm_min: i16,
    pub pwm_max: i16,
    pub name: String,
}

#[derive(Deserialize, Clone, Copy, Default)]
pub struct TPoint {
    pub t: f32,
    pub pwm: i16,
}
