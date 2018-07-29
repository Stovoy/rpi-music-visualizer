use gfx;
use led_mapper;
use rpio;
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

        let mut led_data: Vec<u8> = Vec::new();

        // Insert 4 0 bytes for the start frame.
        for _ in 0..4 {
            led_data.push(0);
        }

        // Brightness value from 0-31.
        let brightness = 10;

        for (r, g, b) in pixel_colors.iter() {
            let brightness_frame = 0b11100000 + brightness;
            led_data.push(brightness_frame);
            // Order is Blue, Green, Red.
            led_data.push(*b);
            led_data.push(*g);
            led_data.push(*r);
        }

        // Insert 4 1 bytes for the end frame.
        for _ in 0..4 {
            led_data.push(0b11111111);
        }

        // TODO: Find pin for SPI.
        rpio::send_binary(0, led_data);
    }

    fn uses_window(&self) -> bool {
        false
    }
}
