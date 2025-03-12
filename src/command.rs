use core::{fmt::Debug, time::Duration};

use measurements::{Humidity, Temperature};

use crate::{DeviceVariant, Sgp40, Sgp41};

/// Enumeration of all possible commands.
#[derive(Debug, Copy, Clone)]
pub enum Command<V: DeviceVariant + Copy + Debug> {
    // 0x26 0x12 6 3 45 50
    ExecuteConditioning,

    MeasureRawSignalWithCompensation(Temperature, Humidity),

    MeasureRawSignal,

    ExecuteSelfTest,

    HeaterDisable,

    SerialNumberFetch,

    Variant(V),
}

impl From<Command<Sgp40>> for u16 {
    fn from(value: Command<Sgp40>) -> Self {
        match value {
            Command::ExecuteConditioning => panic!("Sgp40 doesn't support ExecuteConditioning"),
            Command::MeasureRawSignalWithCompensation(..) => 0x2616,
            Command::MeasureRawSignal => 0x2619,
            Command::ExecuteSelfTest => 0x280e,
            Command::HeaterDisable => 0x3615,
            Command::SerialNumberFetch => 0x3682,
            Command::Variant(..) => {
                panic!("Noop is an invalid command and only used to track device type.")
            }
        }
    }
}

impl From<Command<Sgp40>> for Duration {
    fn from(value: Command<Sgp40>) -> Self {
        match value {
            Command::ExecuteConditioning => panic!("Sgp40 doesn't support ExecuteConditioning"),
            Command::MeasureRawSignalWithCompensation(..) => Duration::from_millis(50),
            Command::MeasureRawSignal => Duration::from_millis(30),
            Command::ExecuteSelfTest => Duration::from_millis(320),
            Command::HeaterDisable => Duration::from_millis(1),
            Command::SerialNumberFetch => Duration::from_millis(1),
            Command::Variant(..) => {
                panic!("Noop is an invalid command and only used to track device type.")
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
                panic!("Noop is an invalid command and only used to track device type.")
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
                panic!("Noop is an invalid command and only used to track device type.")
            }
        }
    }
}
