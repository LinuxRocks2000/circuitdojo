pub(crate) mod mosi {
    pub(crate)const PLEASE_ESTABLISH: u8 = 0xFF;
    pub(crate)const REQUEST_BOARD_PARAMETERS: u8 = 0x80;
    pub(crate)const SET_PIN_MODE_INPUT: u8 = 0x81;
    pub(crate)const SET_PIN_MODE_OUTPUT: u8 = 0x82;
    pub(crate)const RUN_ONE_SAMPLE: u8 = 0x86;
}

pub(crate) mod miso {
    pub(crate) const ERROR: u8 = 0xFE;
    pub(crate) const ACK: u8 = 0xFF;
    pub(crate) const SAMPLING_BOUNDS: u8 = 0x80;
    pub(crate) const PIN_DESCRIPTION: u8 = 0x81;
    pub(crate) const BOARD_DESCRIPTION: u8 = 0x82;
}