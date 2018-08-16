use blinkt::Blinkt;

use gfx;
use led_mapper;
use screen;

pub struct HardwareScreen {
    mapper: led_mapper::LedDiskMapper,
    blinkt: blinkt::Blinkt,
}

impl HardwareScreen {
    pub fn new() -> HardwareScreen {
        let mut blinkt = Blinkt::with_spi(16_000_000, 144).unwrap();
        blinkt.set_all_pixels_brightness(10.0);

        HardwareScreen {
            mapper: led_mapper::LedDiskMapper::new(),
            blinkt,
        }
    }
}

impl screen::Screen for HardwareScreen {
    fn setup(&mut self, _gl: &gfx::gl::Gl) {}

    fn render_from_texture(&self, gl: &gfx::gl::Gl, texture: u32) {
        let pixel_colors = self.mapper.map_from_texture(gl, texture);

        for (i, (r, g, b)) in pixel_colors.iter().enumerate() {
            self.blinkt.set_pixel(i, *r, *g, *b);
        }

        self.blinkt.show().unwrap();
    }

    fn uses_window(&self) -> bool {
        false
    }
}
