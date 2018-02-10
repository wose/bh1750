//! A platform agnostic driver to interface with the BH1750 ambient light sensor.
//!
//! This driver was built using [`embedded-hal`] traits.
//!
//! [`embedded-hal`]: https://docs.rs/embedded-hal/~0.1

//#![deny(missing_docs)]
//#![deny(warnings)]
#![no_std]

extern crate embedded_hal as hal;

use hal::blocking::delay::DelayMs;
use hal::blocking::i2c::{Write, Read};

const ADDRESS: u8 = 0x23;

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

#[derive(Clone, Copy, Debug)]
pub enum MeasurementMode {
    ContHRes,
    ContHRes2,
    ContLRes,
    OneTimeHRes,
    OneTimeHRes2,
    OneTimeLRes,
}

impl MeasurementMode {
    /// Get command to set mode.
    fn cmd(&self) -> Command {
        match *self {
            MeasurementMode::ContHRes => Command::ContHResMode,
            MeasurementMode::ContHRes2 => Command::ContHResMode2,
            MeasurementMode::ContLRes => Command::ContLResMode,
            MeasurementMode::OneTimeHRes => Command::OneTimeHResMode,
            MeasurementMode::OneTimeHRes2 => Command::OneTimeHResMode2,
            MeasurementMode::OneTimeLRes => Command::OneTimeLResMode,
        }
    }
}

/// BH1750 Driver
pub struct BH1750<I2C, D> {
    mode: MeasurementMode,
    i2c: I2C,
    delay: D,
}

impl <I2C, D, E> BH1750<I2C, D>
where
    I2C: Read<Error = E> + Write<Error = E>,
    D: DelayMs<u8>,
{
    /// Creates a new driver from an I2C peripheral.
    pub fn new(i2c: I2C, delay: D) -> Self {
        BH1750 { mode: MeasurementMode::OneTimeHRes, i2c, delay }
    }

    /// Measure ambient light level.
    pub fn light_level(&mut self) -> Result<f32, E> {
        let cmd = self.mode.cmd();
        self.command(cmd)?;
        self.delay();
        let light = self.read_measurement()?;
        Ok(light)
    }

    /// Set measurement mode.
    pub fn set_measurement_mode(&mut self, mode: MeasurementMode) {
        self.mode = mode;
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
        self.reset()
    }

    fn command(&mut self, command: Command) -> Result<(), E> {
        self.i2c.write(ADDRESS, &[command.cmd()])
    }

    fn delay(&mut self) {
        let delay = match self.mode {
            MeasurementMode::ContHRes | MeasurementMode::ContHRes2 |
            MeasurementMode::OneTimeHRes | MeasurementMode::OneTimeHRes2 => 180,
            _ => 24,
        };

        self.delay.delay_ms(delay);
    }

    fn read_measurement(&mut self) -> Result<f32, E> {
        let value = self.read_u16()?;
        let light = match self.mode {
            MeasurementMode::ContHRes | MeasurementMode::OneTimeHRes |
            MeasurementMode::ContLRes | MeasurementMode::OneTimeLRes => value as f32 / 1.2,
            MeasurementMode::ContHRes2 | MeasurementMode::OneTimeHRes2 => value as f32 / 2.4,
        };
        Ok(light)
    }

    fn read_u16(&mut self) -> Result<u16, E> {
        let mut buffer = [0, 0];
        self.i2c.read(ADDRESS, &mut buffer)?;
        Ok(((buffer[0] as u16) << 8) + (buffer[1] as u16))
    }
}
