use sysfs_gpio::{Direction, Pin};
use std::thread::sleep;
use std::time::Duration;

#[allow(dead_code)]
fn blink_pin(pin_number: u64) {
    let pin = Pin::new(pin_number);
    pin.set_direction(Direction::Out).unwrap();
    pin.with_exported(|| {
        loop {
            pin.set_value(0).unwrap();
            sleep(Duration::from_millis(200));

            pin.set_value(1).unwrap();
            sleep(Duration::from_millis(200));
        }
    }).unwrap();
}
