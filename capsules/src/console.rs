//! Console Capsule
//!
//! Console provides userspace with the ability to print text via a serial
//! interface.

use core::cell::Cell;
use kernel::{AppId, AppSlice, Container, Callback, Shared, Driver, ReturnCode};
use kernel::common::take_cell::TakeCell;
use kernel::hil::uart::{self, UART, Client};
use kernel::process::Error;

pub struct App {
    write_callback: Option<Callback>,
    read_buffer: Option<AppSlice<Shared, u8>>,
    write_buffer: Option<AppSlice<Shared, u8>>,
    write_len: usize,
    write_remaining: usize, // How many bytes didn't fit in the buffer and still need to be printed.
    pending_write: bool,
    read_idx: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            write_callback: None,
            read_buffer: None,
            write_buffer: None,
            write_len: 0,
            write_remaining: 0,
            pending_write: false,
            read_idx: 0,
        }
    }
}

pub static mut WRITE_BUF: [u8; 64] = [0; 64];

pub struct Console<'a, U: UART + 'a> {
    uart: &'a U,
    apps: Container<App>,
    in_progress: Cell<Option<AppId>>,
    tx_buffer: TakeCell<'static, [u8]>,
    baud_rate: u32,
}

impl<'a, U: UART> Console<'a, U> {
    pub fn new(uart: &'a U,
               baud_rate: u32,
               tx_buffer: &'static mut [u8],
               container: Container<App>)
               -> Console<'a, U> {
        Console {
            uart: uart,
            apps: container,
            in_progress: Cell::new(None),
            tx_buffer: TakeCell::new(tx_buffer),
            baud_rate: baud_rate,
        }
    }

    pub fn initialize(&self) {
        self.uart.init(uart::UARTParams {
            baud_rate: self.baud_rate,
            stop_bits: uart::StopBits::One,
            parity: uart::Parity::None,
            hw_flow_control: false,
        });
    }

    /// Internal helper function for setting up a new send transaction
    fn send_new(&self, app_id: AppId, app: &mut App, callback: Callback) -> ReturnCode {
        match app.write_buffer.take() {
            Some(slice) => {
                app.write_len = slice.len();
                app.write_remaining = app.write_len;
                app.write_callback = Some(callback);
                self.send(app_id, app, slice);
                ReturnCode::SUCCESS
            }
            None => ReturnCode::EBUSY,
        }
    }

    /// Internal helper function for continuing a previously set up transaction
    /// Returns true if this send is still active, or false if it has completed
    fn send_continue(&self, app_id: AppId, app: &mut App) -> Result<bool, ReturnCode> {
        if app.write_remaining > 0 {
            match app.write_buffer.take() {
                Some(slice) => {
                    self.send(app_id, app, slice);
                    Ok(true)
                }
                None => Err(ReturnCode::FAIL),
            }
        } else {
            Ok(false)
        }
    }

    /// Internal helper function for sending data for an existing transaction.
    /// Cannot fail. If can't send now, it will schedule for sending later.
    fn send(&self, app_id: AppId, app: &mut App, slice: AppSlice<Shared, u8>) {
        if self.in_progress.get().is_none() {
            self.in_progress.set(Some(app_id));
            self.tx_buffer.take().map(|buffer| {
                let mut transaction_len = app.write_remaining;
                for (i, c) in slice.as_ref()[slice.len() - app.write_remaining..slice.len()]
                    .iter()
                    .enumerate() {
                    if buffer.len() <= i {
                        break;
                    }
                    buffer[i] = *c;
                }

                // Check if everything we wanted to print
                // fit in the buffer.
                if app.write_remaining > buffer.len() {
                    transaction_len = buffer.len();
                    app.write_remaining -= buffer.len();
                    app.write_buffer = Some(slice);
                } else {
                    app.write_remaining = 0;
                }

                self.uart.transmit(buffer, transaction_len);
            });
        } else {
            app.pending_write = true;
            app.write_buffer = Some(slice);
        }
    }
}

impl<'a, U: UART> Driver for Console<'a, U> {
    fn allow(&self, appid: AppId, allow_num: usize, slice: AppSlice<Shared, u8>) -> ReturnCode {
        match allow_num {
            0 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.read_buffer = Some(slice);
                        app.read_idx = 0;
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }
            1 => {
                self.apps
                    .enter(appid, |app, _| {
                        app.write_buffer = Some(slice);
                        ReturnCode::SUCCESS
                    })
                    .unwrap_or_else(|err| match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    })
            }
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(&self, subscribe_num: usize, callback: Callback) -> ReturnCode {
        match subscribe_num {
            0 /* read line */ => {
                // read line is not implemented for console at this time
                ReturnCode::ENOSUPPORT
            },
            1 /* putstr/write_done */ => {
                self.apps.enter(callback.app_id(), |app, _| {
                    self.send_new(callback.app_id(), app, callback)
                }).unwrap_or_else(|err| {
                    match err {
                        Error::OutOfMemory => ReturnCode::ENOMEM,
                        Error::AddressOutOfBounds => ReturnCode::EINVAL,
                        Error::NoSuchApp => ReturnCode::EINVAL,
                    }
                })
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }

    fn command(&self, cmd_num: usize, arg1: usize, _: AppId) -> ReturnCode {
        match cmd_num {
            0 /* check if present */ => ReturnCode::SUCCESS,
            1 /* putc */ => {
                self.tx_buffer.take().map(|buffer| {
                    buffer[0] = arg1 as u8;
                    self.uart.transmit(buffer, 1);
                });
                ReturnCode::SuccessWithValue { value: 1 }
            },
            _ => ReturnCode::ENOSUPPORT
        }
    }
}

impl<'a, U: UART> Client for Console<'a, U> {
    fn transmit_complete(&self, buffer: &'static mut [u8], _error: uart::Error) {
        // Either print more from the AppSlice or send a callback to the
        // application.
        self.tx_buffer.replace(buffer);
        self.in_progress.get().map(|appid| {
            self.in_progress.set(None);
            self.apps.enter(appid, |app, _| {
                match self.send_continue(appid, app) {
                    Ok(more_to_send) => {
                        if !more_to_send {
                            // Go ahead and signal the application
                            let written = app.write_len;
                            app.write_len = 0;
                            app.write_callback.map(|mut cb| { cb.schedule(written, 0, 0); });
                        }
                    }
                    Err(return_code) => {
                        // XXX This shouldn't ever happen?
                        app.write_len = 0;
                        app.write_remaining = 0;
                        app.pending_write = false;
                        let r0 = isize::from(return_code) as usize;
                        app.write_callback.map(|mut cb| { cb.schedule(r0, 0, 0); });
                    }
                }
            })
        });

        // If we are not printing more from the current AppSlice,
        // see if any other applications have pending messages.
        if self.in_progress.get().is_none() {
            for cntr in self.apps.iter() {
                let started_tx = cntr.enter(|app, _| {
                    if app.pending_write {
                        app.pending_write = false;
                        match self.send_continue(app.appid(), app) {
                            Ok(more_to_send) => more_to_send,
                            Err(return_code) => {
                                // XXX This shouldn't ever happen?
                                app.write_len = 0;
                                app.write_remaining = 0;
                                app.pending_write = false;
                                let r0 = isize::from(return_code) as usize;
                                app.write_callback.map(|mut cb| { cb.schedule(r0, 0, 0); });
                                false
                            }
                        }
                    } else {
                        false
                    }
                });
                if started_tx {
                    break;
                }
            }
        }
    }

    fn receive_complete(&self,
                        _rx_buffer: &'static mut [u8],
                        _rx_len: usize,
                        _error: uart::Error) {
        // this is currently unimplemented for console
    }
}
