use core::{fmt::Debug, time::Duration};

use measurements::{Humidity, Temperature};

#[derive(Debug, Copy, Clone)]
pub struct CommandConvert {
    code: u16,
    duration: Duration,
}

impl CommandConvert {
    pub fn new(code: u16, duration: Duration) -> Self {
        Self { code, duration }
    }
}

impl From<CommandConvert> for u16 {
    fn from(value: CommandConvert) -> Self {
        value.code
    }
}

impl From<CommandConvert> for Duration {
    fn from(value: CommandConvert) -> Self {
        value.duration
    }
}

/// Represents all possible commands for SGP40 and SGP41 sensors.
#[derive(Debug, Copy, Clone)]
pub enum Command {
    /// Execute conditioning cycle (SGP41 only).
    ExecuteConditioning,

    /// Measure raw signal with temperature and humidity compensation.
    MeasureRawSignalWithCompensation(Temperature, Humidity),

    /// Measure raw signal without compensation.
    MeasureRawSignal,

    /// Perform device self-test.
    ExecuteSelfTest,

    /// Disable the heater.
    HeaterDisable,

    /// Fetch the device serial number.
    SerialNumberFetch,
}
