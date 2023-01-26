use bitfield::bitfield;

pub trait RegAddr {
    const START_ADDR: u8;
    const END_ADDR: u8; // If register spans multiple addressed
}

pub trait RegSize {
    const REG_SIZE: usize;
}

impl<T> RegSize for T
where
    T: RegAddr,
{
    const REG_SIZE: usize = (T::END_ADDR - T::START_ADDR + 1) as usize;
}

bitfield! {
    pub struct Calib00_25([u8]);
    pub u16, get_dig_t1, _: (16-1), 0;
    pub i16, get_dig_t2, _: (16*2-1), 16;
    pub i16, get_dig_t3, _: (16*3-1), 16*2;
    pub u16, get_dig_p1, _: (16*4-1), 16*3;
    pub i16, get_dig_p2, _: (16*5-1), 16*4;
    pub i16, get_dig_p3, _: (16*6-1), 16*5;
    pub i16, get_dig_p4, _: (16*7-1), 16*6;
    pub i16, get_dig_p5, _: (16*8-1), 16*7;
    pub i16, get_dig_p6, _: (16*9-1), 16*8;
    pub i16, get_dig_p7, _: (16*10-1), 16*9;
    pub i16, get_dig_p8, _: (16*11-1), 16*10;
    pub i16, get_dig_p9, _: (16*12-1), 16*11;
    pub u8 , get_dig_h1, _: (16*13-1), 16*12+8; // As this calib is shifted right
    // TODO - check that h1 is actually should be read from 0xA1, not from 0xA2
}
impl<T> RegAddr for Calib00_25<T> {
    const START_ADDR: u8 = 0x88;
    const END_ADDR: u8 = 0xA1;
}
pub type Calib00_25Arr = Calib00_25<[u8; Calib00_25::<&[u8]>::REG_SIZE]>;

bitfield! {
    pub struct Id(u8);
    pub u8, get_id, _: 7, 0;
}
impl RegAddr for Id {
    const START_ADDR: u8 = 0xD0;
    const END_ADDR: u8 = Self::START_ADDR;
}
impl Id {
    pub const BME280_STANDARD_ID: u8 = 0x60;
}

bitfield! {
    pub struct Calib26_41([u8]);
    pub i16, get_dig_h2, _: 15, 0;
    pub u8,  get_dig_h3, _: 23, 16;
    // pub i16, get_dig_h4, _: 35, 24; it is implemented manually as bit layout does not match
    pub i16, get_dig_h5, _: 47, 36;
    pub i8 , get_dig_h6, _: 55, 48;
}
impl<T> RegAddr for Calib26_41<T> {
    const START_ADDR: u8 = 0xE1;
    const END_ADDR: u8 = 0xF0;
}
impl<T: AsRef<[u8]>> Calib26_41<T> {
    // Manually implemented due to weird bit order
    pub fn get_dig_h4(&self) -> i16 {
        let arr = self.0.as_ref();
        (arr[3] as i16) << 4 | (arr[4] & 0x0F) as i16
    }
}
pub type Calib26_41Arr = Calib26_41<[u8; Calib26_41::<&[u8]>::REG_SIZE]>;

bitfield! {
    pub struct Reset(u8);
    pub u8, _, set_reset: 7, 0;
}
impl RegAddr for Reset {
    const START_ADDR: u8 = 0xE0;
    const END_ADDR: u8 = Self::START_ADDR;
}
impl Reset {
    pub const RESET_BYTE: u8 = 0xB6;
}

bitfield! {
    struct CtrlHum(u8);
    u8, get_oversampling, set_oversampling: 2, 0;
}
impl RegAddr for CtrlHum {
    const START_ADDR: u8 = 0xF2;
    const END_ADDR: u8 = Self::START_ADDR;
}

bitfield! {
    struct Status(u8);
    u8, get_im_update, _: 1, 0;
    u8, get_measuring, _: 4, 3;
}
impl RegAddr for Status {
    const START_ADDR: u8 = 0xF3;
    const END_ADDR: u8 = Self::START_ADDR;
}

bitfield! {
    struct CtrlMeas(u8);
    u8, get_mode, set_mode: 1, 0;
    u8, get_press_oversampling, set_press_oversampling: 4, 2;
    u8, get_temp_oversampling, set_temp_oversampling: 7, 5;
}
impl RegAddr for CtrlMeas {
    const START_ADDR: u8 = 0xF4;
    const END_ADDR: u8 = Self::START_ADDR;
}

bitfield! {
    struct Config(u8);
    u8, get_spi3w_en, set_spi3w_en: 0, 0;
    u8, get_filter, set_filter: 4, 2;
    u8, get_t_sb, set_t_sb: 7, 5;
}
impl RegAddr for Config {
    const START_ADDR: u8 = 0xF5;
    const END_ADDR: u8 = Self::START_ADDR;
}

bitfield! {
    struct RawPress(u32);
    u8, get_msb, set_msb: 0, 7;
    u8, get_lsb, set_lsb: 8, 16;
    u8, get_xlsb, set_xlsb: 20, 23;
}
impl RegAddr for RawPress {
    const START_ADDR: u8 = 0xF7;
    const END_ADDR: u8 = 0xF9;
}

bitfield! {
    struct RawTemp(u32);
    u8, get_msb, set_msb: 0, 7;
    u8, get_lsb, set_lsb: 8, 16;
    u8, get_xlsb, set_xlsb: 20, 23;
}
impl RegAddr for RawTemp {
    const START_ADDR: u8 = 0xFA;
    const END_ADDR: u8 = 0xFC;
}

bitfield! {
    struct RawHum(u32);
    u8, get_msb, set_msb: 0, 7;
    u8, get_lsb, set_lsb: 8, 16;
    u8, get_xlsb, set_xlsb: 20, 23;
}
impl RegAddr for RawHum {
    const START_ADDR: u8 = 0xFD;
    const END_ADDR: u8 = 0xFE;
}

#[repr(u8)]
#[derive(num_enum::TryFromPrimitive)]
enum Mode {
    Sleep = 0b00,
    Forced = 0b01,
    ForcedAlt = 0b10, // Same as Forsed
    Normal = 0b11,
}

#[repr(u8)]
enum StandbyPeriod {
    Us500 = 0b000,
    Us62500 = 0b001,
    Ms125 = 0b010,
    Ms250 = 0b011,
    Ms500 = 0b100,
    Ms1000 = 0b101,
    Ms10 = 0b110,
    Ms20 = 0b111,
}

#[repr(u8)]
#[derive(num_enum::TryFromPrimitive)]
enum Oversampling {
    ModuleDisabled = 0b000,
    X1 = 0b001,
    X2 = 0b010,
    X4 = 0b011,
    X8 = 0b100,
    X16 = 0b101,
}

#[repr(u8)]
#[derive(num_enum::TryFromPrimitive)]
enum Filter {
    Off = 0b000,
    C2 = 0b001,
    C4 = 0b010,
    C8 = 0b011,
    C16 = 0b100,
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_status_parse() {
        // Measuring and ImUpdate
        let mock_status = Status(0b00001001);
        assert_eq!(
            (1, 1),
            (mock_status.get_im_update(), mock_status.get_measuring())
        );

        // Measuring and no ImUpdate
        let mock_status = Status(0b00001000);
        assert_eq!(
            (0, 1),
            (mock_status.get_im_update(), mock_status.get_measuring())
        );
    }

    #[test]
    fn test_ctrl_hum_parse() {
        let mock_data = CtrlHum(0b000000000);
        assert!(matches!(
            Oversampling::try_from(mock_data.get_oversampling()).unwrap(),
            Oversampling::ModuleDisabled
        ));

        let mock_data = CtrlHum(0b000000001);
        assert!(matches!(
            Oversampling::try_from(mock_data.get_oversampling()).unwrap(),
            Oversampling::X1
        ));

        let mock_data = CtrlHum(0b000000101);
        assert!(matches!(
            Oversampling::try_from(mock_data.get_oversampling()).unwrap(),
            Oversampling::X16
        ));
    }

    #[test]
    fn test_calib00_25_parse() {
        let mock_data = [
            0x5D, 0x70, 0x4A, 0x6A, 0x32, 0x00, 0x72, 0x91, 0xC7, 0xD6, 0xD0, 0x0B, 0x5F, 0x1C,
            0x1F, 0x00, 0xF9, 0xFF, 0xAC, 0x26, 0x0A, 0xD8, 0xBD, 0x10, 0x00, 0x4B,
        ];

        let calib00_25 = Calib00_25(mock_data);

        assert_eq!(28765, calib00_25.get_dig_t1());
        assert_eq!(27210, calib00_25.get_dig_t2());
        assert_eq!(50, calib00_25.get_dig_t3());
        assert_eq!(37234, calib00_25.get_dig_p1());
        assert_eq!(-10553, calib00_25.get_dig_p2());
        assert_eq!(3024, calib00_25.get_dig_p3());
        assert_eq!(7263, calib00_25.get_dig_p4());
        assert_eq!(31, calib00_25.get_dig_p5());
        assert_eq!(-7, calib00_25.get_dig_p6());
        assert_eq!(9900, calib00_25.get_dig_p7());
        assert_eq!(-10230, calib00_25.get_dig_p8());
        assert_eq!(4285, calib00_25.get_dig_p9());
        assert_eq!(75, calib00_25.get_dig_h1());
    }

    #[test]
    fn test_calib26_41_parse() {
        let mock_data = [
            0x75, 0x01, 0x00, 0x12, 0x25, 0x03, 0x1E, 0x42, 0x41, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF,
        ];

        let calib26_41 = Calib26_41(mock_data);

        assert_eq!(373, calib26_41.get_dig_h2());
        assert_eq!(0, calib26_41.get_dig_h3());
        assert_eq!(293, calib26_41.get_dig_h4());
        assert_eq!(50, calib26_41.get_dig_h5());
        assert_eq!(30, calib26_41.get_dig_h6());
    }

    // TODO - add tests of raw measurements data parse
}
