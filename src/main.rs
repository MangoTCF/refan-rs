use std::{
    io::{Read, Write},
    time::Duration,
};

use anyhow::Context;
use clap::Parser;
use log::info;
use simplelog::{TermLogger, TerminalMode};
use structs::{Args, Config, TPoint};

mod structs;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn map<T: num_traits::Num + Copy>(x: T, in_min: T, in_max: T, out_min: T, out_max: T) -> T {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}

fn main() {
    let args = Args::parse();
    TermLogger::new(
        args.verbosity.to_level_filter(),
        simplelog::Config::default(),
        TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    );
    info!(target: "main", "ReFan v{} online!", VERSION);
    let mut cfg: Config =
        toml::from_str(std::fs::read_to_string(args.config_path).unwrap().as_str()).unwrap();
    for fan in &mut cfg.fans {
        fan.pwm_mode_path
            .write(b"1")
            .context(format!("Trying to set mode for fan {}", fan.name))
            .unwrap();
        log::info!("{} online!", fan.name);
    }
    loop {
        std::thread::sleep(Duration::from_secs_f32(cfg.dt));
        for fan in &mut cfg.fans {
            //read and parse the temperature
            let mut buf = String::default();
            fan.temp_sensor_path
                .read_to_string(&mut buf)
                .context(format!("Reading temperature for fan {}", fan.name))
                .unwrap();
            let t = buf
                .parse::<f32>()
                .context(format!("Parsing temperature for fan {}", fan.name))
                .unwrap()
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
            .clamp(fan.pwm_min as f32, fan.pwm_max as f32) as i16;
            if fan.stopped && pwm > fan.pwm_start {
                fan.stopped = false;
            }
            if (!fan.stopped) && pwm < fan.pwm_stop {
                fan.stopped = true;
            }
            if !fan.stopped {
                fan.pwm_write_path
                    .write(pwm.to_string().as_bytes())
                    .context(format!("Writing PWM value {} for fan {}", pwm, fan.name))
                    .unwrap();
            }
        }
    }
}
