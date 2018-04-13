use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::cmp;
use std::thread::sleep;

use ears::{AudioController, Sound};

use audio;
use music;

pub fn visualize_ogg(ogg_file_name: String, use_sound: bool, tx: mpsc::Sender<audio::AudioFrame>) {
    println!("Parsing {}...", ogg_file_name.clone());

    let ogg_file_path = format!("music/{}", ogg_file_name);
    let (samples, duration_seconds) = music::read_ogg_file(ogg_file_path.clone());

    // Play the song audio.
    println!("Playing {}...", ogg_file_name.clone());
    let mut sound = Sound::new(&ogg_file_path).unwrap();

    if use_sound {
        sound.play();
    }

    let samples_per_sec = (samples.len() as f32 / duration_seconds).ceil();

    let window_size_sec = 0.02;
    let window_duration = Duration::from_millis((window_size_sec * 1000.0) as u64);
    let window_size_samples = (samples_per_sec * window_size_sec).ceil();

    let frequency_bins = audio::frequency_bins(samples_per_sec as u32, window_size_samples as u32);

    let mut window_start: usize = 0;
    let mut current_time = Duration::new(0, 0);
    let mut time_drift_offset_samples: usize = 0;
    while window_start < samples.len() {
        let start = Instant::now();

        let window_end = cmp::min(
            window_start + window_size_samples as usize + time_drift_offset_samples,
            samples.len() - 1,
        );

        let fft_output = audio::compute_fft(samples[window_start..window_end].to_vec());
        let amplitudes = audio::to_amplitude(fft_output);

        let low_threshold_hz = 1000.0;
        let mid_threshold_hz = 6000.0;
        let high_threshold_hz = 20000.0;
        let mut low_power = 0.0;
        let mut mid_power = 0.0;
        let mut high_power = 0.0;
        for i in 0..frequency_bins.len() {
            if frequency_bins[i] <= low_threshold_hz {
                low_power += amplitudes[i];
            } else if frequency_bins[i] <= mid_threshold_hz {
                mid_power += amplitudes[i];
            } else if frequency_bins[i] <= high_threshold_hz {
                high_power += amplitudes[i];
            }
        }
        low_power = f32::min(1.0, low_power);
        mid_power = f32::min(1.0, mid_power);
        high_power = f32::min(1.0, high_power);

        let audio_frame = audio::AudioFrame {
            low_power,
            mid_power,
            high_power,
        };
        tx.send(audio_frame).unwrap();

        sleep(window_duration);
        window_start = window_end + 1;

        let elapsed = start.elapsed();
        current_time += elapsed;
        time_drift_offset_samples = (samples_per_sec
            * ((elapsed - window_duration).subsec_nanos() as f32 / 1000000000.0))
            as usize;
    }
}
