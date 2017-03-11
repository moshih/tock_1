//! Driver for the ISL29035 digital light sensor

use core::cell::Cell;
use kernel::{AppId, Callback, Driver, ReturnCode};
use kernel::common::take_cell::TakeCell;
use kernel::hil::i2c::{I2CDevice, I2CClient, Error};
use kernel::hil::time::{self, Frequency};

pub static mut BUF: [u8; 3] = [0; 3];

#[derive(Copy,Clone,PartialEq)]
enum State {
    Disabled,
    Enabling,
    Integrating,
    ReadingLI,
    Disabling(usize),
}

pub struct Isl29035<'a, A: time::Alarm + 'a> {
    i2c: &'a I2CDevice,
    alarm: &'a A,
    state: Cell<State>,
    buffer: TakeCell<'static, [u8]>,
    callback: Cell<Option<Callback>>,
}

impl<'a, A: time::Alarm + 'a> Isl29035<'a, A> {
    pub fn new(i2c: &'a I2CDevice, alarm: &'a A, buffer: &'static mut [u8]) -> Isl29035<'a, A> {
        Isl29035 {
            i2c: i2c,
            alarm: alarm,
            state: Cell::new(State::Disabled),
            buffer: TakeCell::new(buffer),
            callback: Cell::new(None),
        }
    }

    pub fn start_read_lux(&self) {
        if self.state.get() == State::Disabled {
            self.buffer.take().map(|buf| {
                self.i2c.enable();
                buf[0] = 0;
                // CMD 1 Register:
                // Interrupt persist for 1 integration cycle (bits 0 & 1)
                // Measure ALS continuously (buts 5,6 & 7)
                // Bit 2 is the interrupt bit
                // Bits 3 & 4 are reserved
                buf[1] = 0b10100000;

                // CMD 2 Register:
                // Range 4000 (bits 0, 1)
                // ADC resolution 8-bit (bits 2,3)
                // Other bits are reserved
                buf[2] = 0b00001001;
                self.i2c.write(buf, 3);
                self.state.set(State::Enabling);
            });
        }
    }
}

impl<'a, A: time::Alarm + 'a> Driver for Isl29035<'a, A> {
    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 => {
                self.callback.set(Some(callback));
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn command(&self, command_num: usize, _arg1: usize, _: AppId) -> ReturnCode {
        match command_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 => {
                self.start_read_lux();
                ReturnCode::SUCCESS
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

impl<'a, A: time::Alarm + 'a> time::Client for Isl29035<'a, A> {
    fn fired(&self) {
        self.buffer.take().map(|buffer| {
            // Turn on i2c to send commands.
            self.i2c.enable();

            buffer[0] = 0x02 as u8;
            self.i2c.write_read(buffer, 1, 2);
            self.state.set(State::ReadingLI);
        });
    }
}

impl<'a, A: time::Alarm + 'a> I2CClient for Isl29035<'a, A> {
    fn command_complete(&self, buffer: &'static mut [u8], _error: Error) {
        // TODO(alevy): handle I2C errors
        match self.state.get() {
            State::Enabling => {
                // Set a timer to wait for the conversion to be done.
                // For 8 bits, thats 410 us (per Table 11 in the datasheet).
                let interval = (410 as u32) * <A::Frequency>::frequency() / 1000000;
                let tics = self.alarm.now().wrapping_add(interval);
                self.alarm.set_alarm(tics);

                // Now wait for timer to expire
                self.buffer.replace(buffer);
                self.i2c.disable();
                self.state.set(State::Integrating);
            }
            State::ReadingLI => {
                // During configuration we set the ADC resolution to 8 bits and
                // the range to 4000.
                //
                // Since it's only 8 bits, we ignore the second byte of output.
                //
                // For a given Range and n (-bits of ADC resolution):
                // Lux = Data * (Range / 2^n)
                let data = buffer[0] as usize; //((buffer[1] as usize) << 8) | buffer[0] as usize;
                let lux = (data * 4000) >> 8;

                buffer[0] = 0;
                self.i2c.write(buffer, 2);
                self.state.set(State::Disabling(lux));
            }
            State::Disabling(lux) => {
                self.i2c.disable();
                self.state.set(State::Disabled);
                self.buffer.replace(buffer);
                self.callback.get().map(|mut cb| cb.schedule(lux, 0, 0));
            }
            _ => {}
        }
    }
}
