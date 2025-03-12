#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs)]
#![cfg_attr(not(test), no_std)]
#[cfg(debug_assertions)]
extern crate alloc;

use bitflags::bitflags;
use bytes::Buf;
use core::fmt::Debug;
use core::u16;

/// Module containing blocking implementation for device communication.
pub mod blocking;

/// Module containing asynchronous implementation for device communication.
#[cfg(feature = "embedded-hal-async")]
pub mod asynchronous;

#[doc(hidden)]
pub mod command;

/// Default I2C address for the SGP40/SGP41 sensor.
pub const DEFAULT_I2C_ADDRESS: u8 = 0x59;

/// Trait for representing typestate based on device variant.
pub trait DeviceVariant: Copy + Debug {}

/// SGP40 typestate representation.
#[derive(Debug, Clone, Copy)]
pub struct Sgp40;
impl DeviceVariant for Sgp40 {}

/// SGP41 typestate representation.
#[derive(Debug, Clone, Copy)]
pub struct Sgp41;
impl DeviceVariant for Sgp41 {}

/// Represents raw measurement data returned from the device.
#[derive(Debug, Clone, Copy)]
pub enum RawMeasurement {
    /// VOC-only measurement used by SGP40 and SGP41 during conditioning.
    Partial {
        /// Volatile Organic Compounds (VOC) measurement.
        voc: u16,
    },

    /// VOC and NOx measurements used by SGP41.
    Full {
        /// Volatile Organic Compounds (VOC) measurement.
        voc: u16,

        /// Nitrogen Oxides (NOx) measurement.
        nox: u16,
    },
}

impl<T: AsRef<[u8]>> From<T> for RawMeasurement {
    fn from(value: T) -> Self {
        match value.as_ref().len() {
            3 => RawMeasurement::Partial {
                voc: value.as_ref().get(0..2).unwrap().get_u16(),
            },
            6 => RawMeasurement::Full {
                voc: value.as_ref().get(0..2).unwrap().get_u16(),
                nox: value.as_ref().get(3..5).unwrap().get_u16(),
            },
            _ => panic!("Measurement must be 3 or 6 bytes"),
        }
    }
}

bitflags! {
    /// Results of the built-in self-test checking for integrity of both hotplate and MOX material.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TestResult: u16 {
        /// Indicates that one or more tests of the VOC pixel failed.
        const VOC_FAILURE = 1 << 0;

        /// Indicates that one or more tests of the NOx pixel failed.
        const NOX_FAILURE = 1 << 1;
    }
}

impl TestResult {
    /// Checks if one or more tests of the VOC pixel failed.
    pub fn voc_failure(&self) -> bool {
        self.intersects(TestResult::VOC_FAILURE)
    }

    /// Checks if one or more tests of the NOx pixel failed.
    pub fn nox_failure(&self) -> bool {
        self.intersects(TestResult::NOX_FAILURE)
    }
}
