#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::{Spawner};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use memory_game::game::{GameObject, IoLedMap, IoBtnMap};
use esp_println::{println};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    println!("PANIC!");
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32c3 -o unstable-hal -o embassy -o wokwi -o neovim -o vscode

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // TODO: Spawn some tasks
    let mut game = GameObject::new(
         IoLedMap {
            red: peripherals.GPIO7,
            green: peripherals.GPIO6,
            blue: peripherals.GPIO5,
            yellow: peripherals.GPIO4,
        },
        IoBtnMap {
            red: peripherals.GPIO1,
            green: peripherals.GPIO3,
            blue: peripherals.GPIO2,
            yellow: peripherals.GPIO8,
        }
    );

    game.reset().await;
    game.display_stage().await;
    
    spawner.spawn(game.handle_button_press(0));

    loop {
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}
