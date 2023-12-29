#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn blink(pin: AnyPin) {

    let mut led = Output::new(pin, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(250).await;

        info!("low");
        led.set_low();
        Timer::after_millis(250).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {

    let peripherals = embassy_stm32::init(Default::default());

    info!("Blinking");

    spawner.spawn(blink(peripherals.PB8.degrade())).unwrap();

    loop {
        Timer::after_millis(500).await;
    }
}
