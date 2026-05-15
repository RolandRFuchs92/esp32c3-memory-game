use esp_hal::gpio::{Output, Level, OutputConfig};
use embassy_time::{Timer};
use esp_hal::rng::{Rng};
use esp_println::{println};

pub enum Gpios {
    Red = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3
}

pub struct GameObject  {
    led_arr: [Output<'static>; 4],
    stage: u8,
    player_index: u8,
    is_buttons_locked: bool,
    player_sequence: [u8; 10],
    game_sequence: [u8; 10]
}


impl GameObject {
    pub fn new(
        red_gpio: esp_hal::peripherals::GPIO7<'static>,
        green_gpio: esp_hal::peripherals::GPIO6<'static>,
        blue_gpio: esp_hal::peripherals::GPIO5<'static>,
        yellow_gpio: esp_hal::peripherals::GPIO4<'static>
        ) -> Self {

        let red = Output::new(red_gpio, Level::Low, OutputConfig::default());
        let green = Output::new(green_gpio, Level::Low, OutputConfig::default());
        let blue = Output::new(blue_gpio, Level::Low, OutputConfig::default());
        let yellow = Output::new(yellow_gpio, Level::Low, OutputConfig::default());

        Self {
            led_arr: [
                red,
                green,
                blue,
                yellow
            ],
            stage: 0,
            is_buttons_locked: true,
            player_index: 0,
            player_sequence: [0; 10],
            game_sequence: [0; 10],
        }
    }

    pub async fn reset(self: &mut Self) {
       self.stage = 0;
       self.player_index = 0;
       self.set_display_off();
       self.set_display_strobe().await;
       self.set_display_scroll().await;
    }

    pub async fn set_display_fail(self: &mut Self) {
        for _ in 0..3 {
            self.led_arr[Gpios::Red as usize].set_high();
            Timer::after_millis(500).await;
            self.led_arr[Gpios::Red as usize].set_low();
            Timer::after_millis(500).await;
        }
    }

    pub async fn set_display_success(self: &mut Self) {
        for _ in 0..3 {
            self.led_arr[Gpios::Green as usize].set_high();
            Timer::after_millis(500).await;
            self.led_arr[Gpios::Green as usize].set_low();
            Timer::after_millis(500).await;
        }
    }

    async fn set_display_scroll(self: &mut Self) {
       for _ in 0..3 {
            for o in &mut self.led_arr {
                o.set_high();
                Timer::after_millis(250).await;
                o.set_low();
            }
       }
    }

    async fn set_display_strobe(self: &mut Self){
       for _ in 0..3 {
            for o in &mut self.led_arr {
                o.set_high();
            }
            Timer::after_millis(500).await;

            for o in &mut self.led_arr {
                o.set_low();
            }
            Timer::after_millis(500).await;
        }
    }

    fn set_display_off(self: &mut Self){
        for led_index in 0..3 {
            self.led_arr[led_index as usize].set_low();
        }
    }

    pub async fn display_stage(self: &mut Self) {
        let rng = Rng::new();

        self.is_buttons_locked = true;
        self.set_display_off();

        for sequence in 0..(self.stage + 3) {
            let random_led_index = rng.random() as u8 % 4;
            let led = &mut self.led_arr[random_led_index as usize];
            self.game_sequence[sequence as usize] = random_led_index;
            led.set_high();
            Timer::after_millis(500).await;
            led.set_low();
            Timer::after_millis(200).await;
        }
    }
}
