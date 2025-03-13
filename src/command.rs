use core::{fmt::Debug, time::Duration};

use measurements::{Humidity, Temperature};

use crate::{DeviceVariant, Sgp40, Sgp41};

/// Represents all possible commands for SGP40 and SGP41 sensors.
#[derive(Debug, Copy, Clone)]
pub enum Command<V: DeviceVariant> {
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

    /// Placeholder variant for type-level device differentiation.
    Variant(V),
}

impl From<Command<Sgp40>> for u16 {
    fn from(value: Command<Sgp40>) -> Self {
        match value {
            Command::ExecuteConditioning => panic!("ExecuteConditioning not supported on SGP40"),
            Command::MeasureRawSignalWithCompensation(..) => 0x2616,
            Command::MeasureRawSignal => 0x2619,
            Command::ExecuteSelfTest => 0x280e,
            Command::HeaterDisable => 0x3615,
            Command::SerialNumberFetch => 0x3682,
            Command::Variant(..) => {
                panic!("Variant is a type-level placeholder, not a valid command")
            }
        }
    }
}

impl From<Command<Sgp40>> for Duration {
    fn from(value: Command<Sgp40>) -> Self {
        match value {
            Command::ExecuteConditioning => panic!("ExecuteConditioning not supported on SGP40"),
            Command::MeasureRawSignalWithCompensation(..) => Duration::from_millis(50),
            Command::MeasureRawSignal => Duration::from_millis(30),
            Command::ExecuteSelfTest => Duration::from_millis(320),
            Command::HeaterDisable => Duration::from_millis(1),
            Command::SerialNumberFetch => Duration::from_millis(1),
            Command::Variant(..) => {
                panic!("Variant is a type-level placeholder, not a valid command")
            }
        }
    }
}

impl From<Command<Sgp41>> for u16 {
    fn from(value: Command<Sgp41>) -> Self {
        match value {
            Command::ExecuteConditioning => 0x2612,
            Command::MeasureRawSignalWithCompensation(..) => 0x260f,
            Command::MeasureRawSignal => 0x2619,
            Command::ExecuteSelfTest => 0x280e,
            Command::HeaterDisable => 0x3615,
            Command::SerialNumberFetch => 0x3682,
            Command::Variant(..) => {
                panic!("Variant is a type-level placeholder, not a valid command")
            }
        }
    }
}

impl From<Command<Sgp41>> for Duration {
    fn from(value: Command<Sgp41>) -> Self {
        match value {
            Command::ExecuteConditioning => Duration::from_millis(50),
            Command::MeasureRawSignalWithCompensation(..) => Duration::from_millis(50),
            Command::MeasureRawSignal => Duration::from_millis(50),
            Command::ExecuteSelfTest => Duration::from_millis(320),
            Command::HeaterDisable => Duration::from_millis(1),
            Command::SerialNumberFetch => Duration::from_millis(1),
            Command::Variant(..) => {
                panic!("Variant is a type-level placeholder, not a valid command")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
