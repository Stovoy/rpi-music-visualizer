use gfx;

mod hardware;
mod led_disk_emulator;
mod raw;

pub fn create_screen(selected_screen: String) -> Box<Screen> {
    match selected_screen.as_ref() {
        "raw" => Box::new(raw::RawScreen::new()),
        "emulator" => Box::new(led_disk_emulator::LedDiskEmulatorScreen::new()),
        "hardware" => Box::new(hardware::HardwareScreen::new()),

        _ => Box::new(led_disk_emulator::LedDiskEmulatorScreen::new()),
    }
}

pub trait Screen {
    fn setup(&mut self, gl: &gfx::gl::Gl);
    fn render_from_texture(&mut self, gl: &gfx::gl::Gl, _texture: u32);
    fn uses_window(&self) -> bool;
}
