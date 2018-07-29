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

pub fn send_binary(pin_number: u64, binary: Vec<u8>) {
    let pin = Pin::new(pin_number);

    let get_bit = |byte, n| -> u8 {
        (byte & (1 as u8) << n) >> n
    };

    let mut bits = Vec::new();

    for byte in binary.iter() {
        for n in (0..8).rev() {
            let bit = get_bit(byte, n);
            bits.push(bit);
        }
    }

    // TODO: Actually enable.
    return;
    pin.set_direction(Direction::Out).unwrap();
    pin.with_exported(|| {
        for bit in bits.iter() {
            pin.set_value(*bit).unwrap();
        }

        Ok(())
    }).unwrap();
}
