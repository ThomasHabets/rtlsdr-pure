use std::fmt;

use crate::TunerKind;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Usb(cross_usb::usb::Error),
    #[cfg(target_family = "wasm")]
    WebUsb(String),
    DeviceNotFound,
    TunerNotFound,
    UnsupportedTuner(TunerKind),
    InvalidSampleRate(u32),
    InvalidFrequency(u32),
    InvalidGain(i32),
    InvalidTransferLength(usize),
    ShortTransfer {
        expected: usize,
        actual: usize,
    },
    PllDidNotLock,
    ArithmeticOverflow,
    ControlTransfer {
        operation: &'static str,
        value: u16,
        index: u16,
        source: Box<Error>,
    },
    Operation {
        operation: &'static str,
        source: Box<Error>,
    },
}

impl From<cross_usb::usb::Error> for Error {
    fn from(value: cross_usb::usb::Error) -> Self {
        Self::Usb(value)
    }
}

#[cfg(target_family = "wasm")]
impl From<js_sys::Error> for Error {
    fn from(value: js_sys::Error) -> Self {
        Self::WebUsb(value.message().into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usb(cross_usb::usb::Error::CommunicationError(message)) => {
                write!(f, "USB communication failed: {message}")
            }
            Self::Usb(err) => write!(f, "USB error: {err}"),
            #[cfg(target_family = "wasm")]
            Self::WebUsb(message) => write!(f, "WebUSB error: {message}"),
            Self::DeviceNotFound => f.write_str("no known RTL-SDR device found"),
            Self::TunerNotFound => f.write_str("no supported tuner was detected"),
            Self::UnsupportedTuner(tuner) => write!(f, "unsupported tuner: {tuner:?}"),
            Self::InvalidSampleRate(rate) => write!(f, "invalid RTL2832 sample rate: {rate} Hz"),
            Self::InvalidFrequency(freq) => write!(f, "invalid frequency: {freq} Hz"),
            Self::InvalidGain(gain) => write!(f, "invalid tuner gain: {gain} tenths of a dB"),
            Self::InvalidTransferLength(len) => {
                write!(f, "USB control transfer length is too large: {len}")
            }
            Self::ShortTransfer { expected, actual } => {
                write!(
                    f,
                    "short USB transfer: expected {expected} bytes, got {actual}"
                )
            }
            Self::PllDidNotLock => f.write_str("R82xx tuner PLL did not lock"),
            Self::ArithmeticOverflow => f.write_str("integer overflow while configuring device"),
            Self::ControlTransfer {
                operation,
                value,
                index,
                source,
            } => write!(
                f,
                "{operation} failed (value=0x{value:04x}, index=0x{index:04x}): {source}"
            ),
            Self::Operation { operation, source } => {
                write!(f, "{operation} failed: {source}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Usb(err) => Some(err),
            Self::ControlTransfer { source, .. } | Self::Operation { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl Error {
    pub(crate) fn context(self, operation: &'static str) -> Self {
        Self::Operation {
            operation,
            source: Box::new(self),
        }
    }

    pub(crate) fn control(self, operation: &'static str, value: u16, index: u16) -> Self {
        Self::ControlTransfer {
            operation,
            value,
            index,
            source: Box::new(self),
        }
    }
}

pub(crate) trait ResultExt<T> {
    fn context(self, operation: &'static str) -> Result<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn context(self, operation: &'static str) -> Result<T> {
        self.map_err(|err| err.context(operation))
    }
}
