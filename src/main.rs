#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::{Can, Envelope, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler};
use embassy_stm32::can::bxcan::{Fifo, Id};
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::gpio::{AnyPin, Level, Output, Pin, Speed};
use embassy_stm32::peripherals::{CAN1};
use embassy_time::Timer;
use static_cell::StaticCell;

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

#[embassy_executor::task]
async fn led_control(can: &'static mut Can<'static, CAN1>, pin: AnyPin) {

    info!("LED Control started");

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {

    let peripherals = embassy_stm32::init(Default::default());

    info!("Initializing CAN");

    static CAN: StaticCell<Can<CAN1>> = StaticCell::new();
    let can = CAN.init(Can::new(peripherals.CAN1, peripherals.PA11, peripherals.PA12, Irqs));

    can.as_mut()
        .modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.as_mut()
        .modify_config()
        .set_loopback(false)
        .set_silent(false)
        .leave_disabled();

    can.set_bitrate(1_000_000);

    can.enable().await;

    info!("CAN initialized");

    spawner.spawn(led_control(can, peripherals.PA5.degrade())).unwrap();

    loop {
        Timer::after_secs(1).await;
    }
}
