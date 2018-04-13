extern crate argparse;
extern crate ears;
extern crate gl;
extern crate glutin;
extern crate lewton;
extern crate rustfft;
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
mod music;
mod screen;
mod visualizer;

fn main() {
    let mut ogg_file_name = "".to_string();
    let mut raw = false;
    let mut use_sound = false;
    {
        let mut parser = ArgumentParser::new();
        parser.set_description("LED Music Visualizer");
        parser.refer(&mut ogg_file_name)
              .add_argument("ogg_file_name", Store,
                            "Name of ogg file in music/ folder to visualize.")
              .required();
        parser.refer(&mut raw)
              .add_option(&["-r", "--raw"], StoreTrue,
                          "Display raw texture before mapping to LEDs.");
        parser.refer(&mut use_sound)
              .add_option(&["-s", "--sound"], StoreTrue,
                          "Play the ogg file to the speakers.");
        parser.parse_args_or_exit();
    }

    let (audio_tx, audio_rx) = mpsc::channel::<audio::AudioFrame>();

    thread::spawn(move || {
        listen::visualize_ogg(ogg_file_name, use_sound, audio_tx);
    });

    let mut screen_type = ScreenType::LedDiskEmulator;
    if raw {
        screen_type = ScreenType::Raw;
    }

    let visualizer = visualizer::Visualizer::new();
    let screen = screen::create_screen(screen_type);
    gfx::run(visualizer, screen, audio_rx);
}
