# sensirion-sgp4x

This is a platform agnostic driver for Sensirion SGP4x series of VOC and NOx sensors using traits from [`embedded-hal`]() and [`embedded-hal-async`]()
to allow generalized use on any [`no_std`](https://docs.rust-embedded.org/book/intro/no-std.html) platform. It provides both a blocking interface by default and an asynchronous interface by enabling the `embedded-hal-async` feature.

## Device

The SGP4x family of digital gas sensors, including the SGP40 and SGP41, is designed for seamless integration into air purifiers and demand-controlled ventilation systems. Leveraging Sensirion’s CMOSens® technology, these sensors offer a complete, easy-to-use solution on a single chip, featuring a digital I<sup>2</sup>C interface and temperature-controlled micro hotplates. The SGP40 provides a humidity-compensated VOC-based indoor air quality signal, which can be processed using Sensirion’s Gas Index Algorithm to generate a VOC Index. The SGP41 expands on this functionality by delivering both VOC and NOx-based indoor air quality signals, enabling more comprehensive air quality monitoring.

| Model  | Measures  | Humidity Compensation | Output | Best For |
|--------|----------|----------------------|--------|---------|
| **SGP40** | VOCs | Yes | VOC raw signal (processed into VOC Index) | Air purifiers, HVAC, demand-controlled ventilation |
| **SGP41** | VOCs + NOx | Yes | VOC and NOx raw signals (processed into VOC & NOx Indexes) | Advanced air quality monitoring, smart homes |

## Features

| Feature                               | Blocking  | Async |
| ------------------------------------- | --------- | ----- |
| Execute conditioning (SGP41 only)     | ✅        | ✅    |
| Measure raw signal                    | ✅        | ✅    |
| Measure w/humidity compensation       | ✅        | ✅    |
| Disable heater                        | ✅        | ✅    |
| Execute self test                     | ✅        | ✅    |
| Get serial number                     | ✅        | ✅    |

## Usage

```rust,no_run
use sensirion_core::{Error};
use sensirion_sgp4x::{blocking::Sgp4x, RawMeasurement};
use embedded_hal_mock::eh1::{delay::NoopDelay, i2c};

fn main() {
    // Create the I2C device from the chosen embedded-hal implementation, in this case embedded_hal_mock
    let mut i2c = i2c::Mock::new(&[]);

    // Create the sensor
    let mut sensor = Sgp4x::new(i2c, NoopDelay::default());

    // Perform measurement
    match sensor.execute_conditioning() {
        Ok(RawMeasurement::Full{voc, nox}) => println!(
            "Raw measurement: VOC({}), NOx({})",
            voc,
            nox
        ),
        _ => panic!(""),
    }
}
```
