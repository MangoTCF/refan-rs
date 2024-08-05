use std::{
    io::{Read, Seek, Write},
    time::Duration,
};

use anyhow::Context;
use clap::Parser;
use log::{debug, info};
use simplelog::{trace, TermLogger, TerminalMode};
use structs::{Args, Config, TPoint};

mod structs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn map<T: num_traits::Num + Copy>(x: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}

fn main() {
    let args = Args::parse();
    TermLogger::init(
        args.verbosity.to_level_filter(),
        simplelog::Config::default(),
        TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    ).unwrap();
    info!(target: "main", "ReFan v{} online!", VERSION);
    let mut cfg: Config =
        toml::from_str(std::fs::read_to_string(args.config_path).context("Reading the configuration file").unwrap().as_str()).unwrap();
    for fan in &mut cfg.fans {
        fan.pwm_mode
            .write(b"1")
            .context(format!("Trying to set mode for fan {}", fan.name))
            .unwrap();
        log::info!("{} online!", fan.name);
    }
    if args.dt_override != -1.0 {
        cfg.dt = args.dt_override;
    }
    loop {
        std::thread::sleep(Duration::from_secs_f32(cfg.dt));
        for fan in &mut cfg.fans {
            fan.temp_sensor.seek(std::io::SeekFrom::Start(0)).unwrap();
            //read and parse the temperature
            let mut buf = [0;32];
            let len = fan.temp_sensor
                .read(&mut buf)
                .context(format!("Reading temperature for fan {}", fan.name))
                .unwrap();
            let t = core::str::from_utf8(&buf[0..len-1]).unwrap()
                .parse::<i32>()
                .context(format!("Parsing temperature for fan {} from {} of len {}", fan.name, core::str::from_utf8(&buf).unwrap(), len))
                .unwrap() as f32
                / 1000f32;

            let mut tpoint_l: TPoint = TPoint::default();
            let mut tpoint_h: TPoint = TPoint::default();
            //find the corresponding TPoint.
            for (i, point) in (&fan.curve).into_iter().enumerate() {
                //search for the TPoint
                if point.t > t {
                    //if the temperature is below the first TPoint, use the first to second for interp
                    if i == 0 {
                        tpoint_l = *point;
                        tpoint_h = fan.curve[1];
                        break;
                    }
                    tpoint_l = *point;
                    tpoint_h = fan.curve[i - 1];
                    break;
                }
                //if the temperature is above the last TPoint, use the second last to last for interp.
                if i == (fan.curve.len() - 1) {
                    tpoint_l = fan.curve[fan.curve.len() - 2];
                    tpoint_h = *point;
                    break;
                }
            }
            let pwm = map(
                t,
                tpoint_l.t,
                tpoint_h.t,
                tpoint_l.pwm as f32,
                tpoint_h.pwm as f32,
            )
            .clamp(fan.pwm_min as f32, fan.pwm_max as f32);
            if fan.stopped && (pwm as i16) > fan.pwm_start {
                fan.stopped = false;
            }
            if (!fan.stopped) && (pwm as i16) < fan.pwm_stop {
                fan.stopped = true;
            }
            debug!(target: fan.name.as_str(), "Stopped: {}, pwm: {}, temp: {}, lerp between {:?} and {:?}", fan.stopped, pwm, t, tpoint_l, tpoint_h);
            if !fan.stopped {
                fan.pwm_write
                    .write((pwm as i16).to_string().as_bytes())
                    .context(format!("Writing PWM value {} for fan {}", pwm, fan.name))
                    .unwrap();
            }
        }
    }
}
