#![no_std]
#![no_main]

use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use panic_probe as _;
use defmt_rtt as _;

use embassy_executor::{Spawner, main, task};

#[main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let led_gpio = Output::new(p.PA5, Level::High, Speed::Medium);

    spawner.must_spawn(flash_led(led_gpio));

}

#[task]
async fn flash_led(mut gpio: Output<'static>) -> ! {
    loop {
        gpio.set_high();
        Timer::after_millis(500).await;
        gpio.set_low();
        Timer::after_millis(500).await;
    }
}