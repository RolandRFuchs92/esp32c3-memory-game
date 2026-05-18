#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use memory_game::game::{
    build_button_inputs, button_task, GameObject, GpioIndex, IoBtnMap, IoLedMap,
};

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
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let [red_btn, green_btn, blue_btn, yellow_btn] = build_button_inputs(IoBtnMap {
        red: peripherals.GPIO1,
        green: peripherals.GPIO3,
        blue: peripherals.GPIO2,
        yellow: peripherals.GPIO8,
    });

    spawner.spawn(button_task(red_btn, GpioIndex::Red).unwrap());
    spawner.spawn(button_task(green_btn, GpioIndex::Green).unwrap());
    spawner.spawn(button_task(blue_btn, GpioIndex::Blue).unwrap());
    spawner.spawn(button_task(yellow_btn, GpioIndex::Yellow).unwrap());

    let mut game = GameObject::new(IoLedMap {
        red: peripherals.GPIO7,
        green: peripherals.GPIO6,
        blue: peripherals.GPIO5,
        yellow: peripherals.GPIO4,
    });

    game.run().await
}
