use gfx;

#[cfg(feature="hardware")]
mod hardware;
mod led_disk_emulator;
mod raw;

pub fn create_screen(selected_screen: String) -> Box<dyn Screen> {
    match selected_screen.as_ref() {
        "raw" => Box::new(raw::RawScreen::new()),
        "emulator" => Box::new(led_disk_emulator::LedDiskEmulatorScreen::new()),
        #[cfg(feature="hardware")]
        "hardware" => Box::new(hardware::HardwareScreen::new()),

        _ => Box::new(led_disk_emulator::LedDiskEmulatorScreen::new()),
    }
}

pub trait Screen {
    fn setup(&mut self, gl: &gfx::gl::Gl);
    fn render_from_texture(&mut self, gl: &gfx::gl::Gl, texture: u32, size: i32);
    fn uses_window(&self) -> bool;
}
