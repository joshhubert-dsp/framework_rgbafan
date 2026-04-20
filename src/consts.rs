use framework_lib::chromium_ec::commands::RgbS;

// Milliseconds per second
pub const UNIT_MS: u16 = 1000;
pub const N_LEDS: usize = 8;
// time the program sleeps for between updates, milliseconds
pub const DEFAULT_TICK_TIME_MS: u16 = 32;
// how often to refresh solid color in case computer enters sleep mode or something
pub const SOLID_TICK_TIME_MS: u16 = 5 * UNIT_MS;
pub const SPIN_PERIOD: u16 = (UNIT_MS / DEFAULT_TICK_TIME_MS) * N_LEDS as u16;

pub const OFF: RgbS = RgbS{ r: 0, g: 0, b: 0 };

// pretty rainbow colors from framework_tool example
pub const RAINBOW: [RgbS; N_LEDS] = [
    RgbS{ r: 0xff, g: 0x00, b: 0x00 },
    RgbS{ r: 0xff, g: 0x80, b: 0x00 },
    RgbS{ r: 0xff, g: 0xff, b: 0x00 },
    RgbS{ r: 0x00, g: 0xff, b: 0x00 },
    RgbS{ r: 0x00, g: 0xff, b: 0xff },
    RgbS{ r: 0x00, g: 0x00, b: 0xff },
    RgbS{ r: 0x80, g: 0x00, b: 0xff },
    RgbS{ r: 0xff, g: 0x00, b: 0xff },
];

// MPD
//pub const SAMPLE_RATE: f32 = 44100.0;
pub const FFT_SIZE: usize = 1024;
pub const FIFO_PATH: &str = "/tmp/rgb.fifo";
pub const MPD_QUIET_TIMEOUT: u8 = 5;
