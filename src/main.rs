#![no_std]
#![no_main]

mod cancom;
mod measure;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Resolution, SampleTime, VrefInt};
use embassy_stm32::can::{Can, Envelope};
use embassy_stm32::can::bxcan::Id;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_stm32::peripherals::{ADC1, CAN1, PA1};
use embassy_time::{Delay, Timer};
use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayUs;
use static_cell::StaticCell;

use crate::cancom::create_can;
use crate::measure::{convert_to_millivolts, MAX_ADC_SAMPLE};

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn led_control(can: &'static mut Can<'static, CAN1>, pin: AnyPin) {

    info!("LED Control task started");

    let mut led = Output::new(pin, Level::High, Speed::Low);
    led.set_low();

    loop {
        let Envelope { ts, frame } = can.read().await.unwrap();
        info!("Received frame @ {:?}: {:?}", ts, frame);

        match frame.id() {
            Id::Extended(id) => {
                match id.as_raw() {
                    0x0001 => led.set_high(),
                    0x0002 => led.set_low(),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[embassy_executor::task]
async fn measure(adc: &'static mut Adc<'static, ADC1>, mut pin: PA1) {

    info!("Measure task started");

    let mut delay = Delay;

    let mut vrefint = adc.enable_vrefint();
    delay.delay_us(VrefInt::start_time_us());

    let vref = u32::from(adc.read(&mut vrefint));

    info!("VCCA: {} mV", convert_to_millivolts(vref, MAX_ADC_SAMPLE));

    info!("Measuring");

    loop {
        let value = adc.read(&mut pin);
        info!("PA1: {} ({} mV)", value, convert_to_millivolts(vref, value));
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {

    let peripherals = embassy_stm32::init(Default::default());


    let can = create_can(peripherals.CAN1, peripherals.PA11, peripherals.PA12);

    can.enable().await;

    let mut delay = Delay;

    static ADC: StaticCell<Adc<ADC1>> = StaticCell::new();
    let adc = ADC.init(Adc::new(peripherals.ADC1, &mut delay));
    adc.set_resolution(Resolution::EightBit);
    adc.set_sample_time(SampleTime::Cycles3);

    spawner.spawn(led_control(can, peripherals.PA5.degrade())).unwrap();
    spawner.spawn(measure(adc, peripherals.PA1)).unwrap();

    loop {
        Timer::after_secs(1).await;
    }
}
