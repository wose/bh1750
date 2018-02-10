extern crate linux_embedded_hal  as hal;
extern crate bh1750;

use std::thread;
use std::time::Duration;

use hal::{Delay, I2cdev};
use bh1750::{BH1750};

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut bh1750 = BH1750::new(dev, Delay);

    loop {
        println!("{:>6} lx", bh1750.light_level().unwrap());
        thread::sleep(Duration::from_secs(1));
    }
}
