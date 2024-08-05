use std::fs::{File, OpenOptions};

use clap::Parser;
use glob::glob;
use log::Level;
use serde::{Deserialize, Deserializer};

fn deserialize_path_to_file_ro<'de, D>(deserializer: D) -> Result<File, D::Error>
where
    D: Deserializer<'de>,
{
    let path = String::deserialize(deserializer).unwrap();

    if path.contains("*") {
        let path = glob(&path)
            .expect("Invalid pattern given")
            .next()
            .unwrap()
            .unwrap();
        return Ok(OpenOptions::new()
            .write(false)
            .read(true)
            .create(false)
            .open(path)
            .unwrap());
    }

    return Ok(OpenOptions::new()
        .write(false)
        .read(true)
        .create(false)
        .open(path)
        .unwrap());
}
fn deserialize_path_to_file_rw<'de, D>(deserializer: D) -> Result<File, D::Error>
where
    D: Deserializer<'de>,
{
    let path = String::deserialize(deserializer).unwrap();

    if path.contains("*") {
        let path = glob(&path)
            .expect("Invalid pattern given")
            .next()
            .unwrap()
            .unwrap();
        return Ok(OpenOptions::new()
            .write(true)
            .read(true)
            .create(false)
            .open(path)
            .unwrap());
    }

    return Ok(OpenOptions::new()
        .write(true)
        .read(true)
        .create(false)
        .open(path)
        .unwrap());
}

//A customizable fan control program
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    //Path to refan.toml
    #[arg(short, long, default_value = "/etc/refan.toml")]
    pub config_path: String,

    //Verbosity
    #[arg(short, long, default_value = "INFO")]
    pub verbosity: Level,

    //Override time between updates
    #[arg(short = 't', long, default_value = "-1")]
    pub dt_override: f32,
}

#[derive(Deserialize)]
pub struct Config {
    pub dt: f32,
    pub fans: Vec<Fan>,
}

#[derive(Deserialize)]
pub struct Fan {
    #[serde(
        deserialize_with = "deserialize_path_to_file_ro",
        rename = "temp_sensor_path"
    )]
    pub temp_sensor: File,
    #[serde(
        deserialize_with = "deserialize_path_to_file_rw",
        rename = "pwm_write_path"
    )]
    pub pwm_write: File,
    #[serde(
        deserialize_with = "deserialize_path_to_file_rw",
        rename = "pwm_mode_path"
    )]
    pub pwm_mode: File,
    pub curve: Vec<TPoint>,
    pub pwm_start: i16,
    pub pwm_stop: i16,
    #[serde(skip_deserializing)]
    pub stopped: bool,
    pub pwm_min: i16,
    pub pwm_max: i16,
    pub name: String,
}

#[derive(Deserialize, Clone, Copy, Default, Debug)]
pub struct TPoint {
    pub t: f32,
    pub pwm: i16,
}
