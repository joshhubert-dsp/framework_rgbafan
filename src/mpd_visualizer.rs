use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use framework_lib::chromium_ec::commands::RgbS;
use rustfft::{Fft, FftPlanner};
use num_complex::Complex;

use crate::consts::{
    FFT_SIZE,
    N_LEDS,
    FIFO_PATH,
    MPD_QUIET_TIMEOUT
};

use crate::animations::Animation;


// -----------------------------------------------------------------------------
// Visualizer Implementation
// -----------------------------------------------------------------------------

pub struct MpdVisualizer {
    fifo_path: PathBuf,
    file: Option<File>,
    last_audio_time: Instant,

    // Audio Buffering & FFT State
    sample_buffer: Vec<f32>,
    fft: Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    fft_scratch: Vec<Complex<f32>>,

    // Fallback State
    fallback_gradient: Vec<RgbS>,
    fallback_angle: f32,
    fallback_period: u16,
}

impl MpdVisualizer {
    pub fn new(fallback_gradient: Vec<RgbS>, fallback_period: u16) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        Self {
            fifo_path: PathBuf::from(FIFO_PATH),
            file: None,
            last_audio_time: Instant::now(),

            // Buffer capacity: roughly enough for a few ticks of overflow
            sample_buffer: Vec::with_capacity(FFT_SIZE * 2),
            fft,
            fft_scratch: vec![Complex::new(0.0, 0.0); FFT_SIZE], // Pre-allocate scratch space
            fft_buffer: vec![Complex::new(0.0, 0.0); FFT_SIZE],
            fallback_gradient: fallback_gradient,
            fallback_angle: 0.0,
            fallback_period: fallback_period,
        }
    }

    pub fn tick(&mut self, leds: &mut [RgbS; N_LEDS]) {
        // 1. Open FIFO if needed
        if self.file.is_none() {
            let file_result = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&self.fifo_path);

            if let Ok(f) = file_result {
                self.file = Some(f);
            }
        }

        let mut audio_processed = false;

        // 2. Read and Buffer Audio
        if let Some(ref mut file) = self.file {
            let mut raw_bytes = [0u8; 2048]; // Read ~2kb chunks
            match file.read(&mut raw_bytes) {
                Ok(bytes_read) if bytes_read > 0 => {
                    self.last_audio_time = Instant::now();
                    self.process_incoming_bytes(&raw_bytes[..bytes_read], leds);
                    audio_processed = true;
                }
                Ok(_) => {} // Empty read
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(_) => self.file = None, // Reset on error
            }
        }

        // 3. Fallback Logic (Silence detection)
        if !audio_processed {
            let silence_duration = Instant::now().duration_since(self.last_audio_time);

            // If we have data buffered but not enough for a full FFT yet,
            // we treat it as silence if no NEW data came in for 1 second.
            if silence_duration > Duration::from_secs(MPD_QUIET_TIMEOUT as u64) {
                // Clear buffer so old audio doesn't flash when music resumes
                self.sample_buffer.clear();

                Animation::step_smoothspin(
                    leds,
                    &mut self.fallback_angle,
                    &self.fallback_gradient,
                    self.fallback_period,
                );
            } else {
                // Decay logic: If music is playing but we are between FFT frames,
                // we dampen the previous frame slightly to make it look organic.
                for led in leds.iter_mut() {
                    led.r = (led.r as f32 * 0.85) as u8;
                    led.g = (led.g as f32 * 0.85) as u8;
                    led.b = (led.b as f32 * 0.85) as u8;
                }
            }
        }
    }

    // Converts raw PCM bytes to f32 and pushes to buffer.
    // Triggers FFT when buffer is full.
    fn process_incoming_bytes(&mut self, data: &[u8], leds: &mut [RgbS; N_LEDS]) {
        // 16-bit signed stereo = 4 bytes per frame (L+R)
        // We mix stereo down to mono for visualization
        for chunk in data.chunks_exact(4) {
            let l_sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32;
            let r_sample = i16::from_le_bytes([chunk[2], chunk[3]]) as f32;

            // Average channels and normalize to -1.0 .. 1.0
            let mono = (l_sample + r_sample) / 2.0 / 32768.0;
            self.sample_buffer.push(mono);
        }

        // If we have enough samples, run the FFT
        if self.sample_buffer.len() >= FFT_SIZE {
            for (i, sample) in self.sample_buffer.drain(0..FFT_SIZE).enumerate() {
                self.fft_buffer[i] = Complex::new(sample, 0.0);
            }

            self.perform_fft(leds);
        }
    }

    fn perform_fft(&mut self, leds: &mut [RgbS]) {
        // Run FFT
        self.fft.process_with_scratch(&mut self.fft_buffer, &mut self.fft_scratch);

        // Map Spectrum to 8 LEDs
        // We only care about the first half of the buffer (Nyquist frequency)
        let output_len = self.fft_buffer.len() / 2;
        let spectrum = &self.fft_buffer[0..output_len];

        // Define logarithmic bands for 8 sections (approximate indices for 1024 FFT @ 44.1k)
        // Hz: [0-60], [60-150], [150-400], [400-1k], [1k-2.5k], [2.5k-6k], [6k-12k], [12k+]
        // Bin width is ~43Hz
        let bands = [(1, 2), (2, 4), (4, 10), (10, 25), (25, 60), (60, 150), (150, 300), (300, 511)];

        for (i, (start, end)) in bands.iter().enumerate() {
            if i >= bands.len() { break; }

            // Calculate magnitude (volume) for this frequency band
            let mut sum_mag = 0.0;
            let count = (end - start) as f32;
            for bin_idx in *start..*end {
                if bin_idx < spectrum.len() {
                    sum_mag += spectrum[bin_idx].norm_sqr();
                }
            }
            let avg_mag = sum_mag / count;

            // Apply slight log scaling to amplitude to match human hearing
            let amplitude = (avg_mag * 0.1).ln_1p().min(1.0).max(0.0);

            // Get the base color for this frequency
            let base_color = self.get_freq_color(i);

            // Apply brightness based on amplitude
            leds[i] = RgbS {
                r: (base_color.r as f32 * amplitude) as u8,
                g: (base_color.g as f32 * amplitude) as u8,
                b: (base_color.b as f32 * amplitude) as u8,
            };
        }
    }

    // Maps LED index (Frequency Band) to Hue
    // Bass (Low Index) -> Cool (Purple/Blue/Green)
    // Treble (High Index) -> Warm (Yellow/Red/Pink)
    fn get_freq_color(&self, index: usize) -> RgbS {
        match index {
            0 => RgbS { r: 255,  g: 0,   b: 255 }, // Deep Bass: Indigo/Purple
            1 => RgbS { r: 0,   g: 0,   b: 255 }, // Bass: Blue
            2 => RgbS { r: 0,   g: 100, b: 255 }, // Low Mid: Cyan
            3 => RgbS { r: 0,   g: 255, b: 0 },   // Mid: Spring Green
            4 => RgbS { r: 255, g: 255, b: 0   }, // High Mid: Yellow
            5 => RgbS { r: 255, g: 50, b: 0   }, // Presence: Orange
            6 => RgbS { r: 255, g: 0,   b: 0   }, // Treble: Red
            7 => RgbS { r: 255, g: 80, b: 80 },   // Air: Pink
            _ => RgbS { r: 255, g: 255, b: 255 },
        }
    }
}
