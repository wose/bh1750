extern crate linux_embedded_hal  as hal;
extern crate bh1750;

use std::thread;
use std::time::Duration;

use hal::{Delay, I2cdev};
use bh1750::{BH1750, MeasurementMode};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut bh1750 = BH1750::new(dev, Delay);
    // or with explicit i2c address
    // let mut bh1750 = BH1750::with_address(dev, Delay, bh1750::Address::Low);

    println!("HiResMode");
    bh1750.set_measurement_mode(MeasurementMode::ContHRes);
    for _ in 0..5 {
        println!("{:>6} lx", bh1750.light_level().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("HiResMode2");
    bh1750.set_measurement_mode(MeasurementMode::ContHRes2);

    for _ in 0..5 {
        println!("{:>6} lx", bh1750.light_level().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("LowResMode");
    bh1750.set_measurement_mode(MeasurementMode::ContLRes);
    for _ in 0..5 {
        println!("{:>6} lx", bh1750.light_level().unwrap());
        thread::sleep(Duration::from_secs(1));
    }
}
