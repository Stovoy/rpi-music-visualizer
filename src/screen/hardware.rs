use blinkt::Blinkt;
use gfx;
use led_mapper;
use screen;
use std::sync::mpsc;
use std::thread;

pub struct HardwareScreen {
    mapper: led_mapper::LedDiskMapper,
    pixels_tx: mpsc::SyncSender<[(u8, u8, u8); led_mapper::LedDiskMapper::NUM_PIXELS]>,
}

impl HardwareScreen {
    pub fn new() -> HardwareScreen {
        let (pixels_tx, pixels_rx) = mpsc::sync_channel::<[(u8, u8, u8); led_mapper::LedDiskMapper::NUM_PIXELS]>(1);

        thread::spawn(move || {
            blinkt_pipeline(pixels_rx);
        });

        HardwareScreen {
            mapper: led_mapper::LedDiskMapper::new(),
            pixels_tx,
        }
    }
}

impl screen::Screen for HardwareScreen {
    fn setup(&mut self, _gl: &gfx::gl::Gl) {}

    fn render_from_texture(&mut self, gl: &gfx::gl::Gl, texture: u32) {
        let pixel_colors = self.mapper.map_from_texture(gl, texture);

        self.pixels_tx.send(pixel_colors).unwrap();
    }

    fn uses_window(&self) -> bool {
        false
    }
}

fn blinkt_pipeline(pixels_rx: mpsc::Receiver<[(u8, u8, u8); led_mapper::LedDiskMapper::NUM_PIXELS]>) {
    let mut blinkt = Blinkt::with_spi(1_000_000, 255).unwrap();
    blinkt.set_all_pixels_brightness(0.08);

    loop {
        let pixel_colors = match pixels_rx.recv() {
            Ok(x) => x,
            Err(_) => continue,
        };

        for (i, (r, g, b)) in pixel_colors.iter().enumerate() {
            blinkt.set_pixel(i, *r, *g, *b);
        }

        blinkt.show().unwrap();
    }
}