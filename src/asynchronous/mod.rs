use core::time::Duration;

use bytes::{Buf, BytesMut};
use embedded_hal_async::i2c::SevenBitAddress;
use measurements::{Humidity, Temperature};
use sensirion_core::{Error, asynchronous::SensirionI2c};

use crate::{
    DEFAULT_I2C_ADDRESS, RawMeasurement, TestResult,
    command::{Command, CommandConvert},
};

trait Sgp4x<I2C, D>: SensirionI2c<I2C, D>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    /// Executes the device's built-in self-test.
    async fn self_test(&mut self) -> Result<TestResult, Error<I2C::Error>> {
        let mut test_result = BytesMut::with_capacity(8);
        test_result.resize(3, 0);

        self.read_command(
            self.command_convert(Command::ExecuteSelfTest),
            &mut test_result,
        )
        .await
        .map(|test_result| TestResult::from_bits_retain(test_result.get(0..2).unwrap().get_u16()))
    }

    /// Disables the device's heater and stops measurements.
    async fn heater_disable(&mut self) -> Result<(), Error<I2C::Error>> {
        self.write_command(self.command_convert(Command::HeaterDisable))
            .await
    }

    /// Retrieves the device's serial number.
    async fn serial_number_fetch(&mut self) -> Result<u64, Error<I2C::Error>> {
        let mut serial_number = BytesMut::with_capacity(9);
        serial_number.resize(9, 0);

        self.read_command(
            self.command_convert(Command::SerialNumberFetch),
            &mut serial_number,
        )
        .await
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

    fn default_command_convert(&self, command: Command) -> CommandConvert {
        match command {
            Command::ExecuteSelfTest => CommandConvert::new(0x280e, Duration::from_millis(320)),
            Command::HeaterDisable => CommandConvert::new(0x3615, Duration::from_millis(1)),
            Command::SerialNumberFetch => CommandConvert::new(0x3682, Duration::from_millis(1)),
            _ => todo!(),
        }
    }

    fn command_convert(&self, command: Command) -> CommandConvert {
        self.default_command_convert(command)
    }
}

struct Sgp40<I2C, D> {
    i2c: I2C,
    delay: D,
}

impl<I2C, D> Sgp40<I2C, D> {
    pub fn new(i2c: I2C, delay: D) -> Self {
        Self { i2c, delay }
    }
}

impl<I2C, D> Sgp40<I2C, D>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    /// Performs a measurement of raw signals without humidity compensation.
    pub async fn measure_raw_signals(&mut self) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(3);
        buffer.resize(3, 0);

        self.read_command_with_args(
            self.command_convert(Command::MeasureRawSignal),
            Some(&[0x8000, 0x6666]),
            &mut buffer,
        )
        .await
        .map(RawMeasurement::from)
    }

    /// Performs a measurement of raw signals with humidity compensation.
    pub async fn measure_raw_signals_with_compensation(
        &mut self,
        temperature: Temperature,
        humidity: Humidity,
    ) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(3);
        buffer.resize(3, 0);

        let temperature_bytes =
            (((temperature.as_celsius() + 45.0) * u16::MAX as f64) / 175.0) as u16;
        let humidity_bytes = ((humidity.as_percent() * u16::MAX as f64) / 100.0) as u16;

        self.read_command_with_args(
            self.command_convert(Command::MeasureRawSignalWithCompensation(
                temperature,
                humidity,
            )),
            Some(&[temperature_bytes, humidity_bytes]),
            &mut buffer,
        )
        .await
        .map(RawMeasurement::from)
    }
}

impl<I2C, D> Sgp4x<I2C, D> for Sgp40<I2C, D>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    fn command_convert(&self, command: Command) -> CommandConvert {
        match command {
            Command::MeasureRawSignal => CommandConvert::new(0x260f, Duration::from_millis(30)),
            Command::MeasureRawSignalWithCompensation(..) => {
                CommandConvert::new(0x260f, Duration::from_millis(50))
            }
            _ => self.default_command_convert(command),
        }
    }
}

impl<I2C, D> SensirionI2c<I2C, D> for Sgp40<I2C, D>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    fn i2c(&mut self) -> &mut I2C {
        &mut self.i2c
    }

    fn address(&mut self) -> SevenBitAddress {
        DEFAULT_I2C_ADDRESS
    }

    fn delay(&mut self) -> &mut D {
        &mut self.delay
    }
}

struct Sgp41<I2C, D> {
    i2c: I2C,
    delay: D,
}

impl<I2C, D> Sgp41<I2C, D> {
    pub fn new(i2c: I2C, delay: D) -> Self {
        Self { i2c, delay }
    }
}

impl<I2C, D> Sgp41<I2C, D>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    /// Initiates the conditioning process for the device.
    pub async fn execute_conditioning(&mut self) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(3);
        buffer.resize(3, 0);

        self.read_command_with_args(
            self.command_convert(Command::ExecuteConditioning),
            Some(&[0x8000, 0x6666]),
            &mut buffer,
        )
        .await
        .map(RawMeasurement::from)
    }

    /// Performs a measurement of raw signals without humidity compensation.
    pub async fn measure_raw_signals(&mut self) -> Result<RawMeasurement, Error<I2C::Error>> {
        let mut buffer = BytesMut::with_capacity(6);
        buffer.resize(6, 0);

        self.read_command_with_args(
            self.command_convert(Command::MeasureRawSignal),
            Some(&[0x8000, 0x6666]),
            &mut buffer,
        )
        .await
        .map(RawMeasurement::from)
    }

    /// Performs a measurement of raw signals with humidity compensation.
    pub async fn measure_raw_signals_with_compensation(
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
            self.command_convert(Command::MeasureRawSignalWithCompensation(
                temperature,
                humidity,
            )),
            Some(&[temperature_bytes, humidity_bytes]),
            &mut buffer,
        )
        .await
        .map(RawMeasurement::from)
    }
}

impl<I2C, D> SensirionI2c<I2C, D> for Sgp41<I2C, D>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    fn i2c(&mut self) -> &mut I2C {
        &mut self.i2c
    }

    fn address(&mut self) -> SevenBitAddress {
        DEFAULT_I2C_ADDRESS
    }

    fn delay(&mut self) -> &mut D {
        &mut self.delay
    }
}

impl<I2C, D> Sgp4x<I2C, D> for Sgp41<I2C, D>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    fn command_convert(&self, command: Command) -> CommandConvert {
        match command {
            Command::ExecuteConditioning => CommandConvert::new(0x2612, Duration::from_millis(50)),
            Command::MeasureRawSignal => CommandConvert::new(0x2619, Duration::from_millis(50)),
            Command::MeasureRawSignalWithCompensation(..) => {
                CommandConvert::new(0x2619, Duration::from_millis(50))
            }
            _ => self.default_command_convert(command),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use bytes::BufMut;
    use core::u16;

    use super::*;

    use embedded_hal_mock::{
        common::Generic,
        eh1::{delay::NoopDelay, i2c::Transaction},
    };
    use measurements::{Humidity, Temperature};

    fn create_i2c<A>(expectations: &[Transaction], action: A)
    where
        A: AsyncFnOnce(&mut Generic<Transaction>),
    {
        let mut i2c_mock = embedded_hal_mock::eh1::i2c::Mock::new(expectations);
        futures::executor::block_on(action(&mut i2c_mock));
        i2c_mock.done();
    }

    #[test]
    fn test_new() {
        create_i2c(&[], async |i2c| {
            let mut device = Sgp40::new(i2c, NoopDelay);
            #[cfg(feature = "log")]
            log::info!("Address {}", device.address());
        });
    }

    #[test]
    fn test_heater_disable() {
        let expectations = [Transaction::write(
            DEFAULT_I2C_ADDRESS,
            (0x3615 as u16).to_be_bytes().to_vec(),
        )];

        create_i2c(&expectations, async |i2c| {
            let mut device = Sgp40::new(i2c, NoopDelay);
            assert!(device.heater_disable().await.is_ok());
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
            Transaction::write(DEFAULT_I2C_ADDRESS, (0x3682 as u16).to_be_bytes().to_vec()),
            Transaction::read(DEFAULT_I2C_ADDRESS, serial_number.to_vec()),
        ];

        create_i2c(&expectations, async |i2c| {
            let mut device = Sgp40::new(i2c, NoopDelay);
            assert_eq!(
                device.serial_number_fetch().await.unwrap(),
                raw_serial_number
            );
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
            Transaction::write(DEFAULT_I2C_ADDRESS, (0x280e as u16).to_be_bytes().to_vec()),
            Transaction::read(DEFAULT_I2C_ADDRESS, test_results.to_vec()),
        ];

        create_i2c(&expectations, async |i2c| {
            let mut device = Sgp40::new(i2c, NoopDelay);
            assert_eq!(
                device.self_test().await.unwrap(),
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

        let mut expected = BytesMut::with_capacity(8);
        expected.put_u16(0x260f);
        expected.put_u16(0x8000);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &(0x8000 as u16).to_be_bytes().to_vec(),
        ));
        expected.put_u16(0x6666);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &(0x6666 as u16).to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(DEFAULT_I2C_ADDRESS, expected.to_vec()),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
        ];

        create_i2c(&expectations, async |i2c| {
            let mut device = Sgp40::new(i2c, NoopDelay);
            assert_matches!(
                device.measure_raw_signals().await,
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

        let mut expected = BytesMut::with_capacity(8);
        expected.put_u16(0x2619);
        expected.put_u16(0x8000);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &(0x8000 as u16).to_be_bytes().to_vec(),
        ));
        expected.put_u16(0x6666);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &(0x6666 as u16).to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(DEFAULT_I2C_ADDRESS, expected.to_vec()),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
        ];

        create_i2c(&expectations, async |i2c| {
            let mut device = Sgp41::new(i2c, NoopDelay);
            assert_matches!(
                device.measure_raw_signals().await,
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
        expected.put_u16(0x260f);
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

        create_i2c(&expectations, async |i2c| {
            let mut device = Sgp40::new(i2c, NoopDelay);
            assert_matches!(
                device
                    .measure_raw_signals_with_compensation(temperature, humidity)
                    .await,
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
        expected.put_u16(0x2619);
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

        create_i2c(&expectations, async |i2c| {
            let mut device = Sgp41::new(i2c, NoopDelay);
            assert_matches!(
                device
                    .measure_raw_signals_with_compensation(temperature, humidity)
                    .await,
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

        let mut expected = BytesMut::with_capacity(8);
        expected.put_u16(0x2612);
        expected.put_u16(0x8000);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &(0x8000 as u16).to_be_bytes().to_vec(),
        ));
        expected.put_u16(0x6666);
        expected.put_u8(sensirion_i2c::crc8::calculate(
            &(0x6666 as u16).to_be_bytes().to_vec(),
        ));

        let expectations = [
            Transaction::write(DEFAULT_I2C_ADDRESS, expected.to_vec()),
            Transaction::read(DEFAULT_I2C_ADDRESS, raw_signals.to_vec()),
        ];

        create_i2c(&expectations, async |i2c| {
            let mut device = Sgp41::new(i2c, NoopDelay);
            assert_matches!(
                device.execute_conditioning().await,
                Ok(RawMeasurement::Partial { voc: 44 })
            );
        });
    }
}
