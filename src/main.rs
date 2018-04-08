extern crate ears;
extern crate gl;
extern crate glutin;
extern crate lewton;
extern crate rustfft;
extern crate sysfs_gpio;

mod audio;
mod listen;
#[macro_use]
mod gfx;
mod rpio;
mod music;
mod screen;
mod visualizer;

use std::env;
use std::process::exit;
use std::sync::mpsc;
use std::thread;

use screen::ScreenType;

fn main() {
    println!("=== Starting Music Visualizer ===");

    let mut args = env::args();
    if args.len() == 1 {
        println!("First arg must be name of ogg file in the music folder.");
        exit(1);
    }
    let ogg_file_name = args.nth(1).unwrap();

    let (audio_tx, audio_rx) = mpsc::channel::<audio::AudioFrame>();

    thread::spawn(move || {
        listen::visualize_ogg(ogg_file_name, audio_tx);
    });

    let mut screen_type = ScreenType::LedDiskEmulator;
    if args.len() == 1 {
        let screen = args.nth(0).unwrap();
        if screen == "--raw" {
            screen_type = ScreenType::Raw;
        }
    }
    let visualizer = visualizer::Visualizer::new();
    let screen = screen::create_screen(screen_type);
    gfx::run(visualizer, screen, audio_rx);
}
