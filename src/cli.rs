use crate::consts::DEFAULT_BRIGHTNESS_EFFECT_PERIOD;
use crate::consts::DEFAULT_TICK_TIME_MS;
use clap::{ColorChoice, Parser};

/// Animate your Framework computer RGB fan! Don't forget sudo!
/// Personal favorite: `sudo framework_rgbafan smoothspin 20 -e cwfade -p 50`
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    color = ColorChoice::Auto,
    long_about = None,
    override_usage="sudo framework_rgbafan [OPTIONS] <MODE> [TICK_MS]")
]
pub struct Args {
    /// Available animation modes: static, sequence, random, randominput, quadspin, fullspin, smoothspin, rainbowspin, mpd.
    /// - static: static color(s) across all LEDs, no animation.
    /// - sequence: iterate through colors, one on all LEDs at a time.
    /// - random: random colors on each LED changing every update.
    /// - randominput: random brightnesses selected from passed colors on each LED changing every update, can achieve fireplace flicker vibe.
    /// - quadspin: LEDs divided into 4 quadrants of color cycling discretely.
    /// - fullspin: spinning colors across all LEDs.
    /// - smoothspin: spinning color gradient with interpolation across all LEDs.
    /// - rainbowspin: spinning rainbow across all LEDs.
    /// - mpd: music visualizer mode, see README.
    #[arg(required = true)]
    pub mode: String,

    /// Integer number of milliseconds between updates.
    #[arg(default_value_t = DEFAULT_TICK_TIME_MS)]
    pub tick_ms: u64,

    /// List of 1-8 color hex strings, specified with 6 characters each or a single 0 for LED off.
    /// For static and 'spin' modes, if fewer than 8 colors are specified, they will map linearly across the 8 LEDs.
    /// rainbowspin has preset colors and therefore ignores this list.
    #[arg(
        short,
        long,
        value_name = "str",
        num_args = 1..9,
        default_values_t = ["ff0000".to_string(), "00ff00".to_string(), "0000ff".to_string()])
    ]
    pub colors: Vec<String>,

    /// Available brightness effects: blink, pulse, cwfade, ccwfade, cwccwfade. Effects can be optionally applied to any animation mode.
    #[arg(short, long, value_name = "str")]
    pub effect: Option<String>,

    /// Brightness effect period in units of ticks.
    #[arg(
        short = 'p',
        long = "effect-period",
        value_name = "uint",
        default_value_t = DEFAULT_BRIGHTNESS_EFFECT_PERIOD)
    ]
    pub effect_period: usize,

    /// Flag to make the fan speed control the update time, from 500 ms with fan off to 1 ms with it at 100%.
    #[arg(short, long)]
    pub speed_from_fan: bool,
}
