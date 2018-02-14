//! A platform agnostic driver to interface with the BH1750 ambient light sensor.
//!
//! This driver was built using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/~0.1

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

extern crate embedded_hal as hal;

use hal::blocking::delay::DelayMs;
use hal::blocking::i2c::{Write, Read};

#[allow(dead_code)]
#[derive(Copy, Clone)]
enum Command {
    /// No active state.
    PowerDown = 0b0000_0000,
    /// Waiting for measurement.
    PowerOn = 0b0000_0001,
    /// Reset Data register value. Reset command is not acceptable in Power Down mode.
    Reset = 0b0000_0111,
    /// Start measurement at 1 lx resolution. Measurement time is typically 120 ms.
    ContHResMode = 0b0001_0000,
    /// Start measurement at 0.5 lx resolution. Measurement time is typically 120 ms.
    ContHResMode2 = 0b0001_0001,
    /// Start measurement at 4 lx resolution. Measurement time is typically 16 ms.
    ContLResMode = 0b0001_0011,
    /// Start measurement at 1 lx resolution. Measurement time is typically 120 ms.
    /// It is automatically set to *Power Down* mode after measurement.
    OneTimeHResMode = 0b0010_0000,
    /// Start measurement at 0.5 lx resolution. Measurement time is typically 120 ms.
    /// It is automatically set to *Power Down* mode after measurement.
    OneTimeHResMode2 = 0b0010_0001,
    /// Start measurement at 4 lx resolution. Measurement time is typically 16 ms.
    /// It is automatically set to *Power Down* mode after measurement.
    OneTimeLResMode = 0b0010_0011,
    /// Change measurement time (High bit) 0b01000_MT[7,6,5]
    ChangeMeasurementTimeHB = 0b01000_000,
    /// Change measurement time (Low bit) 0b011_MT[4,3,2,1,0]
    ChangeMeasurementTimeLB = 0b011_00000,

}

impl Command {
    pub fn cmd(&self) -> u8 {
        *self as u8
    }
}

/// Measurement resolution
#[derive(Clone, Copy)]
pub enum Resolution {
    /// Resolution of 0.5 lx.
    Lx0_5,
    /// Resolution of 1.0 lx.
    Lx1_0,
    /// Resolution of 4.0 lx.
    Lx4_0,
}

/// Measurement mode
#[derive(Clone, Copy)]
pub enum MeasurementMode {
    /// Continious measurement.
    Continious,
    /// One time measurement and go to sleep mode right after.
    OneTime,
}

/// I2C address
#[derive(Clone, Copy)]
pub enum Address {
    /// The i2c address if the `ADDR` pin is low.
    Low = 0x23,
    /// The i2c address if the `ADDR` pin is high.
    High = 0x5C,
}

impl Address {
    fn addr(&self) -> u8 {
        *self as u8
    }
}

/// Measurement Time
#[derive(Clone, Copy)]
pub enum MeasurementTime {
    /// Default measurement time of 69.
    Default,
    /// Custom measurement time (31-254).
    Custom(u8),
}

impl MeasurementTime {
    fn high_byte(&self) -> u8 {
        match *self {
            MeasurementTime::Default => Command::ChangeMeasurementTimeHB.cmd() | (69 >> 5),
            MeasurementTime::Custom(val) => Command::ChangeMeasurementTimeHB.cmd() | (val >> 5),
        }
    }

    fn low_byte(&self) -> u8 {
        match *self {
            MeasurementTime::Default => Command::ChangeMeasurementTimeLB.cmd() | (69 & 0b000_11111),
            MeasurementTime::Custom(val) => Command::ChangeMeasurementTimeLB.cmd() | (val & 0b000_11111),
        }
    }
}

/// BH1750 Driver
pub struct BH1750<I2C, D> {
    addr: Address,
    mode: MeasurementMode,
    mt: MeasurementTime,
    res: Resolution,
    i2c: I2C,
    delay: D,
}

impl <I2C, D, E> BH1750<I2C, D>
where
    I2C: Read<Error = E> + Write<Error = E>,
    D: DelayMs<u16>,
{
    /// Creates a new driver from an I2C peripheral.
    pub fn new(i2c: I2C, delay: D) -> Self {
        BH1750 {
            addr: Address::Low,
            mode: MeasurementMode::OneTime,
            mt: MeasurementTime::Default,
            res: Resolution::Lx1_0,
            i2c: i2c,
            delay: delay
        }
    }

    /// Creates a new driver from an I2C peripheral and the given i2c address.
    pub fn with_address(i2c: I2C, delay: D, address: Address) -> Self {
        BH1750 {
            addr: address,
            mode: MeasurementMode::Continious,
            mt: MeasurementTime::Default,
            res: Resolution::Lx1_0,
            i2c: i2c,
            delay: delay
        }
    }

    /// Measure illuminance.
    pub fn illuminance(&mut self) -> Result<f32, E> {
        let cmd = self.measurement_command();
        self.command(cmd)?;
        self.delay();
        let light = self.read_measurement()?;
        Ok(light)
    }

    /// Set measurement mode.
    pub fn set_measurement_mode(&mut self, mode: MeasurementMode) {
        self.mode = mode;
    }

    /// Set measurement time.
    pub fn set_measurement_time(&mut self, mt: MeasurementTime) -> Result<(), E> {
        // Sensor doesn't support multiple commands without stop condition.
        self.i2c.write(self.addr.addr(), &[mt.high_byte()])?;
        self.i2c.write(self.addr.addr(), &[mt.low_byte()])?;
        self.mt = mt;
        Ok(())
    }

    /// Set resolution.
    pub fn set_resolution(&mut self, res: Resolution) {
        self.res = res;
    }

    /// Wakeup from sleep mode.
    pub fn power_on(&mut self) -> Result<(), E> {
        self.command(Command::PowerOn)
    }

    /// Stop all measurements and enter sleep mode.
    pub fn power_down(&mut self) -> Result<(), E> {
        self.command(Command::PowerDown)
    }

    /// Reset Data register value.
    pub fn reset(&mut self) -> Result<(), E> {
        self.power_on()?;
        self.command(Command::Reset)
    }

    fn command(&mut self, command: Command) -> Result<(), E> {
        self.i2c.write(self.addr.addr(), &[command.cmd()])
    }

    fn delay(&mut self) {
        let delay = match self.res {
            Resolution::Lx4_0 => 24,
            _ => match self.mt {
                MeasurementTime::Default => 180,
                // If the measurement time is doubled we need to double the delay as well.
                MeasurementTime::Custom(mt) => {
                    let scaling = mt as f32 / 69.0;
                    (180.0 * scaling) as u16
                },
            },
        };

        self.delay.delay_ms(delay);
    }

    fn measurement_command(&self) -> Command {
        match self.mode {
            MeasurementMode::Continious => match self.res {
                Resolution::Lx0_5 => Command::ContHResMode2,
                Resolution::Lx1_0 => Command::ContHResMode,
                Resolution::Lx4_0 => Command::ContLResMode,
            },
            MeasurementMode::OneTime => match self.res {
                Resolution::Lx0_5 => Command::OneTimeHResMode2,
                Resolution::Lx1_0 => Command::OneTimeHResMode,
                Resolution::Lx4_0 => Command::OneTimeLResMode,
            }
        }
    }

    fn read_measurement(&mut self) -> Result<f32, E> {
        let value = self.read_u16()? as f32 / 1.2;
        let scaling = match self.mt {
            MeasurementTime::Default => 1.0,
            MeasurementTime::Custom(mt) => 69.0 / mt as f32,
        };

        let light = match self.res {
            Resolution::Lx0_5 => value / 2.0 * scaling,
            Resolution::Lx1_0 => value * scaling,
            Resolution::Lx4_0 => value,
        };
        Ok(light)
    }

    fn read_u16(&mut self) -> Result<u16, E> {
        let mut buffer = [0, 0];
        self.i2c.read(self.addr.addr(), &mut buffer)?;
        Ok(((buffer[0] as u16) << 8) + (buffer[1] as u16))
    }
}
