#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs)]
#![cfg_attr(not(test), no_std)]
#[cfg(debug_assertions)]
extern crate alloc;

use bitflags::bitflags;
use bytes::Buf;
use core::u16;

///
pub mod blocking;

///
#[cfg(feature = "embedded-hal-async")]
pub mod asynchronous;

#[doc(hidden)]
pub mod command;

///
pub const DEFAULT_I2C_ADDRESS: u8 = 0x59;

///
pub trait DeviceVariant {}

#[derive(Debug, Clone, Copy)]
///
pub struct Sgp40;
impl DeviceVariant for Sgp40 {}

#[derive(Debug, Clone, Copy)]
///
pub struct Sgp41;
impl DeviceVariant for Sgp41 {}

///
#[derive(Debug, Clone, Copy)]
pub enum RawMeasurement {
    ///
    Partial {
        ///
        voc: u16,
    },

    ///
    Full {
        ///
        voc: u16,

        ///
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
    ///
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TestResult: u16 {
        /// One or more tests of the VOC pixel failed
        const VOC_FAILURE = 1 << 0;

        /// One or more tests of the NOx pixel failed
        const NOX_FAILURE = 1 << 1;
    }
}

impl TestResult {
    ///
    pub fn voc_failure(&self) -> bool {
        self.intersects(TestResult::VOC_FAILURE)
    }

    ///
    pub fn nox_failure(&self) -> bool {
        self.intersects(TestResult::NOX_FAILURE)
    }
}
