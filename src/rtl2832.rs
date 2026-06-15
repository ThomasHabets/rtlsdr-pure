use cross_usb::prelude::*;
use cross_usb::usb::{ControlIn, ControlOut, ControlType, Recipient};

use crate::error::{Error, Result, ResultExt};
use crate::r82xx::{R82XX_CHECK_ADDR, R82XX_CHECK_VAL, R82XX_IF_FREQ, R82xx, R82xxKind};

pub const DEFAULT_SAMPLE_RATE: u32 = 2_048_000;

const DEF_RTL_XTAL_FREQ: u32 = 28_800_000;
const DEFAULT_BULK_ENDPOINT: u8 = 0x81;
const DEFAULT_READ_LENGTH: usize = 16 * 32 * 512;

const E4K_I2C_ADDR: u8 = 0xc8;
const E4K_CHECK_ADDR: u8 = 0x02;
const E4K_CHECK_VAL: u8 = 0x40;

const FC0012_I2C_ADDR: u8 = 0xc6;
const FC0012_CHECK_ADDR: u8 = 0x00;
const FC0012_CHECK_VAL: u8 = 0xa1;

const FC0013_I2C_ADDR: u8 = 0xc6;
const FC0013_CHECK_ADDR: u8 = 0x00;
const FC0013_CHECK_VAL: u8 = 0xa3;

const FC2580_I2C_ADDR: u8 = 0xac;
const FC2580_CHECK_ADDR: u8 = 0x01;
const FC2580_CHECK_VAL: u8 = 0x56;

const R820T_I2C_ADDR: u8 = 0x34;
const R828D_I2C_ADDR: u8 = 0x74;
const R828D_XTAL_FREQ: u32 = 16_000_000;

const USB_SYSCTL: u16 = 0x2000;
const USB_EPA_CTL: u16 = 0x2148;
const USB_EPA_MAXPKT: u16 = 0x2158;

const DEMOD_CTL: u16 = 0x3000;
const GPO: u16 = 0x3001;
const GPOE: u16 = 0x3003;
const GPD: u16 = 0x3004;
const DEMOD_CTL_1: u16 = 0x300b;

const USBB: u8 = 1;
const SYSB: u8 = 2;
const IICB: u8 = 6;

const FIR_DEFAULT: [i16; 16] = [
    -54, -36, -41, -40, -32, -14, 14, 53, 101, 156, 215, 273, 327, 372, 404, 421,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownDevice {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: &'static str,
}

pub const KNOWN_DEVICES: &[KnownDevice] = &[
    KnownDevice {
        vendor_id: 0x0bda,
        product_id: 0x2832,
        name: "Generic RTL2832U",
    },
    KnownDevice {
        vendor_id: 0x0bda,
        product_id: 0x2838,
        name: "Generic RTL2832U OEM",
    },
    KnownDevice {
        vendor_id: 0x0413,
        product_id: 0x6680,
        name: "DigitalNow Quad DVB-T PCI-E card",
    },
    KnownDevice {
        vendor_id: 0x0413,
        product_id: 0x6f0f,
        name: "Leadtek WinFast DTV Dongle mini D",
    },
    KnownDevice {
        vendor_id: 0x0458,
        product_id: 0x707f,
        name: "Genius TVGo DVB-T03 USB dongle (Ver. B)",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00a9,
        name: "Terratec Cinergy T Stick Black (rev 1)",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00b3,
        name: "Terratec NOXON DAB/DAB+ USB dongle (rev 1)",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00b4,
        name: "Terratec Deutschlandradio DAB Stick",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00b5,
        name: "Terratec NOXON DAB Stick - Radio Energy",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00b7,
        name: "Terratec Media Broadcast DAB Stick",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00b8,
        name: "Terratec BR DAB Stick",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00b9,
        name: "Terratec WDR DAB Stick",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00c0,
        name: "Terratec MuellerVerlag DAB Stick",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00c6,
        name: "Terratec Fraunhofer DAB Stick",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00d3,
        name: "Terratec Cinergy T Stick RC (Rev.3)",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00d7,
        name: "Terratec T Stick PLUS",
    },
    KnownDevice {
        vendor_id: 0x0ccd,
        product_id: 0x00e0,
        name: "Terratec NOXON DAB/DAB+ USB dongle (rev 2)",
    },
    KnownDevice {
        vendor_id: 0x1554,
        product_id: 0x5020,
        name: "PixelView PV-DT235U(RN)",
    },
    KnownDevice {
        vendor_id: 0x15f4,
        product_id: 0x0131,
        name: "Astrometa DVB-T/DVB-T2",
    },
    KnownDevice {
        vendor_id: 0x15f4,
        product_id: 0x0133,
        name: "HanfTek DAB+FM+DVB-T",
    },
    KnownDevice {
        vendor_id: 0x185b,
        product_id: 0x0620,
        name: "Compro Videomate U620F",
    },
    KnownDevice {
        vendor_id: 0x185b,
        product_id: 0x0650,
        name: "Compro Videomate U650F",
    },
    KnownDevice {
        vendor_id: 0x185b,
        product_id: 0x0680,
        name: "Compro Videomate U680F",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd393,
        name: "GIGABYTE GT-U7300",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd394,
        name: "DIKOM USB-DVBT HD",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd395,
        name: "Peak 102569AGPK",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd397,
        name: "KWorld KW-UB450-T USB DVB-T Pico TV",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd398,
        name: "Zaapa ZT-MINDVBZP",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd39d,
        name: "SVEON STV20 DVB-T USB & FM",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd3a4,
        name: "Twintech UT-40",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd3a8,
        name: "ASUS U3100MINI_PLUS_V2",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd3af,
        name: "SVEON STV27 DVB-T USB & FM",
    },
    KnownDevice {
        vendor_id: 0x1b80,
        product_id: 0xd3b0,
        name: "SVEON STV21 DVB-T USB & FM",
    },
    KnownDevice {
        vendor_id: 0x1d19,
        product_id: 0x1101,
        name: "Dexatek DK DVB-T Dongle (Logilink VG0002A)",
    },
    KnownDevice {
        vendor_id: 0x1d19,
        product_id: 0x1102,
        name: "Dexatek DK DVB-T Dongle (MSI DigiVox mini II V3.0)",
    },
    KnownDevice {
        vendor_id: 0x1d19,
        product_id: 0x1103,
        name: "Dexatek Technology Ltd. DK 5217 DVB-T Dongle",
    },
    KnownDevice {
        vendor_id: 0x1d19,
        product_id: 0x1104,
        name: "MSI DigiVox Micro HD",
    },
    KnownDevice {
        vendor_id: 0x1f4d,
        product_id: 0xa803,
        name: "Sweex DVB-T USB",
    },
    KnownDevice {
        vendor_id: 0x1f4d,
        product_id: 0xb803,
        name: "GTek T803",
    },
    KnownDevice {
        vendor_id: 0x1f4d,
        product_id: 0xc803,
        name: "Lifeview LV5TDeluxe",
    },
    KnownDevice {
        vendor_id: 0x1f4d,
        product_id: 0xd286,
        name: "MyGica TD312",
    },
    KnownDevice {
        vendor_id: 0x1f4d,
        product_id: 0xd803,
        name: "PROlectrix DV107669",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDescription {
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: Option<&'static str>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunerKind {
    Unknown,
    E4000,
    Fc0012,
    Fc0013,
    Fc2580,
    R820T,
    R828D,
}

impl TunerKind {
    pub fn is_supported(self) -> bool {
        matches!(self, Self::R820T | Self::R828D)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GainMode {
    Auto,
    ManualTenthsDb(i32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IqSample {
    pub i: f32,
    pub q: f32,
}

pub struct RtlSdr {
    usb: RtlUsb,
    vendor_id: u16,
    product_id: u16,
    manufacturer: Option<String>,
    product: Option<String>,
    tuner: TunerState,
    rtl_xtal_hz: u32,
    sample_rate_hz: u32,
    center_freq_hz: Option<u32>,
    bandwidth_hz: Option<u32>,
    freq_correction_ppm: i32,
}

enum TunerState {
    Unknown,
    Unsupported(TunerKind),
    R82xx(R82xx),
}

impl RtlSdr {
    pub async fn open_first() -> Result<Self> {
        #[cfg(not(target_family = "wasm"))]
        {
            let infos = cross_usb::get_device_list(device_filters()).await?;
            for info in infos {
                let vendor_id = info.vendor_id().await;
                let product_id = info.product_id().await;
                if known_device_name(vendor_id, product_id).is_some() {
                    return Self::open(info).await;
                }
            }
            Err(Error::DeviceNotFound)
        }

        #[cfg(target_family = "wasm")]
        {
            let info = cross_usb::get_device(device_filters()).await?;
            Self::open(info).await
        }
    }

    #[cfg(not(target_family = "wasm"))]
    pub async fn list_devices() -> Result<Vec<DeviceDescription>> {
        let infos = cross_usb::get_device_list(device_filters()).await?;
        let mut devices = Vec::new();

        for info in infos {
            let vendor_id = info.vendor_id().await;
            let product_id = info.product_id().await;
            let name = known_device_name(vendor_id, product_id);
            if name.is_none() {
                continue;
            }
            devices.push(DeviceDescription {
                vendor_id,
                product_id,
                name,
                manufacturer: info.manufacturer_string().await,
                product: info.product_string().await,
            });
        }

        Ok(devices)
    }

    pub async fn open(info: cross_usb::DeviceInfo) -> Result<Self> {
        let vendor_id = info.vendor_id().await;
        let product_id = info.product_id().await;
        let manufacturer = info.manufacturer_string().await;
        let product = info.product_string().await;
        let device = info.open().await?;
        let interface = device.detach_and_open_interface(0).await?;

        let mut sdr = Self {
            usb: RtlUsb {
                _device: device,
                interface,
            },
            vendor_id,
            product_id,
            manufacturer,
            product,
            tuner: TunerState::Unknown,
            rtl_xtal_hz: DEF_RTL_XTAL_FREQ,
            sample_rate_hz: 0,
            center_freq_hz: None,
            bandwidth_hz: None,
            freq_correction_ppm: 0,
        };

        sdr.usb
            .write_reg(USBB, USB_SYSCTL, 0x09, 1)
            .await
            .context("initial USB_SYSCTL write")?;
        sdr.init_baseband()
            .await
            .context("RTL2832 baseband initialization")?;
        sdr.probe_tuner().await.context("tuner probe")?;
        sdr.set_sample_rate(DEFAULT_SAMPLE_RATE)
            .await
            .context("default sample-rate setup")?;
        sdr.reset_buffer().await.context("USB FIFO reset")?;

        Ok(sdr)
    }

    pub fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    pub fn product_id(&self) -> u16 {
        self.product_id
    }

    pub fn known_name(&self) -> Option<&'static str> {
        known_device_name(self.vendor_id, self.product_id)
    }

    pub fn manufacturer(&self) -> Option<&str> {
        self.manufacturer.as_deref()
    }

    pub fn product(&self) -> Option<&str> {
        self.product.as_deref()
    }

    pub fn tuner_kind(&self) -> TunerKind {
        match &self.tuner {
            TunerState::Unknown => TunerKind::Unknown,
            TunerState::Unsupported(kind) => *kind,
            TunerState::R82xx(tuner) => match tuner.kind() {
                R82xxKind::R820T => TunerKind::R820T,
                R82xxKind::R828D => TunerKind::R828D,
            },
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate_hz
    }

    pub fn center_frequency(&self) -> Option<u32> {
        self.center_freq_hz
    }

    pub fn frequency_correction(&self) -> i32 {
        self.freq_correction_ppm
    }

    pub async fn set_sample_rate(&mut self, sample_rate_hz: u32) -> Result<u32> {
        if sample_rate_hz <= 225_000
            || sample_rate_hz > 3_200_000
            || (sample_rate_hz > 300_000 && sample_rate_hz <= 900_000)
        {
            return Err(Error::InvalidSampleRate(sample_rate_hz));
        }

        let numerator = (self.rtl_xtal_hz as u64)
            .checked_shl(22)
            .ok_or(Error::ArithmeticOverflow)?;
        let mut ratio = numerator / sample_rate_hz as u64;
        ratio &= 0x0fff_fffc;
        if ratio == 0 {
            return Err(Error::InvalidSampleRate(sample_rate_hz));
        }

        let real_ratio = ratio | ((ratio & 0x0800_0000) << 1);
        let real_rate = (numerator / real_ratio) as u32;

        let bandwidth_hz = self.bandwidth_hz.unwrap_or(real_rate);
        let new_if = if matches!(self.tuner, TunerState::R82xx(_)) {
            self.set_i2c_repeater(true).await?;
            let result = match &mut self.tuner {
                TunerState::R82xx(tuner) => {
                    tuner
                        .set_bandwidth(&self.usb, bandwidth_hz, real_rate)
                        .await
                }
                _ => unreachable!(),
            };
            let repeater_result = self.set_i2c_repeater(false).await;
            let if_hz = result?;
            repeater_result?;
            Some(if_hz)
        } else {
            None
        };
        if let Some(if_hz) = new_if {
            self.set_if_freq(if_hz).await?;
        }

        self.usb
            .demod_write_reg(1, 0x9f, ((ratio >> 16) & 0xffff) as u16, 2)
            .await?;
        self.usb
            .demod_write_reg(1, 0xa1, (ratio & 0xffff) as u16, 2)
            .await?;
        self.set_sample_freq_correction(self.freq_correction_ppm)
            .await?;
        self.reset_demod().await?;

        self.sample_rate_hz = real_rate;
        if self.center_freq_hz.is_some() && new_if.is_some() {
            self.retune().await?;
        }

        Ok(real_rate)
    }

    pub async fn set_center_frequency(&mut self, frequency_hz: u32) -> Result<()> {
        if frequency_hz == 0 {
            return Err(Error::InvalidFrequency(frequency_hz));
        }

        self.set_i2c_repeater(true).await?;
        let result = match &mut self.tuner {
            TunerState::R82xx(tuner) => tuner.set_freq(&self.usb, frequency_hz).await,
            TunerState::Unsupported(kind) => Err(Error::UnsupportedTuner(*kind)),
            TunerState::Unknown => Err(Error::TunerNotFound),
        };
        let repeater_result = self.set_i2c_repeater(false).await;

        result?;
        repeater_result?;

        self.center_freq_hz = Some(frequency_hz);
        Ok(())
    }

    pub async fn set_tuner_gain(&mut self, gain: GainMode) -> Result<()> {
        let (manual, gain_tenths_db) = match gain {
            GainMode::Auto => (false, 0),
            GainMode::ManualTenthsDb(gain) => {
                if !(-100..=500).contains(&gain) {
                    return Err(Error::InvalidGain(gain));
                }
                (true, gain)
            }
        };

        self.set_i2c_repeater(true).await?;
        let result = match &mut self.tuner {
            TunerState::R82xx(tuner) => tuner.set_gain(&self.usb, manual, gain_tenths_db).await,
            TunerState::Unsupported(kind) => Err(Error::UnsupportedTuner(*kind)),
            TunerState::Unknown => Err(Error::TunerNotFound),
        };
        let repeater_result = self.set_i2c_repeater(false).await;

        result?;
        repeater_result?;

        Ok(())
    }

    pub async fn set_bandwidth(&mut self, bandwidth_hz: Option<u32>) -> Result<()> {
        self.bandwidth_hz = bandwidth_hz;
        let bandwidth = bandwidth_hz.unwrap_or(self.sample_rate_hz);
        self.set_i2c_repeater(true).await?;
        let result = match &mut self.tuner {
            TunerState::R82xx(tuner) => {
                tuner
                    .set_bandwidth(&self.usb, bandwidth, self.sample_rate_hz)
                    .await
            }
            TunerState::Unsupported(kind) => Err(Error::UnsupportedTuner(*kind)),
            TunerState::Unknown => Err(Error::TunerNotFound),
        };
        let repeater_result = self.set_i2c_repeater(false).await;
        let new_if = Some(result?);
        repeater_result?;

        if let Some(if_hz) = new_if {
            self.set_if_freq(if_hz).await?;
            if self.center_freq_hz.is_some() {
                self.retune().await?;
            }
        }

        Ok(())
    }

    pub async fn set_frequency_correction(&mut self, ppm: i32) -> Result<()> {
        self.freq_correction_ppm = ppm;
        self.set_sample_freq_correction(ppm).await?;
        if self.center_freq_hz.is_some() {
            self.retune().await?;
        }
        Ok(())
    }

    pub async fn set_agc_mode(&self, enabled: bool) -> Result<()> {
        self.usb
            .demod_write_reg(0, 0x19, if enabled { 0x25 } else { 0x05 }, 1)
            .await
    }

    pub async fn set_test_mode(&self, enabled: bool) -> Result<()> {
        self.usb
            .demod_write_reg(0, 0x19, if enabled { 0x03 } else { 0x05 }, 1)
            .await
    }

    pub async fn reset_buffer(&self) -> Result<()> {
        self.usb.write_reg(USBB, USB_EPA_CTL, 0x1002, 2).await?;
        self.usb.write_reg(USBB, USB_EPA_CTL, 0x0000, 2).await
    }

    pub async fn read_bytes(&self, len: usize) -> Result<Vec<u8>> {
        let len = if len == 0 { DEFAULT_READ_LENGTH } else { len };
        self.usb
            .interface
            .bulk_in(DEFAULT_BULK_ENDPOINT, len)
            .await
            .map_err(Error::from)
    }

    pub async fn read_iq_samples(&self, sample_count: usize) -> Result<Vec<IqSample>> {
        let bytes = self.read_bytes(sample_count.saturating_mul(2)).await?;
        Ok(bytes_to_iq_samples(&bytes))
    }

    async fn init_baseband(&mut self) -> Result<()> {
        self.usb.write_reg(USBB, USB_SYSCTL, 0x09, 1).await?;
        self.usb.write_reg(USBB, USB_EPA_MAXPKT, 0x0002, 2).await?;
        self.usb.write_reg(USBB, USB_EPA_CTL, 0x1002, 2).await?;

        self.usb.write_reg(SYSB, DEMOD_CTL_1, 0x22, 1).await?;
        self.usb.write_reg(SYSB, DEMOD_CTL, 0xe8, 1).await?;
        self.reset_demod().await?;

        self.usb.demod_write_reg(1, 0x15, 0x00, 1).await?;
        self.usb.demod_write_reg(1, 0x16, 0x0000, 2).await?;
        for offset in 0..6 {
            self.usb.demod_write_reg(1, 0x16 + offset, 0x00, 1).await?;
        }

        self.set_fir().await?;
        self.usb.demod_write_reg(0, 0x19, 0x05, 1).await?;
        self.usb.demod_write_reg(1, 0x93, 0xf0, 1).await?;
        self.usb.demod_write_reg(1, 0x94, 0x0f, 1).await?;
        self.usb.demod_write_reg(1, 0x11, 0x00, 1).await?;
        self.usb.demod_write_reg(1, 0x04, 0x00, 1).await?;
        self.usb.demod_write_reg(0, 0x61, 0x60, 1).await?;
        self.usb.demod_write_reg(0, 0x06, 0x80, 1).await?;
        self.usb.demod_write_reg(1, 0xb1, 0x1b, 1).await?;
        self.usb.demod_write_reg(0, 0x0d, 0x83, 1).await
    }

    async fn probe_tuner(&mut self) -> Result<()> {
        self.set_i2c_repeater(true).await?;
        let result = self.probe_tuner_inner().await;
        let repeater_result = self.set_i2c_repeater(false).await;

        result?;
        repeater_result?;
        Ok(())
    }

    async fn probe_tuner_inner(&mut self) -> Result<()> {
        if self.try_i2c_read_reg(E4K_I2C_ADDR, E4K_CHECK_ADDR).await == Some(E4K_CHECK_VAL) {
            self.tuner = TunerState::Unsupported(TunerKind::E4000);
            return Ok(());
        }
        if self
            .try_i2c_read_reg(FC0013_I2C_ADDR, FC0013_CHECK_ADDR)
            .await
            == Some(FC0013_CHECK_VAL)
        {
            self.tuner = TunerState::Unsupported(TunerKind::Fc0013);
            return Ok(());
        }
        if self
            .try_i2c_read_reg(R820T_I2C_ADDR, R82XX_CHECK_ADDR)
            .await
            == Some(R82XX_CHECK_VAL)
        {
            self.configure_r82xx(R82xxKind::R820T).await?;
            return Ok(());
        }
        if self
            .try_i2c_read_reg(R828D_I2C_ADDR, R82XX_CHECK_ADDR)
            .await
            == Some(R82XX_CHECK_VAL)
        {
            self.configure_r82xx(R82xxKind::R828D).await?;
            return Ok(());
        }

        self.set_gpio_output(4).await?;
        self.set_gpio_bit(4, true).await?;
        self.set_gpio_bit(4, false).await?;

        if self
            .try_i2c_read_reg(FC2580_I2C_ADDR, FC2580_CHECK_ADDR)
            .await
            .is_some_and(|reg| (reg & 0x7f) == FC2580_CHECK_VAL)
        {
            self.tuner = TunerState::Unsupported(TunerKind::Fc2580);
            return Ok(());
        }
        if self
            .try_i2c_read_reg(FC0012_I2C_ADDR, FC0012_CHECK_ADDR)
            .await
            == Some(FC0012_CHECK_VAL)
        {
            self.set_gpio_output(6).await?;
            self.tuner = TunerState::Unsupported(TunerKind::Fc0012);
            return Ok(());
        }

        self.tuner = TunerState::Unknown;
        Ok(())
    }

    async fn configure_r82xx(&mut self, kind: R82xxKind) -> Result<()> {
        self.usb.demod_write_reg(1, 0xb1, 0x1a, 1).await?;
        self.usb.demod_write_reg(0, 0x08, 0x4d, 1).await?;
        self.set_if_freq(R82XX_IF_FREQ).await?;
        self.usb.demod_write_reg(1, 0x15, 0x01, 1).await?;

        let tuner_xtal = match kind {
            R82xxKind::R820T => self.rtl_xtal_hz,
            R82xxKind::R828D if self.is_blog_v4() => self.rtl_xtal_hz,
            R82xxKind::R828D => R828D_XTAL_FREQ,
        };

        let mut tuner = R82xx::new(kind, tuner_xtal, self.is_blog_v4());
        tuner.init(&self.usb).await?;
        self.tuner = TunerState::R82xx(tuner);
        Ok(())
    }

    async fn retune(&mut self) -> Result<()> {
        if let Some(frequency) = self.center_freq_hz {
            self.set_center_frequency(frequency).await?;
        }
        Ok(())
    }

    async fn set_if_freq(&self, frequency_hz: u32) -> Result<()> {
        let ratio = ((frequency_hz as i64) << 22) / self.rtl_xtal_hz as i64;
        let if_freq = -ratio;
        self.usb
            .demod_write_reg(1, 0x19, ((if_freq >> 16) & 0x3f) as u16, 1)
            .await?;
        self.usb
            .demod_write_reg(1, 0x1a, ((if_freq >> 8) & 0xff) as u16, 1)
            .await?;
        self.usb
            .demod_write_reg(1, 0x1b, (if_freq & 0xff) as u16, 1)
            .await
    }

    async fn set_sample_freq_correction(&self, ppm: i32) -> Result<()> {
        let offset = (ppm as i64)
            .checked_mul(-1)
            .and_then(|v| v.checked_mul(1 << 24))
            .ok_or(Error::ArithmeticOverflow)?
            / 1_000_000;
        self.usb
            .demod_write_reg(1, 0x3f, (offset & 0xff) as u16, 1)
            .await?;
        self.usb
            .demod_write_reg(1, 0x3e, ((offset >> 8) & 0x3f) as u16, 1)
            .await
    }

    async fn reset_demod(&self) -> Result<()> {
        self.usb.demod_write_reg(1, 0x01, 0x14, 1).await?;
        self.usb.demod_write_reg(1, 0x01, 0x10, 1).await
    }

    async fn set_i2c_repeater(&self, enabled: bool) -> Result<()> {
        self.usb
            .demod_write_reg(1, 0x01, if enabled { 0x18 } else { 0x10 }, 1)
            .await
    }

    async fn set_fir(&self) -> Result<()> {
        let mut fir = [0u8; 20];
        for (i, value) in FIR_DEFAULT.iter().take(8).enumerate() {
            fir[i] = *value as i8 as u8;
        }
        for i in (0..8).step_by(2) {
            let val0 = FIR_DEFAULT[8 + i];
            let val1 = FIR_DEFAULT[8 + i + 1];
            let out = 8 + i * 3 / 2;
            fir[out] = (val0 >> 4) as u8;
            fir[out + 1] = ((val0 << 4) as u8) | (((val1 >> 8) as u8) & 0x0f);
            fir[out + 2] = val1 as u8;
        }

        for (offset, value) in fir.into_iter().enumerate() {
            self.usb
                .demod_write_reg(1, 0x1c + offset as u16, value as u16, 1)
                .await?;
        }

        Ok(())
    }

    async fn try_i2c_read_reg(&self, i2c_addr: u8, reg: u8) -> Option<u8> {
        self.usb.i2c_read_reg(i2c_addr, reg).await.ok()
    }

    async fn set_gpio_output(&self, gpio: u8) -> Result<()> {
        self.usb.set_gpio_output(gpio).await
    }

    pub(crate) async fn set_gpio_bit(&self, gpio: u8, enabled: bool) -> Result<()> {
        self.usb.set_gpio_bit(gpio, enabled).await
    }

    fn is_blog_v4(&self) -> bool {
        self.manufacturer.as_deref() == Some("RTLSDRBlog")
            && self.product.as_deref() == Some("Blog V4")
    }
}

pub fn bytes_to_iq_samples(bytes: &[u8]) -> Vec<IqSample> {
    bytes
        .chunks_exact(2)
        .map(|pair| IqSample {
            i: (pair[0] as f32 - 127.5) / 127.5,
            q: (pair[1] as f32 - 127.5) / 127.5,
        })
        .collect()
}

pub(crate) struct RtlUsb {
    _device: cross_usb::Device,
    interface: cross_usb::Interface,
}

impl RtlUsb {
    pub(crate) async fn set_gpio_output(&self, gpio: u8) -> Result<()> {
        let bit = 1u16 << gpio;
        let direction = self.read_reg(SYSB, GPD, 1).await?;
        self.write_reg(SYSB, GPD, direction & !bit, 1).await?;
        let output_enable = self.read_reg(SYSB, GPOE, 1).await?;
        self.write_reg(SYSB, GPOE, output_enable | bit, 1).await
    }

    pub(crate) async fn set_gpio_bit(&self, gpio: u8, enabled: bool) -> Result<()> {
        let bit = 1u16 << gpio;
        let value = self.read_reg(SYSB, GPO, 1).await?;
        let value = if enabled { value | bit } else { value & !bit };
        self.write_reg(SYSB, GPO, value, 1).await
    }
    pub(crate) async fn read_reg(&self, block: u8, addr: u16, len: u16) -> Result<u16> {
        let data = self.read_array(block, addr, len).await?;
        Ok(match data.as_slice() {
            [lo] => *lo as u16,
            [lo, hi, ..] => ((*hi as u16) << 8) | *lo as u16,
            _ => 0,
        })
    }

    pub(crate) async fn write_reg(
        &self,
        block: u8,
        addr: u16,
        value: u16,
        len: usize,
    ) -> Result<()> {
        let data = reg_value_bytes(value, len)?;
        self.write_array(block, addr, &data).await
    }

    pub(crate) async fn read_array(&self, block: u8, addr: u16, len: u16) -> Result<Vec<u8>> {
        self.interface
            .control_in(ControlIn {
                control_type: ControlType::Vendor,
                recipient: Recipient::Device,
                request: 0,
                value: addr,
                index: (block as u16) << 8,
                length: len,
            })
            .await
            .map_err(Error::from)
            .map_err(|err| err.control("control_in", addr, (block as u16) << 8))
    }

    pub(crate) async fn write_array(&self, block: u8, addr: u16, data: &[u8]) -> Result<()> {
        if data.len() > u16::MAX as usize {
            return Err(Error::InvalidTransferLength(data.len()));
        }

        let actual = self
            .interface
            .control_out(ControlOut {
                control_type: ControlType::Vendor,
                recipient: Recipient::Device,
                request: 0,
                value: addr,
                index: ((block as u16) << 8) | 0x10,
                data,
            })
            .await
            .map_err(Error::from)
            .map_err(|err| err.control("control_out", addr, ((block as u16) << 8) | 0x10))?;

        if actual == data.len() {
            Ok(())
        } else {
            Err(Error::ShortTransfer {
                expected: data.len(),
                actual,
            })
        }
    }

    pub(crate) async fn demod_write_reg(
        &self,
        page: u8,
        addr: u16,
        value: u16,
        len: usize,
    ) -> Result<()> {
        let data = reg_value_bytes(value, len)?;
        let actual = self
            .interface
            .control_out(ControlOut {
                control_type: ControlType::Vendor,
                recipient: Recipient::Device,
                request: 0,
                value: (addr << 8) | 0x20,
                index: 0x10 | page as u16,
                data: &data,
            })
            .await
            .map_err(Error::from)
            .map_err(|err| {
                err.control("demod_control_out", (addr << 8) | 0x20, 0x10 | page as u16)
            })?;

        if actual != data.len() {
            return Err(Error::ShortTransfer {
                expected: data.len(),
                actual,
            });
        }

        let _ = self.demod_read_reg(0x0a, 0x01, 1).await?;
        Ok(())
    }

    pub(crate) async fn demod_read_reg(&self, page: u8, addr: u16, len: u16) -> Result<u16> {
        let data = self
            .interface
            .control_in(ControlIn {
                control_type: ControlType::Vendor,
                recipient: Recipient::Device,
                request: 0,
                value: (addr << 8) | 0x20,
                index: page as u16,
                length: len,
            })
            .await
            .map_err(Error::from)
            .map_err(|err| err.control("demod_control_in", (addr << 8) | 0x20, page as u16))?;

        Ok(match data.as_slice() {
            [lo] => *lo as u16,
            [lo, hi, ..] => ((*hi as u16) << 8) | *lo as u16,
            _ => 0,
        })
    }

    pub(crate) async fn i2c_write(&self, i2c_addr: u8, data: &[u8]) -> Result<()> {
        self.write_array(IICB, i2c_addr as u16, data).await
    }

    pub(crate) async fn i2c_read(&self, i2c_addr: u8, len: u16) -> Result<Vec<u8>> {
        self.read_array(IICB, i2c_addr as u16, len).await
    }

    pub(crate) async fn i2c_read_reg(&self, i2c_addr: u8, reg: u8) -> Result<u8> {
        self.i2c_write(i2c_addr, &[reg]).await?;
        let data = self.i2c_read(i2c_addr, 1).await?;
        data.first().copied().ok_or(Error::ShortTransfer {
            expected: 1,
            actual: 0,
        })
    }
}

fn reg_value_bytes(value: u16, len: usize) -> Result<Vec<u8>> {
    match len {
        1 => Ok(vec![(value & 0xff) as u8]),
        2 => Ok(vec![(value >> 8) as u8, (value & 0xff) as u8]),
        _ => Err(Error::InvalidTransferLength(len)),
    }
}

fn device_filters() -> Vec<cross_usb::DeviceFilter> {
    KNOWN_DEVICES
        .iter()
        .map(|device| {
            cross_usb::DeviceFilter::new(
                Some(device.vendor_id),
                Some(device.product_id),
                None,
                None,
                None,
            )
        })
        .collect()
}

fn known_device_name(vendor_id: u16, product_id: u16) -> Option<&'static str> {
    KNOWN_DEVICES
        .iter()
        .find(|device| device.vendor_id == vendor_id && device.product_id == product_id)
        .map(|device| device.name)
}
