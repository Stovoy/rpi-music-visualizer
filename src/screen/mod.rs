mod led_disk_emulator;
mod raw;

use gfx;

pub fn create_screen(screen_type: ScreenType) -> Box<Screen> {
    match screen_type {
        ScreenType::Raw => Box::new(raw::RawScreen::new()),
        ScreenType::LedDiskEmulator => Box::new(led_disk_emulator::LedDiskEmulatorScreen::new()),
    }
}

pub enum ScreenType {
    Raw,
    LedDiskEmulator,
}

pub trait Screen {
    fn setup(&mut self, gl: &gfx::gl::Gl);
    fn render_from_texture(&self, gl: &gfx::gl::Gl, _texture: u32);
}
