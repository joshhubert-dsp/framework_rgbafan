use std::env;
use std::num::ParseIntError;
use std::thread;
use std::time::Duration;

use framework_lib::chromium_ec::commands::RgbS;
use framework_lib::chromium_ec::{CrosEc, EcResult};

mod mpd_visualizer;
mod animations;
mod consts;

use consts::{OFF, N_LEDS, TICK_TIME_MS};
use animations::Animation;


#[derive(Debug)]
enum HexParseError {
    WrongLength,
    InvalidDigit,
}

impl From<ParseIntError> for HexParseError {
    fn from(_: ParseIntError) -> Self {
        HexParseError::InvalidDigit
    }
}


fn parse_hex(s: &str) -> Result<RgbS, HexParseError> {
    if s.len() != 6 {
        return Err(HexParseError::WrongLength);
    }

    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;

    Ok(RgbS { r, g, b })
}

fn args_to_rgbs(args: Vec<String>) -> Result<Vec<RgbS>, HexParseError> {
    args.into_iter()
        .skip(1)
        .map(|s| parse_hex(&s))
        .collect()
}

fn main() -> EcResult<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let animation_modestr = args.get(0)
        .expect("Expected an animation argument, and several color arguments.")
        .as_str()
        .to_owned();

    let colors: Vec<RgbS> = args_to_rgbs(args).unwrap_or_else(|e| panic!("Failed to parse color argument: {e:?}"));

    let mut animation = Animation::from_cli(&animation_modestr, colors);

    let ec = CrosEc::new();
    
    let mut leds: [RgbS; N_LEDS] = [OFF; N_LEDS];
        
    loop {
        animation.step(&mut leds);

        if let Err(e) = ec.rgbkbd_set_color(0, leds.to_vec()) {
            eprintln!("Error setting lights: {:?}", e);
        }
        thread::sleep(Duration::from_millis(TICK_TIME_MS.into()));
    }
        
}
