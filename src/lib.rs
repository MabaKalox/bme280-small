#![no_std]

use embedded_hal::blocking::delay::DelayMs;
use fixed::types::{U22F10, U24F8};

pub mod submodules;
use submodules::registers::{
    Calib00_25, Calib00_25Arr, Calib26_41, Calib26_41Arr, Id, RegAddr, RegSize, Reset,
};

pub struct CalibData {
    pub dig_t1: u16,
    pub dig_t2: i16,
    pub dig_t3: i16,
    pub dig_p1: u16,
    pub dig_p2: i16,
    pub dig_p3: i16,
    pub dig_p4: i16,
    pub dig_p5: i16,
    pub dig_p6: i16,
    pub dig_p7: i16,
    pub dig_p8: i16,
    pub dig_p9: i16,
    pub dig_h1: u8,
    pub dig_h2: i16,
    pub dig_h3: u8,
    pub dig_h4: i16,
    pub dig_h5: i16,
    pub dig_h6: i8,
}

impl CalibData {
    pub fn new<A1: AsRef<[u8]>, A2: AsRef<[u8]>>(
        calib00_25: Calib00_25<A1>,
        calib26_41: Calib26_41<A2>,
    ) -> Self {
        Self {
            dig_t1: calib00_25.get_dig_t1(),
            dig_t2: calib00_25.get_dig_t2(),
            dig_t3: calib00_25.get_dig_t3(),
            dig_p1: calib00_25.get_dig_p1(),
            dig_p2: calib00_25.get_dig_p2(),
            dig_p3: calib00_25.get_dig_p3(),
            dig_p4: calib00_25.get_dig_p4(),
            dig_p5: calib00_25.get_dig_p5(),
            dig_p6: calib00_25.get_dig_p6(),
            dig_p7: calib00_25.get_dig_p7(),
            dig_p8: calib00_25.get_dig_p8(),
            dig_p9: calib00_25.get_dig_p9(),
            dig_h1: calib00_25.get_dig_h1(),
            dig_h2: calib26_41.get_dig_h2(),
            dig_h3: calib26_41.get_dig_h3(),
            dig_h4: calib26_41.get_dig_h4(),
            dig_h5: calib26_41.get_dig_h5(),
            dig_h6: calib26_41.get_dig_h6(),
        }
    }
}

pub trait RegRead {
    type Error;

    fn reg_read(&mut self, dev_addr: u8, reg_addr: u8, buf: &mut [u8]) -> Result<(), Self::Error>;
}

pub trait RegWrite {
    type Error;

    fn reg_write(&mut self, dev_addr: u8, reg_addr: u8, data: u8) -> Result<(), Self::Error>;
}

pub enum Bme280Error<InterfaceE> {
    Inteface(InterfaceE),
    IdDoesNotMatch,
}

pub struct Bme280<InterfaceT: RegRead + RegWrite, DelayT> {
    interface: InterfaceT,
    dev_addr: u8,
    calib_data: CalibData,
    delay: DelayT,
}

impl<InterfaceT, InterfaceE, DelayT> Bme280<InterfaceT, DelayT>
where
    InterfaceT: RegRead<Error = InterfaceE> + RegWrite<Error = InterfaceE>,
    DelayT: DelayMs<u16>,
{
    pub fn new(
        mut interface: InterfaceT,
        dev_addr: u8,
        mut delay: DelayT,
    ) -> Result<Self, Bme280Error<InterfaceE>> {
        Self::reset(&mut interface, dev_addr, &mut delay)?;

        if Self::read_id(&mut interface, dev_addr)? != Id::BME280_STANDARD_ID {
            return Err(Bme280Error::IdDoesNotMatch);
        }

        Ok(Self {
            calib_data: Self::read_calib(&mut interface, dev_addr)?,
            dev_addr,
            interface,
            delay,
        })
    }

    fn read_calib(
        inteface: &mut InterfaceT,
        dev_addr: u8,
    ) -> Result<CalibData, Bme280Error<InterfaceE>> {
        let mut calib00_25 = Calib00_25([0; Calib00_25Arr::REG_SIZE]);
        let mut calib26_41 = Calib26_41([0; Calib26_41Arr::REG_SIZE]);

        inteface
            .reg_read(dev_addr, Calib00_25Arr::START_ADDR, &mut calib00_25.0)
            .map_err(Bme280Error::Inteface)?;
        inteface
            .reg_read(dev_addr, Calib26_41Arr::START_ADDR, &mut calib26_41.0)
            .map_err(Bme280Error::Inteface)?;

        Ok(CalibData::new(calib00_25, calib26_41))
    }

    fn reset(
        interface: &mut InterfaceT,
        dev_addr: u8,
        delay: &mut DelayT,
    ) -> Result<(), Bme280Error<InterfaceE>> {
        let mut reset = Reset(0);
        reset.set_reset(Reset::RESET_BYTE);
        interface
            .reg_write(dev_addr, Reset::START_ADDR, reset.0)
            .map_err(Bme280Error::Inteface)?;
        delay.delay_ms(10);

        Ok(())
    }

    fn read_id(interface: &mut InterfaceT, dev_addr: u8) -> Result<u8, Bme280Error<InterfaceE>> {
        let mut buf = [0];
        interface
            .reg_read(dev_addr, Id::START_ADDR, &mut buf)
            .map_err(Bme280Error::Inteface)?;

        Ok(Id(buf[0]).get_id())
    }

    // From BME 280 datasheet page 25
    fn compensate_t(calib_data: &CalibData, adc_t: i32) -> (i32, i32) {
        let var1 =
            (((adc_t >> 3) - ((calib_data.dig_t1 as i32) << 1)) * (calib_data.dig_t2 as i32)) >> 11;
        let var2 = (((((adc_t >> 4) - (calib_data.dig_t1 as i32))
            * ((adc_t >> 4) - (calib_data.dig_t1 as i32)))
            >> 12)
            * (calib_data.dig_t3 as i32))
            >> 14;
        let t_fine = var1 + var2;
        let t = (t_fine * 5 + 128) >> 8;

        (t_fine, t)
    }

    // From BME 280 datasheet page 25
    fn compensate_p(calib_data: &CalibData, t_fine: i32, adc_p: i32) -> U24F8 {
        let mut var1 = (t_fine as i64) - 128000;
        let mut var2 = var1 * var1 * calib_data.dig_p6 as i64;
        var2 += (var1 * calib_data.dig_p5 as i64) << 17;
        var2 += (calib_data.dig_p4 as i64) << 35;
        var1 = ((var1 * var1 * calib_data.dig_p3 as i64) >> 8)
            + ((var1 * calib_data.dig_p2 as i64) << 12);
        var1 = (((1 << 47) + var1) * calib_data.dig_p1 as i64) >> 33;
        if var1 == 0 {
            return U24F8::ZERO; // avoid exception caused by division by zero
        }
        let mut p = 1048576 - adc_p as i64;
        p = (((p << 31) - var2) * 3125) / var1;
        var1 = ((calib_data.dig_p9 as i64) * (p >> 13) * (p >> 13)) >> 25;
        var2 = ((calib_data.dig_p8 as i64) * p) >> 19;
        p = ((p + var1 + var2) >> 8) + ((calib_data.dig_p7 as i64) << 4);

        U24F8::from_bits(p as u32)
    }

    // From BME 280 datasheet page 25
    fn compensate_h(calib_data: &CalibData, t_fine: i32, adc_h: i32) -> U22F10 {
        let mut val = t_fine - 76800;
        val = ((((adc_h << 14)
            - ((calib_data.dig_h4 as i32) << 20)
            - ((calib_data.dig_h5 as i32) * val))
            + (16384))
            >> 15)
            * (((((((val * (calib_data.dig_h6 as i32)) >> 10)
                * (((val * (calib_data.dig_h3 as i32)) >> 11) + (32768)))
                >> 10)
                + (2097152))
                * (calib_data.dig_h2 as i32)
                + 8192)
                >> 14);
        val = val - (((((val >> 15) * (val >> 15)) >> 7) * (calib_data.dig_h1 as i32)) >> 4);
        val = val.clamp(0, 419430400);
        val >>= 12;
        U22F10::from_bits(val as u32)
    }

    pub fn get_calib(&self) -> &CalibData {
        &self.calib_data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const MOCK_CALIB_DATA: &CalibData = &CalibData {
        dig_t1: 28765,
        dig_t2: 27210,
        dig_t3: 50,
        dig_p1: 37234,
        dig_p2: -10553,
        dig_p3: 3024,
        dig_p4: 7263,
        dig_p5: 31,
        dig_p6: -7,
        dig_p7: 9900,
        dig_p8: -10230,
        dig_p9: 4285,
        dig_h1: 75,
        dig_h2: 373,
        dig_h3: 0,
        dig_h4: 293,
        dig_h5: 50,
        dig_h6: 30,
    };

    struct DummyInterface {}
    struct DummyDelay {}

    impl RegRead for DummyInterface {
        type Error = ();

        fn reg_read(
            &mut self,
            _dev_addr: u8,
            _reg_addr: u8,
            _buf: &mut [u8],
        ) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    impl RegWrite for DummyInterface {
        type Error = ();

        fn reg_write(&mut self, dev_addr: u8, reg_addr: u8, data: u8) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    impl DelayMs<u16> for DummyDelay {
        fn delay_ms(&mut self, ms: u16) {
            unimplemented!()
        }
    }

    #[test]
    fn test_compensate_t() {
        // Magic numbers obtained by dumping values from proofed to work bme280 lib
        let mock_adc_t = 526514;
        let expected_t_fine = 110074;
        let expected_t = 2150;

        let (t_fine, t) =
            Bme280::<DummyInterface, DummyDelay>::compensate_t(MOCK_CALIB_DATA, mock_adc_t);
        assert_eq!(expected_t_fine, t_fine);
        assert_eq!(expected_t, t);
    }

    #[test]
    fn test_compensate_p() {
        // Magic numbers obtained by dumping values from proofed to work bme280 lib
        let mock_adc_p = 322858;
        let mock_t_fine = 120188;
        let expected_p = U24F8::from_bits(26110518);

        let p = Bme280::<DummyInterface, DummyDelay>::compensate_p(
            MOCK_CALIB_DATA,
            mock_t_fine,
            mock_adc_p,
        );
        assert_eq!(expected_p, p);
    }

    #[test]
    fn test_compensate_h() {
        // Magic numbers obtained by dumping values from proofed to work bme280 lib
        let mock_adc_h = 23549;
        let mock_t_fine = 99523;
        let expected_h = U22F10::from_bits(27726);

        let p = Bme280::<DummyInterface, DummyDelay>::compensate_h(
            MOCK_CALIB_DATA,
            mock_t_fine,
            mock_adc_h,
        );
        assert_eq!(expected_h, p);
    }
}
