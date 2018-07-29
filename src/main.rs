extern crate argparse;
extern crate gl;
extern crate glutin;
extern crate rustfft;
extern crate sphinxad_sys;
extern crate sysfs_gpio;

use argparse::{ArgumentParser, Store, StoreTrue};
use screen::ScreenType;
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
    let mut raw = false;
    let mut selected_visualizer = "".to_string();
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("LED Music Visualizer");
        parser.refer(&mut raw)
              .add_option(&["-r", "--raw"], StoreTrue,
                          "Display raw texture before mapping to LEDs.");
        parser.refer(&mut selected_visualizer)
              .add_option(&["-v", "--visualizer"], Store,
                          "Which visualizer to use.");
        parser.parse_args_or_exit();
    }

    let (audio_tx, audio_rx) = mpsc::sync_channel::<audio::AudioFrame>(1);

    thread::spawn(move || {
        listen::visualize_microphone(audio_tx);
    });

    let screen_type;
    if raw {
        screen_type = ScreenType::Raw;
    } else {
        screen_type = ScreenType::LedDiskEmulator;
    }

    let visualizer = visualizer::Visualizer::new(selected_visualizer);
    let screen = screen::create_screen(screen_type);
    gfx::run(visualizer, screen, audio_rx);
}
