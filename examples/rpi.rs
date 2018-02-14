extern crate linux_embedded_hal  as hal;
extern crate bh1750;

use std::thread;
use std::time::Duration;

use hal::{Delay, I2cdev};
use bh1750::{BH1750, MeasurementTime, Resolution};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut bh1750 = BH1750::new(dev, Delay);
    // or with explicit i2c address
    // let mut bh1750 = BH1750::with_address(dev, Delay, bh1750::Address::Low);

    bh1750.set_measurement_time(MeasurementTime::Default).unwrap();
    bh1750.reset().unwrap();

    println!("HiResMode 1.0 lx resolution");
    bh1750.set_resolution(Resolution::Lx1_0);
    for _ in 0..5 {
        println!("{:>6} lx", bh1750.illuminance().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("HiResMode2 0.5 lx resolution");
    bh1750.set_resolution(Resolution::Lx0_5);

    for _ in 0..5 {
        println!("{:>6} lx", bh1750.illuminance().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("LowResMode 4.0 lx resolution");
    bh1750.set_resolution(Resolution::Lx4_0);
    for _ in 0..5 {
        println!("{:>6} lx", bh1750.illuminance().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("HiResMode 1.85 lx resolution and min measurement time (31)");
    bh1750.set_resolution(Resolution::Lx1_0);
    bh1750.set_measurement_time(MeasurementTime::Custom(31)).unwrap();

    for _ in 0..5 {
        println!("{:>6} lx", bh1750.illuminance().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("HiResMode 0.23 lx resolution and max measurement time (254)");
    bh1750.set_resolution(Resolution::Lx1_0);
    bh1750.set_measurement_time(MeasurementTime::Custom(254)).unwrap();

    for _ in 0..5 {
        println!("{:>6} lx", bh1750.illuminance().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("HiResMode2 0.11 lx resolution and max measurement time (254)");
    bh1750.reset().unwrap();
    bh1750.set_resolution(Resolution::Lx0_5);
    bh1750.set_measurement_time(MeasurementTime::Custom(254)).unwrap();

    for _ in 0..5 {
        println!("{:>6} lx", bh1750.illuminance().unwrap());
        thread::sleep(Duration::from_secs(1));
    }
}
