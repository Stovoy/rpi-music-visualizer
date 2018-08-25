#![feature(extern_prelude)]

extern crate argparse;

#[cfg(feature="hardware")]
extern crate blinkt;

extern crate gl;
extern crate glutin;
extern crate rand;
extern crate rustfft;
extern crate sphinxad_sys;
extern crate sysfs_gpio;

use argparse::{ArgumentParser, Store, StoreTrue};
use std::sync::mpsc;
use std::thread;

#[macro_use]
mod gfx;

mod audio;
mod led_mapper;
mod listen;
mod rpio;
mod screen;
mod visualizer;

fn main() {
    let mut selected_visualizer = "".to_string();
    let mut selected_screen = "".to_string();
    let mut size = 128;
    let mut samples_per_second = 24000;
    let mut window_sample_size = 1024;
    let mut amplitude_scalar = 16.0;
    let mut use_fake_audio = false;
    let mut pin = 35;
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("LED Music Visualizer");
        parser.refer(&mut selected_visualizer)
              .add_option(&["-v", "--visualizer"], Store,
                          "Which visualizer to use.");
        parser.refer(&mut selected_screen)
              .add_option(&["-s", "--screen"], Store,
                          "Which screen to use.");
        parser.refer(&mut size)
              .add_option(&["--size"], Store,
                          "Window size.");
        parser.refer(&mut samples_per_second)
              .add_option(&["--samples_per_second"], Store,
                          "Number of samples per second to record from the micrphone.");
        parser.refer(&mut window_sample_size)
              .add_option(&["--window_sample_size"], Store,
                          "Number of samples to process at a time.");
        parser.refer(&mut amplitude_scalar)
              .add_option(&["--amplitude_scalar"], Store,
                          "Multiplier for audio ampltitude.");
        parser.refer(&mut pin)
              .add_option(&["--pin"], Store,
                          "Multiplier for audio ampltitude.");
        parser.refer(&mut use_fake_audio)
              .add_option(&["--fake"], StoreTrue,
                          "Use fake audio.");
        parser.parse_args_or_exit();
    }

    rpio::read_button(pin);

    let (audio_tx, audio_rx) = mpsc::sync_channel::<audio::AudioFrame>(1);

	if !use_fake_audio {
		thread::spawn(move || {
			listen::visualize_microphone(audio_tx, samples_per_second, window_sample_size, amplitude_scalar);
		});
	} else {
		thread::spawn(move || {
			listen::visualize_fake(audio_tx);
		});
	}

    let visualizer = visualizer::Visualizer::new(selected_visualizer);
    let screen = screen::create_screen(selected_screen);
    gfx::run(visualizer, screen, audio_rx, size);
}
