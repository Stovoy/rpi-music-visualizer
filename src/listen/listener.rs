use audio;
use sphinxad_sys::{ad_open_sps, ad_read, ad_start_rec};
use std::cmp;
use std::sync::mpsc;


pub fn visualize_microphone(tx: mpsc::SyncSender<audio::AudioFrame>) {
    let samples_per_second = 22050;
    let ad = unsafe { ad_open_sps(samples_per_second) };
    let rec_successful = unsafe { ad_start_rec(ad) } == 0;
    if !rec_successful {
        panic!("Could not start recording microphone.");
    }

    loop {
        let mut buffer = vec![0; 22050];
        let raw_buffer = buffer.as_mut_ptr();
        let sample_count = unsafe { ad_read(ad, raw_buffer, 22050) };
        let duration_seconds = sample_count as f32 / samples_per_second as f32;

        let mut samples = Vec::new();
        for i in 0..sample_count as usize {
            let sample_value = buffer[i] as f32 / i16::max_value() as f32;
            let clamped_value = f32::max(-1.0, f32::min(1.0, sample_value));
            samples.push(clamped_value);
        }

        visualize_samples(samples, sample_count as usize, duration_seconds, &tx);
    }
}

fn visualize_samples(samples: Vec<f32>, length: usize, duration_seconds: f32, tx: &mpsc::SyncSender<audio::AudioFrame>) {
    let samples_per_sec = (length as f32 / duration_seconds).ceil();

    let window_size_sec = 0.05;
    let window_size_samples = (samples_per_sec * window_size_sec).ceil();

    let mut window_start: usize = 0;
    while window_start < length {
        let window_end = cmp::min(
            window_start + window_size_samples as usize,
            length - 1,
        );

        let frequency_bins = audio::frequency_bins(
            samples_per_sec as u32,
            (window_end - window_start) as u32);

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

        window_start = window_end + 1;
    }
}
