#![no_std]
#![no_main]

use defmt::info;
use embassy_stm32::{exti::ExtiInput, gpio::{Level, Output, Pull, Speed}};
use embassy_stm32::i2c::I2c;
use embassy_stm32::bind_interrupts;
use embassy_stm32::Config;
use embassy_time::Timer;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use sh1106::{prelude::*, Builder};
use panic_probe as _;
use defmt_rtt as _;

use embassy_executor::{Spawner, main, task};

// ----------------------------------------
// I2C interrupts for STM32U031
// ----------------------------------------
bind_interrupts!(struct Irqs {
    I2C1 => embassy_stm32::i2c::InterruptHandler<embassy_stm32::peripherals::I2C1>;
});

// ----------------------------------------
// Small adapter: Embassy async I2C → blocking Write impl
// SH1106 driver requires embedded_hal::blocking::i2c::Write
// ----------------------------------------
struct AsyncI2cBlocking<'a, T: embassy_stm32::i2c::Instance>(&'a mut I2c<'a, T>);

impl<'a, T: embassy_stm32::i2c::Instance> embedded_hal::blocking::i2c::Write for AsyncI2cBlocking<'a, T> {
    type Error = embassy_stm32::i2c::Error;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        // We *must* block here, because the SH1106 driver is synchronous.
        // But this happens inside its own task, so it's fine.
        futures::executor::block_on(self.0.write(addr, bytes))
    }
}

// ----------------------------------------
#[main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let led_gpio = Output::new(p.PA5, Level::High, Speed::Medium);

    info!("Starting stm32-gpios");

    spawner.must_spawn(flash_led(led_gpio));

    let b1 = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up);

    spawner.must_spawn(button(b1, "B1"));

        // ----------------------------------------
    // I2C1 on PA9=SCL, PA10=SDA (example pins – change to match your board!)
    // ----------------------------------------
    let mut i2c = I2c::new(
        p.I2C1,
        p.PA9,  // SCL
        p.PA10, // SDA
        Irqs,
        embassy_stm32::i2c::Config::default(),
    );

    // SH1106 expects blocking I2C -> wrap embassy async I2C
    let mut i2c_wrapper = AsyncI2cBlocking(&mut i2c);

    // ----------------------------------------
    // Initialise display
    // ----------------------------------------
    let mut disp: GraphicsMode<_> = Builder::new()
        .with_size(DisplaySize::Display128x64)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_i2c(i2c_wrapper)
        .into();

    disp.init().unwrap();
    disp.clear();

    // ----------------------------------------
    // Draw text with embedded-graphics
    // ----------------------------------------
    let style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

    Text::new("Hello from Embassy!", Point::new(0, 10), style)
        .draw(&mut disp)
        .unwrap();

    disp.flush().unwrap();

    info!("Display updated.");


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