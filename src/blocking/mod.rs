use core::{fmt::Debug, marker::PhantomData, time::Duration};

use bytes::{Buf, BytesMut};
use embedded_hal::i2c::SevenBitAddress;
use measurements::{Humidity, Temperature};
use sensirion_core::{Error, blocking::SensirionI2c};

use crate::{
    DEFAULT_I2C_ADDRESS, DeviceVariant, RawMeasurement, Sgp40, Sgp41, TestResult, command::Command,
};

/// A blocking driver for the SGP4x device.
#[derive(Debug, Default)]
pub struct Sgp4x<I2C, A, D, V> {
    i2c: I2C,
    address: A,
    delay: D,
    variant: PhantomData<V>,
}

impl<I2C, A, D, V> Sgp4x<I2C, A, D, V>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    /// Transforms the driver into a different variant.
    pub fn into_variant<T: DeviceVariant>(self) -> Sgp4x<I2C, A, D, T> {
        Sgp4x::<I2C, A, D, T> {
            i2c: self.i2c,
            address: self.address,
            delay: self.delay,
            variant: PhantomData::default(),
        }
    }
}

impl<I2C, D, V> SensirionI2c<I2C, D> for Sgp4x<I2C, SevenBitAddress, D, V>
where
    I2C: embedded_hal::i2c::I2c + Debug,
    D: embedded_hal::delay::DelayNs,
    V: DeviceVariant,
{
    fn i2c(&mut self) -> &mut I2C {
        &mut self.i2c
    }

    fn address(&mut self) -> SevenBitAddress {
        self.address
    }

    fn delay(&mut self) -> &mut D {
        &mut self.delay
    }
}

impl<I2C, D> Sgp4x<I2C, SevenBitAddress, D, Sgp41>
where
    I2C: embedded_hal::i2c::I2c + Debug,
    D: embedded_hal::delay::DelayNs,
    u16: From<Command<Sgp41>>,
    Duration: From<Command<Sgp41>>,
{
    /// Initiates the conditioning process for the device.
    pub fn execute_conditioning(&mut self) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(3);
        buffer.resize(3, 0);

        self.read_command(Command::<Sgp41>::ExecuteConditioning, &mut buffer)
            .map(RawMeasurement::from)
    }

    /// Performs a measurement of raw signals without humidity compensation.
    pub fn measure_raw_signals(&mut self) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(6);
        buffer.resize(6, 0);

        self.read_command(Command::<Sgp41>::MeasureRawSignal, &mut buffer)
            .map(RawMeasurement::from)
    }

    /// Performs a measurement of raw signals with humidity compensation.
    pub fn measure_raw_signals_with_compensation(
        &mut self,
        temperature: Temperature,
        humidity: Humidity,
    ) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(6);
        buffer.resize(6, 0);

        let temperature_bytes =
            (((temperature.as_celsius() + 45.0) * u16::MAX as f64) / 175.0) as u16;
        let humidity_bytes = ((humidity.as_percent() * u16::MAX as f64) / 100.0) as u16;

        self.read_command_with_args(
            Command::<Sgp41>::MeasureRawSignalWithCompensation(temperature, humidity),
            Some(&[temperature_bytes, humidity_bytes]),
            &mut buffer,
        )
        .map(RawMeasurement::from)
    }
}

impl<I2C, D> Sgp4x<I2C, SevenBitAddress, D, Sgp40>
where
    I2C: embedded_hal::i2c::I2c + Debug,
    D: embedded_hal::delay::DelayNs,
    u16: From<Command<Sgp40>>,
    Duration: From<Command<Sgp40>>,
{
    /// Performs a measurement of raw signals without humidity compensation.
    pub fn measure_raw_signals(&mut self) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(6);
        buffer.resize(3, 0);

        self.read_command(Command::<Sgp40>::MeasureRawSignal, &mut buffer)
            .map(RawMeasurement::from)
    }

    /// Performs a measurement of raw signals with humidity compensation.
    pub fn measure_raw_signals_with_compensation(
        &mut self,
        temperature: Temperature,
        humidity: Humidity,
    ) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(6);
        buffer.resize(3, 0);

        let temperature_bytes =
            (((temperature.as_celsius() + 45.0) * u16::MAX as f64) / 175.0) as u16;
        let humidity_bytes = ((humidity.as_percent() * u16::MAX as f64) / 100.0) as u16;

        self.read_command_with_args(
            Command::<Sgp40>::MeasureRawSignalWithCompensation(temperature, humidity),
            Some(&[temperature_bytes, humidity_bytes]),
            &mut buffer,
        )
        .map(RawMeasurement::from)
    }
}

impl<I2C, D, V> Sgp4x<I2C, SevenBitAddress, D, V>
where
    I2C: embedded_hal::i2c::I2c + Debug,
    D: embedded_hal::delay::DelayNs,
    V: DeviceVariant,
    u16: From<Command<V>>,
    Duration: From<Command<V>>,
{
    /// Creates a new instance of the SGP4x device driver.
    pub fn new(i2c: I2C, delay: D) -> Self {
        Self::new_with_variant::<Sgp41>(i2c, delay)
    }

    /// Creates a new instance of the SGP4x device driver with a specific variant.
    pub fn new_with_variant<T: DeviceVariant>(i2c: I2C, delay: D) -> Self {
        Self {
            address: DEFAULT_I2C_ADDRESS,
            i2c,
            delay,
            variant: PhantomData::default(),
        }
    }

    /// Executes the device's built-in self-test.
    pub fn self_test(&mut self) -> Result<TestResult, Error<I2C::Error>> {
        let mut test_result = BytesMut::with_capacity(8);
        test_result.resize(3, 0);

        self.read_command(Command::ExecuteSelfTest, &mut test_result)
            .map(|test_result| {
                TestResult::from_bits_retain(test_result.get(0..2).unwrap().get_u16())
            })
    }

    /// Disables the device's heater and stops measurements.
    pub fn heater_disable(&mut self) -> Result<(), Error<I2C::Error>> {
        self.write_command(Command::HeaterDisable)
    }

    /// Retrieves the device's serial number.
    pub fn serial_number_fetch(&mut self) -> Result<u64, Error<I2C::Error>> {
        let mut serial_number = BytesMut::with_capacity(9);
        serial_number.resize(9, 0);

        self.read_command(Command::SerialNumberFetch, &mut serial_number)
            .map(|serial_number| {
                u64::from_be_bytes([
                    0,
                    0,
                    serial_number[0],
                    serial_number[1],
                    serial_number[3],
                    serial_number[4],
                    serial_number[6],
                    serial_number[7],
                ])
            })
    }
}

#[cfg(test)]
mod tests {
    use core::u16;

    use crate::{Sgp40, Sgp41};
    use assert_matches::assert_matches;
    use bytes::BufMut;

    use super::*;

    use embedded_hal_mock::eh1::{delay::NoopDelay, i2c::Transaction};
    use measurements::{Humidity, Temperature};

    fn create_i2c<V, T>(expectations: &[Transaction], action: T)
    where
        T: FnOnce(Sgp4x<&mut embedded_hal_mock::common::Generic<Transaction>, u8, NoopDelay, V>),
        V: DeviceVariant,
        u16: From<Command<V>>,
        Duration: From<Command<V>>,
    {
        let mut i2c_mock = embedded_hal_mock::eh1::i2c::Mock::new(expectations);
        let sht3x = create_device(&mut i2c_mock);
        action(sht3x);
        i2c_mock.done();
    }

    fn create_device<V>(
        i2c: &mut embedded_hal_mock::common::Generic<Transaction>,
    ) -> Sgp4x<&mut embedded_hal_mock::common::Generic<Transaction>, u8, NoopDelay, V>
    where
        V: DeviceVariant,
        u16: From<Command<V>>,
        Duration: From<Command<V>>,
    {
        Sgp4x::new_with_variant::<V>(i2c, NoopDelay::default())
    }

    #[test]
    fn test_new() {
        create_i2c::<Sgp40, _>(&[], |device| {
            log::info!("Address: {}", device.address);
        });
    }

    #[test]
    fn test_heater_disable() {
        let expectations = [Transaction::write(
            DEFAULT_I2C_ADDRESS,
            u16::from(Command::<Sgp40>::HeaterDisable)
                .to_be_bytes()
                .to_vec(),
        )];

        create_i2c::<Sgp40, _>(&expectations, |mut device| {
            assert!(device.heater_disable().is_ok());
        });
    }

    #[test]
    fn test_serial_number_fetch() {
        let raw_serial_number = 128;
        let mut serial_number = BytesMut::with_capacity(9);
        for x in 0..3 {
            let put_number = match x {
                2 => raw_serial_number,
                _ => 0,
            } as u16;
            serial_number.put_u16(put_number);
            serial_number.put_u8(sensirion_i2c::crc8::calculate(
                &put_number.to_be_bytes().to_vec(),
            ));
        }

        let expectations = [
            Transaction::write(
                DEFAULT_I2C_ADDRESS,
                u16::from(Command::<Sgp40>::SerialNumberFetch)
                    .to_be_bytes()
                    .to_vec(),
            ),
            Transaction::read(DEFAULT_I2C_ADDRESS, serial_number.to_vec()),
        ];

        create_i2c::<Sgp40, _>(&expectations, move |mut device| {
            assert_eq!(device.serial_number_fetch().unwrap(), raw_serial_number);
        });
    }

    #[test]
    fn test_execute_self_test() {
        let mut test_results = BytesMut::with_capacity(3);
        test_results.put_u16(u16::MAX);
        test_results.put_u8(sensirion_i2c::crc8::calculate(
            &u16::MAX.to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(
                DEFAULT_I2C_ADDRESS,
                u16::from(Command::<Sgp40>::ExecuteSelfTest)
                    .to_be_bytes()
                    .to_vec(),
            ),
            Transaction::read(DEFAULT_I2C_ADDRESS, test_results.to_vec()),
        ];

        create_i2c::<Sgp40, _>(&expectations, move |mut device| {
            assert_eq!(
                device.self_test().unwrap(),
                TestResult::from_bits_retain(u16::MAX)
            );
        });
    }

    #[test]
    fn test_sgp40_measure_raw_signals() {
        let mut raw_signals = BytesMut::with_capacity(3);
        raw_signals.put_u16(44);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &44u16.to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(
                DEFAULT_I2C_ADDRESS,
                u16::from(Command::<Sgp40>::MeasureRawSignal)
                    .to_be_bytes()
                    .to_vec(),
            ),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
        ];

        create_i2c::<Sgp40, _>(&expectations, move |mut device| {
            assert_matches!(
                device.measure_raw_signals(),
                Ok(RawMeasurement::Partial { voc: 44 })
            );
        });
    }

    #[test]
    fn test_sgp41_measure_raw_signals() {
        let mut raw_signals = BytesMut::with_capacity(3);
        raw_signals.put_u16(44);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &44u16.to_be_bytes().to_vec(),
        ));
        raw_signals.put_u16(66);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &66u16.to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(
                DEFAULT_I2C_ADDRESS,
                u16::from(Command::<Sgp41>::MeasureRawSignal)
                    .to_be_bytes()
                    .to_vec(),
            ),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
        ];

        create_i2c::<Sgp41, _>(&expectations, move |mut device| {
            assert_matches!(
                device.measure_raw_signals(),
                Ok(RawMeasurement::Full { voc: 44, nox: 66 })
            );
        });
    }

    #[test]
    fn test_sgp40_measure_raw_signals_with_compensation() {
        let temperature = Temperature::from_celsius(50.0);
        let temperature_bytes =
            (((temperature.as_celsius() + 45.0) * u16::MAX as f64) / 175.0) as u16;
        let humidity = Humidity::from_percent(50.0);
        let humidity_bytes = ((humidity.as_percent() * u16::MAX as f64) / 100.0) as u16;

        let mut expected = BytesMut::with_capacity(8);
        expected.put_u16(u16::from(
            Command::<Sgp40>::MeasureRawSignalWithCompensation(temperature, humidity),
        ));
        expected.put_u16(temperature_bytes);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &temperature_bytes.to_be_bytes(),
        ));
        expected.put_u16(humidity_bytes);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &humidity_bytes.to_be_bytes(),
        ));

        let mut raw_signals = BytesMut::with_capacity(3);
        raw_signals.put_u16(44);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &44u16.to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(DEFAULT_I2C_ADDRESS, expected.to_vec()),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
        ];

        create_i2c::<Sgp40, _>(&expectations, move |mut device| {
            assert_matches!(
                device.measure_raw_signals_with_compensation(temperature, humidity),
                Ok(RawMeasurement::Partial { voc: 44 })
            );
        });
    }

    #[test]
    fn test_sgp41_measure_raw_signals_with_compensation() {
        let temperature = Temperature::from_celsius(50.0);
        let temperature_bytes =
            (((temperature.as_celsius() + 45.0) * u16::MAX as f64) / 175.0) as u16;
        let humidity = Humidity::from_percent(50.0);
        let humidity_bytes = ((humidity.as_percent() * u16::MAX as f64) / 100.0) as u16;

        let mut expected = BytesMut::with_capacity(8);
        expected.put_u16(u16::from(
            Command::<Sgp41>::MeasureRawSignalWithCompensation(temperature, humidity),
        ));
        expected.put_u16(temperature_bytes);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &temperature_bytes.to_be_bytes(),
        ));
        expected.put_u16(humidity_bytes);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &humidity_bytes.to_be_bytes(),
        ));

        let mut raw_signals = BytesMut::with_capacity(3);
        raw_signals.put_u16(44);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &44u16.to_be_bytes().to_vec(),
        ));
        raw_signals.put_u16(66);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &66u16.to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(DEFAULT_I2C_ADDRESS, expected.to_vec()),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
        ];

        create_i2c::<Sgp41, _>(&expectations, move |mut device| {
            assert_matches!(
                device.measure_raw_signals_with_compensation(temperature, humidity),
                Ok(RawMeasurement::Full { voc: 44, nox: 66 })
            );
        });
    }

    #[test]
    fn test_sgp41_execute_conditioning() {
        let mut raw_signals = BytesMut::with_capacity(3);
        raw_signals.put_u16(44);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &44u16.to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(
                DEFAULT_I2C_ADDRESS,
                u16::from(Command::<Sgp41>::ExecuteConditioning)
                    .to_be_bytes()
                    .to_vec(),
            ),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
        ];

        create_i2c::<Sgp41, _>(&expectations, move |mut device| {
            assert_matches!(
                device.execute_conditioning(),
                Ok(RawMeasurement::Partial { voc: 44 })
            );
        });
    }

    #[test]
    fn test_into_variant() {
        let mut raw_signals = BytesMut::with_capacity(3);
        raw_signals.put_u16(44);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &44u16.to_be_bytes().to_vec(),
        ));
        raw_signals.put_u16(66);
        raw_signals.put_u8(sensirion_i2c::crc8::calculate(
            &66u16.to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(
                DEFAULT_I2C_ADDRESS,
                u16::from(Command::<Sgp41>::MeasureRawSignal)
                    .to_be_bytes()
                    .to_vec(),
            ),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
            Transaction::write(
                DEFAULT_I2C_ADDRESS,
                u16::from(Command::<Sgp40>::MeasureRawSignal)
                    .to_be_bytes()
                    .to_vec(),
            ),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals[0..3].to_vec()),
        ];

        create_i2c::<Sgp41, _>(&expectations, move |mut device| {
            assert_matches!(
                device.measure_raw_signals(),
                Ok(RawMeasurement::Full { voc: 44, nox: 66 })
            );

            // convert into Sgp40
            let mut new_device = device.into_variant::<Sgp40>();
            assert_matches!(
                new_device.measure_raw_signals(),
                Ok(RawMeasurement::Partial { voc: 44 })
            );
        });
    }
}
