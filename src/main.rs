extern crate argparse;
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
    let mut use_fake_audio = false;
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("LED Music Visualizer");
        parser.refer(&mut selected_visualizer)
              .add_option(&["-v", "--visualizer"], Store,
                          "Which visualizer to use.");
        parser.refer(&mut selected_screen)
              .add_option(&["-s", "--screen"], Store,
                          "Which screen to use.");
        parser.refer(&mut use_fake_audio)
              .add_option(&["--fake"], StoreTrue,
                          "Use fake audio.");
        parser.parse_args_or_exit();
    }

    let (audio_tx, audio_rx) = mpsc::sync_channel::<audio::AudioFrame>(1);

	if !use_fake_audio {
		thread::spawn(move || {
			listen::visualize_microphone(audio_tx);
		});
	} else {
		thread::spawn(move || {
			listen::visualize_fake(audio_tx);
		});
	}

    let visualizer = visualizer::Visualizer::new(selected_visualizer);
    let screen = screen::create_screen(selected_screen);
    gfx::run(visualizer, screen, audio_rx);
}
