use blinkt::Blinkt;

use gfx;
use led_mapper;
use screen;

pub struct HardwareScreen {
    mapper: led_mapper::LedDiskMapper,
}

impl HardwareScreen {
    pub fn new() -> HardwareScreen {
        HardwareScreen {
            mapper: led_mapper::LedDiskMapper::new(),
        }
    }
}

impl screen::Screen for HardwareScreen {
    fn setup(&mut self, _gl: &gfx::gl::Gl) {}

    fn render_from_texture(&self, gl: &gfx::gl::Gl, texture: u32) {
        let pixel_colors = self.mapper.map_from_texture(gl, texture);

        let mut blinkt = Blinkt::with_spi(1_000_000, 255).unwrap();
        blinkt.set_clear_on_drop(false);
        blinkt.set_all_pixels_brightness(5.0);

        for (i, (r, g, b)) in pixel_colors.iter().enumerate() {
            blinkt.set_pixel(i, *r, *g, *b);
        }

        blinkt.show().unwrap();
    }

    fn uses_window(&self) -> bool {
        false
    }
}
