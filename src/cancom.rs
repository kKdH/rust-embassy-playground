use defmt::info;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::bxcan::Fifo;
use embassy_stm32::can::bxcan::filter::Mask32;
use embassy_stm32::can::{Can, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, TxInterruptHandler};
use embassy_stm32::peripherals::{CAN1, PA11, PA12};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

pub fn create_can(can_peripheral: CAN1, rx: PA11, tx: PA12) -> &'static mut Can<'static, CAN1> {

    info!("Initializing CAN");

    static CAN: StaticCell<Can<CAN1>> = StaticCell::new();
    let can = CAN.init(Can::new(can_peripheral, rx, tx, Irqs));

    can.as_mut()
        .modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.as_mut()
        .modify_config()
        .set_loopback(false)
        .set_silent(false)
        .leave_disabled();

    can.set_bitrate(1_000_000);

    info!("CAN initialized");

    can
}
