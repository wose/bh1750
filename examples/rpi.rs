extern crate linux_embedded_hal  as hal;
extern crate bh1750;

use std::thread;
use std::time::Duration;

use hal::{Delay, I2cdev};
use bh1750::{BH1750, Resolution};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut bh1750 = BH1750::new(dev, Delay);
    // or with explicit i2c address
    // let mut bh1750 = BH1750::with_address(dev, Delay, bh1750::Address::Low);

    println!("HiResMode 1.0 lx resolution");
    bh1750.set_resolution(Resolution::Lx1_0);
    for _ in 0..5 {
        println!("{:>6} lx", bh1750.light_level().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("HiResMode2 0.5 lx resolution");
    bh1750.set_resolution(Resolution::Lx0_5);

    for _ in 0..5 {
        println!("{:>6} lx", bh1750.light_level().unwrap());
        thread::sleep(Duration::from_secs(1));
    }

    println!("LowResMode 4.0 lx resolution");
    bh1750.set_resolution(Resolution::Lx4_0);
    for _ in 0..5 {
        println!("{:>6} lx", bh1750.light_level().unwrap());
        thread::sleep(Duration::from_secs(1));
    }
}
