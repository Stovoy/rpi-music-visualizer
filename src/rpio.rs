use sysfs_gpio::{Direction, Pin};
use std::thread::sleep;
use std::time::Duration;

pub fn read_button() {
    let pin = Pin::new(24);
    pin.set_direction(Direction::In).unwrap();
    pin.set_active_low(true).unwrap();

    pin.with_exported(|| {
        loop {
            println!("{:?}", pin.get_value().unwrap());
            sleep(Duration::from_millis(200));
        }

    }).unwrap();
}
