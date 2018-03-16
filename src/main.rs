extern crate gl;
extern crate glutin;
extern crate pad;
extern crate rustfft;
extern crate sysfs_gpio;
extern crate simplemad;

mod audio;
mod gfx;
mod rpio;
mod music;

use std::env;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use std::cmp;

use glutin::GlContext;
use pad::PadStr;

fn main() {
    println!("=== Starting Music Visualizer ===");

    let mut args = env::args();
    if args.len() < 2 {
        test_graphics();
    } else {
        let mp3_file_name = args.nth(1).unwrap();
        let wait = args.len() >= 1 && args.nth(0).unwrap() == "wait";
        visualize_mp3(mp3_file_name, wait);
    }
}

fn test_graphics() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Music Visualizer")
        .with_dimensions(1024, 768);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    let _ = unsafe { gl_window.make_current() };

    let gl = gfx::load(&gl_window);

    events_loop.run_forever(|event| {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => return glutin::ControlFlow::Break,
                glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                _ => (),
            },
            _ => ()
        }

        gl.draw_frame([0.0, 1.0, 0.0, 1.0]);
        let _ = gl_window.swap_buffers();
        glutin::ControlFlow::Continue
    });
}

fn visualize_mp3(mp3_file_name: String, wait: bool) {
    println!("Parsing {}...", mp3_file_name.clone());
    let (samples, duration_seconds) = music::read_mp3_file(mp3_file_name.clone());

    if wait {
        println!("Starting playback of {} in 3...", mp3_file_name);
        sleep(Duration::from_millis(1000));
        println!("2...");
        sleep(Duration::from_millis(1000));
        println!("1...");
        sleep(Duration::from_millis(1000));
    }

    let samples_per_sec = (samples.len() as f32 / duration_seconds).ceil();

    let window_size_sec = 0.001;
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

        println!("LOW {} | MID {} | HIGH {} | {}.{}s",
            (0..((low_power * 10.0) as usize)).map(|_| '#').collect::<String>().with_exact_width(10),
            (0..((mid_power * 10.0) as usize)).map(|_| '#').collect::<String>().with_exact_width(10),
            (0..((high_power * 10.0) as usize)).map(|_| '#').collect::<String>().with_exact_width(10),
            current_time.as_secs(), current_time.subsec_nanos());

        sleep(window_duration);
        window_start = window_end + 1;

        let elapsed = start.elapsed();
        current_time += elapsed;
        time_drift_offset_samples = (samples_per_sec * ((elapsed - window_duration).subsec_nanos() as f32 / 1000000000.0)) as usize;
    }
}
