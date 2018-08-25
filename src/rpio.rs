use sysfs_gpio::{Direction, Pin};
use std::thread::sleep;
use std::time::Duration;

pub fn read_button(pin: u64) {
    let pin = Pin::new(pin);
    pin.set_direction(Direction::In).unwrap();
    pin.set_active_low(false).unwrap();

    pin.with_exported(|| {
        loop {
            println!("{:?}", pin.get_value().unwrap());
            sleep(Duration::from_millis(200));
        }

    }).unwrap();
}
echo 24 > /sys/class/gpio/export
