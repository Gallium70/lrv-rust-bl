use core::convert::Infallible;
use core::ptr::{read_volatile, write_volatile};
use embedded_hal::serial::{Read, Write};

pub struct Uartlite {
    base: usize,
    shift: usize,
}

impl Uartlite {
    pub fn new(base: usize, shift: usize) -> Self {
        // init process; ref: MeowSBI/utils/uart.rs
        unsafe {
            write_volatile(
                (base + (offsets::CTRL_REG << shift)) as *mut u8,
                masks::RST_FIFO as u8,
            );
        }
        // init finished
        Self { base, shift }
    }
}

impl Read<u8> for Uartlite {
    // 其实是可能出错的，overrun啊，这些
    type Error = Infallible;

    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        let pending =
            unsafe { read_volatile((self.base + (offsets::STAT_REG << self.shift)) as *const u8) }
                & masks::RX_VALID;
        if pending != 0 {
            let word = unsafe {
                read_volatile((self.base + (offsets::RX_FIFO << self.shift)) as *const u8)
            };
            Ok(word)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl Write<u8> for Uartlite {
    type Error = Infallible;

    fn try_write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        if word == ('\n' as u8) {
            if let Err(e) = self.try_write('\r' as u8) {
                return Err(e);
            }
        }
        // 写，但是不刷新
        unsafe {
            let mut pending =
                read_volatile((self.base + (offsets::STAT_REG << self.shift)) as *const u8)
                    & masks::TX_FULL;
            while pending != 0 {
                pending =
                    read_volatile((self.base + (offsets::STAT_REG << self.shift)) as *const u8)
                        & masks::TX_FULL;
            }
            write_volatile(
                (self.base + (offsets::TX_FIFO << self.shift)) as *mut u8,
                word,
            )
        };
        Ok(())
    }

    fn try_flush(&mut self) -> nb::Result<(), Self::Error> {
        let pending =
            unsafe { read_volatile((self.base + (offsets::STAT_REG << self.shift)) as *const u8) }
                & masks::TX_FULL;
        if pending == 0 {
            // 发送已经结束了
            Ok(())
        } else {
            // 发送还没有结束，继续等
            Err(nb::Error::WouldBlock)
        }
    }
}

mod offsets {
    pub const RX_FIFO: usize = 0x0;
    pub const TX_FIFO: usize = 0x4;
    pub const STAT_REG: usize = 0x8;
    pub const CTRL_REG: usize = 0xc;
}

mod masks {
    pub const RST_FIFO: u8 = 0x03;
    // pub const INTR_EN: u8 = 0x10;
    pub const TX_FULL: u8 = 0x08;
    // pub const TX_EMPTY: u8 = 0x04;
    // pub const RX_FULL: u8 = 0x02;
    pub const RX_VALID: u8 = 0x01;
}
