#![no_std]
#![no_main]

use defmt::info;
use embassy_stm32::{exti::ExtiInput, gpio::{Level, Output, Pull, Speed}};
use embassy_time::Timer;
use panic_probe as _;
use defmt_rtt as _;

use embassy_executor::{Spawner, main, task};

#[main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let led_gpio = Output::new(p.PA5, Level::High, Speed::Medium);

    info!("Starting stm32-gpios");

    spawner.must_spawn(flash_led(led_gpio));

    let b1 = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up);

    spawner.must_spawn(button(b1, "B1"));
}

#[task]
async fn button(mut pin: ExtiInput<'static>, name: &'static str) -> ! {
    loop {
        pin.wait_for_any_edge().await;
        let current_state = pin.is_high();
        info!("{} changed: {}", name, if current_state { "high" } else { "low" } );
    }
}

#[task]
async fn flash_led(mut gpio: Output<'static>) -> ! {
    loop {
        gpio.set_high();
        info!("On");
        Timer::after_millis(500).await;
        gpio.set_low();
        info!("Off");
        Timer::after_millis(500).await;
    }
}