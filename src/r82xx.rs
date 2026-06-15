use crate::error::{Error, Result};
use crate::rtl2832::RtlUsb;

pub(crate) const R82XX_CHECK_ADDR: u8 = 0x00;
pub(crate) const R82XX_CHECK_VAL: u8 = 0x69;
pub(crate) const R82XX_IF_FREQ: u32 = 3_570_000;

const REG_SHADOW_START: u8 = 5;
const NUM_REGS: usize = 30;
const VER_NUM: u8 = 49;

const MHZ: u32 = 1_000_000;
const HF: u8 = 1;
const VHF: u8 = 2;
const UHF: u8 = 3;

const INIT_ARRAY: [u8; NUM_REGS] = [
    0x83, 0x32, 0x75, 0xc0, 0x40, 0xd6, 0x6c, 0xf5, 0x63, 0x75, 0x68, 0x6c, 0x83, 0x80, 0x00, 0x0f,
    0x00, 0xc0, 0x30, 0x48, 0xcc, 0x60, 0x00, 0x54, 0xae, 0x4a, 0xc0, 0x00, 0x00, 0x00,
];

const IF_LOW_PASS_BW_TABLE: [u32; 10] = [
    1_700_000, 1_600_000, 1_550_000, 1_450_000, 1_200_000, 900_000, 700_000, 550_000, 450_000,
    350_000,
];

const LNA_GAIN_STEPS: [i32; 16] = [0, 9, 13, 40, 38, 13, 31, 22, 26, 31, 26, 14, 19, 5, 35, 13];
const MIXER_GAIN_STEPS: [i32; 16] = [0, 5, 10, 10, 19, 9, 10, 25, 17, 10, 8, 16, 13, 6, 3, -8];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum R82xxKind {
    R820T,
    R828D,
}

#[derive(Debug)]
pub(crate) struct R82xx {
    kind: R82xxKind,
    i2c_addr: u8,
    xtal_hz: u32,
    max_i2c_msg_len: usize,
    regs: [u8; NUM_REGS],
    xtal_cap_sel: XtalCap,
    int_freq: u32,
    fil_cal_code: u8,
    input: u8,
    has_lock: bool,
    is_blog_v4: bool,
}

#[derive(Debug, Clone, Copy)]
enum XtalCap {
    High0,
}

#[derive(Clone, Copy)]
struct FreqRange {
    freq_mhz: u32,
    open_d: u8,
    rf_mux_ploy: u8,
    tf_c: u8,
    xtal_cap20p: u8,
    xtal_cap10p: u8,
    xtal_cap0p: u8,
}

const FREQ_RANGES: &[FreqRange] = &[
    FreqRange {
        freq_mhz: 0,
        open_d: 0x08,
        rf_mux_ploy: 0x02,
        tf_c: 0xdf,
        xtal_cap20p: 0x02,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 50,
        open_d: 0x08,
        rf_mux_ploy: 0x02,
        tf_c: 0xbe,
        xtal_cap20p: 0x02,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 55,
        open_d: 0x08,
        rf_mux_ploy: 0x02,
        tf_c: 0x8b,
        xtal_cap20p: 0x02,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 60,
        open_d: 0x08,
        rf_mux_ploy: 0x02,
        tf_c: 0x7b,
        xtal_cap20p: 0x02,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 65,
        open_d: 0x08,
        rf_mux_ploy: 0x02,
        tf_c: 0x69,
        xtal_cap20p: 0x02,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 70,
        open_d: 0x08,
        rf_mux_ploy: 0x02,
        tf_c: 0x58,
        xtal_cap20p: 0x02,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 75,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x44,
        xtal_cap20p: 0x02,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 80,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x44,
        xtal_cap20p: 0x02,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 90,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x34,
        xtal_cap20p: 0x01,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 100,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x34,
        xtal_cap20p: 0x01,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 110,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x24,
        xtal_cap20p: 0x01,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 120,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x24,
        xtal_cap20p: 0x01,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 140,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x14,
        xtal_cap20p: 0x01,
        xtal_cap10p: 0x01,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 180,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x13,
        xtal_cap20p: 0x00,
        xtal_cap10p: 0x00,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 220,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x13,
        xtal_cap20p: 0x00,
        xtal_cap10p: 0x00,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 250,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x11,
        xtal_cap20p: 0x00,
        xtal_cap10p: 0x00,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 280,
        open_d: 0x00,
        rf_mux_ploy: 0x02,
        tf_c: 0x00,
        xtal_cap20p: 0x00,
        xtal_cap10p: 0x00,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 310,
        open_d: 0x00,
        rf_mux_ploy: 0x41,
        tf_c: 0x00,
        xtal_cap20p: 0x00,
        xtal_cap10p: 0x00,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 450,
        open_d: 0x00,
        rf_mux_ploy: 0x41,
        tf_c: 0x00,
        xtal_cap20p: 0x00,
        xtal_cap10p: 0x00,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 588,
        open_d: 0x00,
        rf_mux_ploy: 0x40,
        tf_c: 0x00,
        xtal_cap20p: 0x00,
        xtal_cap10p: 0x00,
        xtal_cap0p: 0x00,
    },
    FreqRange {
        freq_mhz: 650,
        open_d: 0x00,
        rf_mux_ploy: 0x40,
        tf_c: 0x00,
        xtal_cap20p: 0x00,
        xtal_cap10p: 0x00,
        xtal_cap0p: 0x00,
    },
];

impl R82xx {
    pub(crate) fn new(kind: R82xxKind, xtal_hz: u32, is_blog_v4: bool) -> Self {
        let i2c_addr = match kind {
            R82xxKind::R820T => 0x34,
            R82xxKind::R828D => 0x74,
        };

        Self {
            kind,
            i2c_addr,
            xtal_hz,
            max_i2c_msg_len: 8,
            regs: [0; NUM_REGS],
            xtal_cap_sel: XtalCap::High0,
            int_freq: R82XX_IF_FREQ,
            fil_cal_code: 0,
            input: 0,
            has_lock: false,
            is_blog_v4,
        }
    }

    pub(crate) fn kind(&self) -> R82xxKind {
        self.kind
    }

    pub(crate) async fn init(&mut self, usb: &RtlUsb) -> Result<()> {
        self.xtal_cap_sel = XtalCap::High0;
        self.regs = [0; NUM_REGS];
        self.write(usb, 0x05, &INIT_ARRAY).await?;
        self.set_tv_standard(usb).await?;
        self.sysfreq_sel(usb).await
    }

    pub(crate) async fn set_freq(&mut self, usb: &RtlUsb, freq_hz: u32) -> Result<()> {
        let upconvert_freq = if self.is_blog_v4 && freq_hz < 28_800_000 {
            freq_hz
                .checked_add(28_800_000)
                .ok_or(Error::ArithmeticOverflow)?
        } else {
            freq_hz
        };
        let lo_freq = upconvert_freq
            .checked_add(self.int_freq)
            .ok_or(Error::ArithmeticOverflow)?;

        self.set_mux(usb, lo_freq).await?;
        self.set_pll(usb, lo_freq).await?;
        if !self.has_lock {
            return Err(Error::PllDidNotLock);
        }

        if self.is_blog_v4 {
            let open_d = if freq_hz <= 2_200_000
                || (85 * MHZ..=112 * MHZ).contains(&freq_hz)
                || (172 * MHZ..=242 * MHZ).contains(&freq_hz)
            {
                0x00
            } else {
                0x08
            };
            self.write_reg_mask(usb, 0x17, open_d, 0x08).await?;

            let band = if freq_hz <= 28_800_000 {
                HF
            } else if freq_hz < 250 * MHZ {
                VHF
            } else {
                UHF
            };

            if band != self.input {
                self.input = band;
                let cable_2_in = if band == HF { 0x08 } else { 0x00 };
                self.write_reg_mask(usb, 0x06, cable_2_in, 0x08).await?;
                usb.set_gpio_output(5).await?;
                usb.set_gpio_bit(5, cable_2_in == 0).await?;

                let cable_1_in = if band == VHF { 0x40 } else { 0x00 };
                self.write_reg_mask(usb, 0x05, cable_1_in, 0x40).await?;

                let air_in = if band == UHF { 0x00 } else { 0x20 };
                self.write_reg_mask(usb, 0x05, air_in, 0x20).await?;
            }
        } else if self.kind == R82xxKind::R828D {
            let air_cable1_in = if freq_hz > 345 * MHZ { 0x00 } else { 0x60 };
            if air_cable1_in != self.input {
                self.input = air_cable1_in;
                self.write_reg_mask(usb, 0x05, air_cable1_in, 0x60).await?;
            }
        }

        Ok(())
    }

    pub(crate) async fn set_gain(
        &mut self,
        usb: &RtlUsb,
        manual: bool,
        gain_tenths_db: i32,
    ) -> Result<()> {
        if manual {
            let mut total_gain = 0;
            let mut mix_index = 0u8;
            let mut lna_index = 0u8;

            self.write_reg_mask(usb, 0x05, 0x10, 0x10).await?;
            self.write_reg_mask(usb, 0x07, 0x00, 0x10).await?;
            let _ = self.read(usb, 0x00, 4).await?;
            self.write_reg_mask(usb, 0x0c, 0x08, 0x9f).await?;

            for _ in 0..15 {
                if total_gain >= gain_tenths_db {
                    break;
                }
                lna_index += 1;
                total_gain += LNA_GAIN_STEPS[lna_index as usize];
                if total_gain >= gain_tenths_db {
                    break;
                }
                mix_index += 1;
                total_gain += MIXER_GAIN_STEPS[mix_index as usize];
            }

            self.write_reg_mask(usb, 0x05, lna_index, 0x0f).await?;
            self.write_reg_mask(usb, 0x07, mix_index, 0x0f).await?;
        } else {
            self.write_reg_mask(usb, 0x05, 0x00, 0x10).await?;
            self.write_reg_mask(usb, 0x07, 0x10, 0x10).await?;
            self.write_reg_mask(usb, 0x0c, 0x0b, 0x9f).await?;
        }

        Ok(())
    }

    pub(crate) async fn set_bandwidth(
        &mut self,
        usb: &RtlUsb,
        bandwidth_hz: u32,
        sample_rate_hz: u32,
    ) -> Result<u32> {
        let mut bw = if bandwidth_hz == 0 {
            sample_rate_hz
        } else {
            bandwidth_hz
        };
        let (reg_0a, reg_0b, int_freq) = if bw > 7_000_000 {
            (0x10, 0x0b, 4_570_000)
        } else if bw > 6_000_000 {
            (0x10, 0x2a, 4_570_000)
        } else if bw > IF_LOW_PASS_BW_TABLE[0] + 350_000 + 380_000 {
            (0x10, 0x6b, 3_570_000)
        } else {
            let mut reg_0b = 0x80;
            let mut int_freq = 2_300_000;
            let mut real_bw = 0;

            if bw > IF_LOW_PASS_BW_TABLE[0] + 350_000 {
                bw -= 380_000;
                int_freq += 380_000;
                real_bw += 380_000;
            } else {
                reg_0b |= 0x20;
            }

            if bw > IF_LOW_PASS_BW_TABLE[0] {
                bw -= 350_000;
                int_freq += 350_000;
                real_bw += 350_000;
            } else {
                reg_0b |= 0x40;
            }

            let insertion = IF_LOW_PASS_BW_TABLE
                .iter()
                .position(|entry| bw > *entry)
                .unwrap_or(IF_LOW_PASS_BW_TABLE.len());
            let idx = insertion
                .saturating_sub(1)
                .min(IF_LOW_PASS_BW_TABLE.len() - 1);
            reg_0b |= 15 - idx as u8;
            real_bw += IF_LOW_PASS_BW_TABLE[idx];
            int_freq -= real_bw / 2;

            (0x00, reg_0b, int_freq)
        };

        self.int_freq = int_freq;
        self.write_reg_mask(usb, 0x0a, reg_0a, 0x10).await?;
        self.write_reg_mask(usb, 0x0b, reg_0b, 0xef).await?;
        Ok(self.int_freq)
    }

    async fn set_tv_standard(&mut self, usb: &RtlUsb) -> Result<()> {
        let filt_cal_lo = 56_000_000;
        let filt_gain = 0x10;
        let img_r = 0x00;
        let filt_q = 0x10;
        let hp_cor = 0x6b;
        let ext_enable = 0x60;
        let loop_through = 0x01;
        let lt_att = 0x00;
        let flt_ext_widest = 0x00;
        let polyfil_cur = 0x60;

        self.regs = INIT_ARRAY;
        self.write_reg_mask(usb, 0x0c, 0x00, 0x0f).await?;
        self.write_reg_mask(usb, 0x13, VER_NUM, 0x3f).await?;
        self.write_reg_mask(usb, 0x1d, 0x00, 0x38).await?;
        self.int_freq = 3_570_000;

        for _ in 0..2 {
            self.write_reg_mask(usb, 0x0b, hp_cor, 0x60).await?;
            self.write_reg_mask(usb, 0x0f, 0x04, 0x04).await?;
            self.write_reg_mask(usb, 0x10, 0x00, 0x03).await?;
            self.set_pll(usb, filt_cal_lo).await?;
            if !self.has_lock {
                return Err(Error::PllDidNotLock);
            }
            self.write_reg_mask(usb, 0x0b, 0x10, 0x10).await?;
            self.write_reg_mask(usb, 0x0b, 0x00, 0x10).await?;
            self.write_reg_mask(usb, 0x0f, 0x00, 0x04).await?;

            let data = self.read(usb, 0x00, 5).await?;
            self.fil_cal_code = data[4] & 0x0f;
            if self.fil_cal_code != 0 && self.fil_cal_code != 0x0f {
                break;
            }
        }
        if self.fil_cal_code == 0x0f {
            self.fil_cal_code = 0;
        }

        self.write_reg_mask(usb, 0x0a, filt_q | self.fil_cal_code, 0x1f)
            .await?;
        self.write_reg_mask(usb, 0x0b, hp_cor, 0xef).await?;
        self.write_reg_mask(usb, 0x07, img_r, 0x80).await?;
        self.write_reg_mask(usb, 0x06, filt_gain, 0x30).await?;
        self.write_reg_mask(usb, 0x1e, ext_enable, 0x60).await?;
        self.write_reg_mask(usb, 0x05, loop_through, 0x80).await?;
        self.write_reg_mask(usb, 0x1f, lt_att, 0x80).await?;
        self.write_reg_mask(usb, 0x0f, flt_ext_widest, 0x80).await?;
        self.write_reg_mask(usb, 0x19, polyfil_cur, 0x60).await
    }

    async fn sysfreq_sel(&mut self, usb: &RtlUsb) -> Result<()> {
        let mixer_top = 0x24;
        let lna_top = 0xe5;
        let cp_cur = 0x38;
        let div_buf_cur = 0x30;
        let lna_vth_l = 0x53;
        let mixer_vth_l = 0x75;
        let air_cable1_in = 0x00;
        let cable2_in = 0x00;
        let pre_dect = 0x40;
        let lna_discharge = 14;
        let filter_cur = 0x40;

        let _ = pre_dect;
        self.write_reg_mask(usb, 0x1d, lna_top, 0xc7).await?;
        self.write_reg_mask(usb, 0x1c, mixer_top, 0xf8).await?;
        self.write_reg(usb, 0x0d, lna_vth_l).await?;
        self.write_reg(usb, 0x0e, mixer_vth_l).await?;
        self.input = air_cable1_in;
        self.write_reg_mask(usb, 0x05, air_cable1_in, 0x60).await?;
        self.write_reg_mask(usb, 0x06, cable2_in, 0x08).await?;
        self.write_reg_mask(usb, 0x11, cp_cur, 0x38).await?;
        self.write_reg_mask(usb, 0x17, div_buf_cur, 0x30).await?;
        self.write_reg_mask(usb, 0x0a, filter_cur, 0x60).await?;

        self.write_reg_mask(usb, 0x1d, 0x00, 0x38).await?;
        self.write_reg_mask(usb, 0x1c, 0x00, 0x04).await?;
        self.write_reg_mask(usb, 0x06, 0x00, 0x40).await?;
        self.write_reg_mask(usb, 0x1a, 0x30, 0x30).await?;
        self.write_reg_mask(usb, 0x1d, 0x18, 0x38).await?;
        self.write_reg_mask(usb, 0x1c, mixer_top, 0x04).await?;
        self.write_reg_mask(usb, 0x1e, lna_discharge, 0x1f).await?;
        self.write_reg_mask(usb, 0x1a, 0x20, 0x30).await
    }

    async fn set_mux(&mut self, usb: &RtlUsb, freq_hz: u32) -> Result<()> {
        let freq_mhz = freq_hz / MHZ;
        let range = FREQ_RANGES
            .iter()
            .enumerate()
            .find_map(|(i, range)| {
                let next = FREQ_RANGES.get(i + 1)?;
                (freq_mhz < next.freq_mhz).then_some(range)
            })
            .unwrap_or_else(|| FREQ_RANGES.last().expect("non-empty frequency table"));

        self.write_reg_mask(usb, 0x17, range.open_d, 0x08).await?;
        self.write_reg_mask(usb, 0x1a, range.rf_mux_ploy, 0xc3)
            .await?;
        self.write_reg(usb, 0x1b, range.tf_c).await?;

        let val = match self.xtal_cap_sel {
            XtalCap::High0 => range.xtal_cap0p,
        };
        let _ = range.xtal_cap20p;
        let _ = range.xtal_cap10p;
        self.write_reg_mask(usb, 0x10, val, 0x0b).await?;
        self.write_reg_mask(usb, 0x08, 0x00, 0x3f).await?;
        self.write_reg_mask(usb, 0x09, 0x00, 0x3f).await
    }

    async fn set_pll(&mut self, usb: &RtlUsb, freq_hz: u32) -> Result<()> {
        let freq_khz = (freq_hz + 500) / 1000;
        let pll_ref = self.xtal_hz as u64;
        let vco_min = 1_770_000u32;
        let vco_max = vco_min * 2;
        let mut mix_div = 2u32;
        let mut div_num = 0u8;
        let vco_power_ref = if self.kind == R82xxKind::R828D { 1 } else { 2 };

        self.write_reg_mask(usb, 0x1a, 0x00, 0x0c).await?;

        let mut regs = self.reg_window::<7>(0x10, 7)?;
        regs[0] = mask_reg8(regs[0], 0x00, 0x10);
        regs[2] = mask_reg8(regs[2], 0x80, 0xe0);

        loop {
            let vco = freq_khz
                .checked_mul(mix_div)
                .ok_or(Error::ArithmeticOverflow)?;
            if vco >= vco_min && vco < vco_max {
                let mut div_buf = mix_div;
                while div_buf > 2 {
                    div_buf >>= 1;
                    div_num += 1;
                }
                break;
            }
            mix_div = mix_div
                .checked_mul(2)
                .ok_or(Error::InvalidFrequency(freq_hz))?;
            if mix_div > 64 {
                return Err(Error::InvalidFrequency(freq_hz));
            }
        }

        let data = self.read(usb, 0x00, 5).await?;
        let fine_tune = (data[4] & 0x30) >> 4;
        if fine_tune > vco_power_ref {
            div_num = div_num.saturating_sub(1);
        } else if fine_tune < vco_power_ref {
            div_num = div_num.saturating_add(1);
        }
        regs[0] = mask_reg8(regs[0], div_num << 5, 0xe0);

        let vco_freq = freq_hz as u64 * mix_div as u64;
        let vco_div = (pll_ref + 65_536 * vco_freq) / (2 * pll_ref);
        let nint = (vco_div / 65_536) as u8;
        let sdm = (vco_div % 65_536) as u16;

        if nint < 13 || nint > (128 / vco_power_ref) - 1 {
            return Err(Error::InvalidFrequency(freq_hz));
        }

        let ni = (nint - 13) / 4;
        let si = nint - 4 * ni - 13;
        regs[4] = ni + (si << 6);
        regs[2] = mask_reg8(regs[2], if sdm == 0 { 0x08 } else { 0x00 }, 0x08);
        regs[5] = (sdm & 0xff) as u8;
        regs[6] = (sdm >> 8) as u8;

        self.write(usb, 0x10, &regs).await?;

        let mut lock_data = [0u8; 3];
        for attempt in 0..2 {
            let data = self.read(usb, 0x00, 3).await?;
            lock_data.copy_from_slice(&data[..3]);
            if lock_data[2] & 0x40 != 0 {
                break;
            }
            if attempt == 0 {
                self.write_reg_mask(usb, 0x12, 0x60, 0xe0).await?;
            }
        }

        self.has_lock = lock_data[2] & 0x40 != 0;
        if self.has_lock {
            self.write_reg_mask(usb, 0x1a, 0x08, 0x08).await?;
        }
        Ok(())
    }

    async fn write_reg(&mut self, usb: &RtlUsb, reg: u8, value: u8) -> Result<()> {
        self.write(usb, reg, &[value]).await
    }

    async fn write_reg_mask(&mut self, usb: &RtlUsb, reg: u8, value: u8, mask: u8) -> Result<()> {
        let current = self.read_cache_reg(reg)?;
        let value = (current & !mask) | (value & mask);
        self.write(usb, reg, &[value]).await
    }

    async fn write(&mut self, usb: &RtlUsb, reg: u8, values: &[u8]) -> Result<()> {
        if self.shadow_equal(reg, values) {
            return Ok(());
        }
        self.shadow_store(reg, values);

        let mut pos = 0usize;
        let mut reg = reg;
        while pos < values.len() {
            let size = (self.max_i2c_msg_len - 1).min(values.len() - pos);
            let mut buf = Vec::with_capacity(size + 1);
            buf.push(reg);
            buf.extend_from_slice(&values[pos..pos + size]);
            usb.i2c_write(self.i2c_addr, &buf).await?;
            reg = reg.wrapping_add(size as u8);
            pos += size;
        }
        Ok(())
    }

    async fn read(&self, usb: &RtlUsb, reg: u8, len: usize) -> Result<Vec<u8>> {
        usb.i2c_write(self.i2c_addr, &[reg]).await?;
        let data = usb.i2c_read(self.i2c_addr, len as u16).await?;
        Ok(data.into_iter().map(bitrev).collect())
    }

    fn read_cache_reg(&self, reg: u8) -> Result<u8> {
        let idx = reg
            .checked_sub(REG_SHADOW_START)
            .ok_or(Error::ArithmeticOverflow)? as usize;
        self.regs.get(idx).copied().ok_or(Error::ArithmeticOverflow)
    }

    fn reg_window<const N: usize>(&self, reg: u8, len: usize) -> Result<[u8; N]> {
        debug_assert_eq!(N, len);
        let idx = reg
            .checked_sub(REG_SHADOW_START)
            .ok_or(Error::ArithmeticOverflow)? as usize;
        let slice = self
            .regs
            .get(idx..idx + len)
            .ok_or(Error::ArithmeticOverflow)?;
        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    fn shadow_equal(&self, reg: u8, values: &[u8]) -> bool {
        let Some(start) = reg.checked_sub(REG_SHADOW_START) else {
            return false;
        };
        let start = start as usize;
        self.regs
            .get(start..start + values.len())
            .is_some_and(|existing| existing == values)
    }

    fn shadow_store(&mut self, reg: u8, values: &[u8]) {
        let mut start = reg as isize - REG_SHADOW_START as isize;
        let mut source_start = 0usize;
        let mut len = values.len();

        if start < 0 {
            let skip = (-start) as usize;
            if skip >= len {
                return;
            }
            source_start = skip;
            len -= skip;
            start = 0;
        }

        let start = start as usize;
        if start >= NUM_REGS {
            return;
        }
        len = len.min(NUM_REGS - start);
        self.regs[start..start + len].copy_from_slice(&values[source_start..source_start + len]);
    }
}

fn mask_reg8(reg: u8, value: u8, mask: u8) -> u8 {
    (reg & !mask) | (value & mask)
}

fn bitrev(byte: u8) -> u8 {
    const LUT: [u8; 16] = [
        0x0, 0x8, 0x4, 0xc, 0x2, 0xa, 0x6, 0xe, 0x1, 0x9, 0x5, 0xd, 0x3, 0xb, 0x7, 0xf,
    ];
    (LUT[(byte & 0x0f) as usize] << 4) | LUT[(byte >> 4) as usize]
}
