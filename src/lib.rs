//! Pure Rust RTL-SDR driver built on top of `cross_usb`.
//!
//! The public API is async because `cross_usb` maps to both native USB and
//! WebUSB. On native targets, use any executor or a small blocking helper in
//! your application; the library itself does not require a runtime.

mod error;
mod r82xx;
mod rtl2832;

pub use error::{Error, Result};
pub use rtl2832::{
    DEFAULT_SAMPLE_RATE, DeviceDescription, GainMode, IqSample, KNOWN_DEVICES, KnownDevice, RtlSdr,
    TunerKind, bytes_to_iq_samples,
};

/// Open the first known RTL2832U-based dongle and initialize it for SDR use.
pub async fn open_first() -> Result<RtlSdr> {
    RtlSdr::open_first().await
}

/// Return currently visible RTL-SDR devices.
///
/// WebUSB cannot enumerate arbitrary unpaired devices, so this helper is only
/// available on native targets. Use [`open_first`] in browser builds; WebUSB
/// will prompt the user if no matching device has already been granted.
#[cfg(not(target_family = "wasm"))]
pub async fn list_devices() -> Result<Vec<DeviceDescription>> {
    RtlSdr::list_devices().await
}
