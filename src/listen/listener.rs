use audio;
use sphinxad_sys::{ad_open_sps, ad_read, ad_start_rec};
use rand::prelude::*;
use std::sync::mpsc;
use std::thread;
use std::time;


pub fn visualize_microphone(tx: mpsc::SyncSender<audio::AudioFrame>) {
    let samples_per_second = 22050;
    let ad = unsafe { ad_open_sps(samples_per_second) };
    let rec_successful = unsafe { ad_start_rec(ad) } == 0;
    if !rec_successful {
        panic!("Could not start recording microphone.");
    }

    let window_sample_size = 1024;
    let mut window: Vec<f32> = Vec::with_capacity(window_sample_size);
    let duration_seconds = window_sample_size as f32 / samples_per_second as f32;

    loop {
        let mut buffer = vec![0; samples_per_second as usize];
        let raw_buffer = buffer.as_mut_ptr();
        let sample_count = unsafe { ad_read(ad, raw_buffer, samples_per_second) };

        for i in 0..sample_count as usize {
            let sample_value = buffer[i] as f32 / i16::max_value() as f32;
            let clamped_value = f32::max(-1.0, f32::min(1.0, sample_value));
            window.push(clamped_value);
        }

        if window.len() < 1024 {
            continue;
        }

        visualize_samples(&window[0..1023].to_vec(), sample_count as usize, duration_seconds, &tx);
        window = window.split_off(1024);
    }
}

pub fn visualize_fake(tx: mpsc::SyncSender<audio::AudioFrame>) {
	let mut rng = thread_rng();

	loop {
		let bpm = 0.0;

		let low_power = rng.gen();
		let mid_power = rng.gen();
		let high_power = rng.gen();

		let mut hundred_hz_buckets = [0.0; 200];
		for i in 0..hundred_hz_buckets.len() {
			hundred_hz_buckets[i] = rng.gen();
		}

		let audio_frame = audio::AudioFrame {
			bpm,

			low_power,
			mid_power,
			high_power,

			hundred_hz_buckets,
		};

		tx.send(audio_frame).unwrap();

		thread::sleep(time::Duration::from_millis(50));
	}
}

fn visualize_samples(samples: &Vec<f32>, length: usize, duration_seconds: f32, tx: &mpsc::SyncSender<audio::AudioFrame>) {
    let samples_per_sec = (samples.len() as f32 / duration_seconds).ceil();
    let frequency_bins = audio::frequency_bins(
        samples_per_sec as u32,
        samples.len() as u32);

    let fft_output = audio::compute_fft(samples.to_vec());
    let amplitudes = audio::to_amplitude(fft_output);

    let low_threshold_hz = 1000.0;
    let mid_threshold_hz = 4000.0;
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

    let mut hundred_hz_buckets = [0.0; 200];
    for i in 0..frequency_bins.len() {
        let hundred_hz_bucket_index = (frequency_bins[i] / 100.0).floor() as usize;
        hundred_hz_buckets[hundred_hz_bucket_index] += amplitudes[i];
    }

    let bpm = 0.0;

    let audio_frame = audio::AudioFrame {
        bpm,

        low_power,
        mid_power,
        high_power,

        hundred_hz_buckets,
    };

    tx.send(audio_frame).unwrap();
}
