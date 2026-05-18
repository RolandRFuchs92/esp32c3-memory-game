use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::rng::Rng;
use esp_println::println;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GpioIndex {
    Red = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3,
}

pub struct IoLedMap {
    pub red: esp_hal::peripherals::GPIO7<'static>,
    pub green: esp_hal::peripherals::GPIO6<'static>,
    pub blue: esp_hal::peripherals::GPIO5<'static>,
    pub yellow: esp_hal::peripherals::GPIO4<'static>,
}

pub struct IoBtnMap {
    pub red: esp_hal::peripherals::GPIO1<'static>,
    pub green: esp_hal::peripherals::GPIO3<'static>,
    pub blue: esp_hal::peripherals::GPIO2<'static>,
    pub yellow: esp_hal::peripherals::GPIO8<'static>,
}

#[derive(Clone, Copy)]
pub enum BtnEvent {
    Pressed(GpioIndex),
}

pub static BTN_EVENTS: Channel<CriticalSectionRawMutex, BtnEvent, 8> = Channel::new();

#[embassy_executor::task(pool_size = 4)]
pub async fn button_task(mut btn: Input<'static>, which: GpioIndex) {
    loop {
        btn.wait_for_falling_edge().await;
        BTN_EVENTS.send(BtnEvent::Pressed(which)).await;
        Timer::after_millis(20).await;
        btn.wait_for_rising_edge().await;
        Timer::after_millis(20).await;
    }
}

pub fn build_button_inputs(btns: IoBtnMap) -> [Input<'static>; 4] {
    let cfg = InputConfig::default().with_pull(Pull::Up);
    [
        Input::new(btns.red, cfg),
        Input::new(btns.green, cfg),
        Input::new(btns.blue, cfg),
        Input::new(btns.yellow, cfg),
    ]
}

pub struct GameObject {
    leds: [Output<'static>; 4],
    stage: u8,
    player_index: u8,
    game_sequence: [u8; 10],
    player_sequence: [u8; 10],
}

impl GameObject {
    pub fn new(leds: IoLedMap) -> Self {
        let cfg = OutputConfig::default();
        Self {
            leds: [
                Output::new(leds.red, Level::Low, cfg),
                Output::new(leds.green, Level::Low, cfg),
                Output::new(leds.blue, Level::Low, cfg),
                Output::new(leds.yellow, Level::Low, cfg),
            ],
            stage: 0,
            player_index: 0,
            game_sequence: [0; 10],
            player_sequence: [0; 10],
        }
    }

    pub async fn reset(&mut self) {
        self.stage = 0;
        self.player_index = 0;
        self.set_display_off();
        self.set_display_strobe().await;
        self.set_display_scroll().await;
    }

    pub async fn set_display_fail(&mut self) {
        for _ in 0..3 {
            self.leds[GpioIndex::Red as usize].set_high();
            Timer::after_millis(500).await;
            self.leds[GpioIndex::Red as usize].set_low();
            Timer::after_millis(500).await;
        }
    }

    pub async fn set_display_success(&mut self) {
        for _ in 0..3 {
            self.leds[GpioIndex::Green as usize].set_high();
            Timer::after_millis(500).await;
            self.leds[GpioIndex::Green as usize].set_low();
            Timer::after_millis(500).await;
        }
    }

    async fn set_display_scroll(&mut self) {
        for _ in 0..3 {
            for led in &mut self.leds {
                led.set_high();
                Timer::after_millis(250).await;
                led.set_low();
            }
        }
    }

    async fn set_display_blink(&mut self) {
        for led in &mut self.leds {
            led.set_high();
        }
        Timer::after_millis(500).await;
        for led in &mut self.leds {
            led.set_low();
        }
        Timer::after_millis(500).await;
    }

    async fn set_display_strobe(&mut self) {
        for _ in 0..3 {
            self.set_display_blink().await;
        }
    }

    fn set_display_off(&mut self) {
        for led in &mut self.leds {
            led.set_low();
        }
    }

    pub async fn display_stage(&mut self) {
        let rng = Rng::new();
        self.set_display_off();
        self.set_display_blink().await;

        for sequence in 0..(self.stage + 3) {
            let random_led_index = (rng.random() as u8) % 4;
            self.game_sequence[sequence as usize] = random_led_index;
            let led = &mut self.leds[random_led_index as usize];
            led.set_high();
            Timer::after_millis(500).await;
            led.set_low();
            Timer::after_millis(200).await;
        }

        self.set_display_blink().await;
    }

    pub async fn run(&mut self) -> ! {
        loop {
            self.reset().await;
            loop {
                self.display_stage().await;
                if !self.read_player_sequence().await {
                    self.set_display_fail().await;
                    break;
                }
                self.set_display_success().await;
                self.stage += 1;
                if self.stage as usize >= self.game_sequence.len() {
                    self.set_display_success().await;
                    break;
                }
            }
        }
    }

    async fn read_player_sequence(&mut self) -> bool {
        self.player_index = 0;
        let count = self.stage + 3;
        while self.player_index < count {
            let BtnEvent::Pressed(which) = BTN_EVENTS.receive().await;
            let expected = self.game_sequence[self.player_index as usize];
            let pressed = which as u8;

            let led = &mut self.leds[pressed as usize];
            led.set_high();
            Timer::after_millis(150).await;
            led.set_low();

            if pressed != expected {
                println!("wrong press: got {} expected {}\r", pressed, expected);
                return false;
            }
            self.player_sequence[self.player_index as usize] = pressed;
            self.player_index += 1;
        }
        true
    }
}
