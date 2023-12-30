pub const MAX_ADC_SAMPLE: u16 = (1 << 12) - 1;

pub fn convert_to_millivolts(vref: u32, sample: u16) -> u16 {
    // From http://www.st.com/resource/en/datasheet/DM00071990.pdf
    // 6.3.24 Reference voltage
    const VREFINT_MV: u32 = 1210; // mV

    (u32::from(sample) * VREFINT_MV / vref) as u16
}
