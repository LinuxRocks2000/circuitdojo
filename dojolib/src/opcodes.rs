/*
Copyright 2025 Tyler Clarke

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS “AS IS” AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

*/
pub(crate) mod mosi {
    pub(crate) const PLEASE_ESTABLISH: u8 = 0xFF;
    pub(crate) const REQUEST_BOARD_PARAMETERS: u8 = 0x80;
    pub(crate) const SET_PIN_MODE_INPUT: u8 = 0x81;
    pub(crate) const SET_PIN_MODE_OUTPUT: u8 = 0x82;
    pub(crate) const DISABLE_PIN: u8 = 0x83;
    pub(crate) const SUBSCRIBE: u8 = 0x84;
    pub(crate) const ANALOG_WRITE: u8 = 0x85;
    pub(crate) const RUN_ONE_SAMPLE: u8 = 0x86;
}

pub(crate) mod miso {
    pub(crate) const ERROR: u8 = 0xFE;
    pub(crate) const ACK: u8 = 0xFF;
    pub(crate) const SAMPLING_BOUNDS: u8 = 0x80;
    pub(crate) const PIN_DESCRIPTION: u8 = 0x81;
    pub(crate) const BOARD_DESCRIPTION: u8 = 0x82;
    pub(crate) const ANALOG_SAMPLE: u8 = 0x83; // 0x83 <pin number> <sample value low byte> <sample value high byte>
}
