use crate::{RegRead, RegWrite};
use embedded_hal::blocking::i2c::{Write, WriteRead};

impl<T: WriteRead> RegRead for T {
    type Error = T::Error;

    fn reg_read(&mut self, dev_addr: u8, reg_addr: u8, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.write_read(dev_addr, &[reg_addr], buf)
    }
}

impl<T: Write> RegWrite for T {
    type Error = T::Error;

    fn reg_write(&mut self, dev_addr: u8, reg_addr: u8, data: u8) -> Result<(), Self::Error> {
        self.write(dev_addr, &[reg_addr, data])
    }
}
