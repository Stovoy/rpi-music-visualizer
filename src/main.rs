extern crate ears;
extern crate gl;
extern crate glutin;
extern crate lewton;
extern crate rustfft;
extern crate sysfs_gpio;

mod audio;
mod gfx;
mod rpio;
mod music;

use std::cmp;
use std::env;
use std::process::exit;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::thread;

use ears::{Sound, AudioController};
use glutin::GlContext;

fn main() {
    println!("=== Starting Music Visualizer ===");

    let mut args = env::args();
    if args.len() == 1 {
        println!("First arg must be name of ogg file in the music folder.");
        exit(1);
    }
    let ogg_file_name = args.nth(1).unwrap();


    let (tx, rx) = mpsc::channel::<audio::AudioFrame>();

    thread::spawn(move || {
        visualize_ogg(ogg_file_name, tx);
    });
    start_graphics(rx);
}

fn start_graphics(rx: mpsc::Receiver<audio::AudioFrame>) {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Music Visualizer")
        .with_dimensions(800, 800);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();
    unsafe { gl_window.make_current() }.unwrap();

    let mut gl = gfx::load(&gl_window);

    let mut running = true;
    while running {
        let audio_frame = rx.recv().unwrap();

        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event, .. } => match event {
                    glutin::WindowEvent::Closed => running = false,
                    glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                    _ => ()
                },
                _ => ()
            }
        });

        gl.draw_frame(audio_frame);
        gl_window.swap_buffers().unwrap();
    }
}

fn visualize_ogg(ogg_file_name: String, tx: mpsc::Sender<audio::AudioFrame>) {
    println!("Parsing {}...", ogg_file_name.clone());

    let ogg_file_path = format!("music/{}", ogg_file_name);
    let (samples, duration_seconds) = music::read_ogg_file(ogg_file_path.clone());

    // Play the song audio.
    println!("Playing {}...", ogg_file_name.clone());
    let mut sound = Sound::new(&ogg_file_path).unwrap();
    sound.play();

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

        let window_end = cmp::min(window_start + window_size_samples as usize + time_drift_offset_samples, samples.len() - 1);

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
        high_power= f32::min(1.0, high_power);

        let audio_frame = audio::AudioFrame { low_power, mid_power, high_power };
        tx.send(audio_frame).unwrap();

        sleep(window_duration);
        window_start = window_end + 1;

        let elapsed = start.elapsed();
        current_time += elapsed;
        time_drift_offset_samples = (samples_per_sec * ((elapsed - window_duration).subsec_nanos() as f32 / 1000000000.0)) as usize;
    }
}
