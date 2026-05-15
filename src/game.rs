use esp_hal::gpio::{Output, Level, OutputConfig, Input, InputConfig, Pull};
use embassy_time::{Timer};
use esp_hal::rng::{Rng};

pub enum GpioIndexs {
    Red = 0,
    Green = 1,
    Blue = 2,
    Yellow = 3
}

pub struct GameObject  {
    io_arr: [IoPair ;4],
    stage: u8,
    player_index: u8,
    is_buttons_locked: bool,
    player_sequence: [u8; 10],
    game_sequence: [u8; 10]
}

pub struct IoLedMap {
    pub red: esp_hal::peripherals::GPIO7<'static>,
    pub green: esp_hal::peripherals::GPIO6<'static>,
    pub blue: esp_hal::peripherals::GPIO5<'static>,
    pub yellow: esp_hal::peripherals::GPIO4<'static>
}

pub struct IoBtnMap {
    pub red: esp_hal::peripherals::GPIO1<'static>,
    pub green: esp_hal::peripherals::GPIO3<'static>,
    pub blue: esp_hal::peripherals::GPIO2<'static>,
    pub yellow: esp_hal::peripherals::GPIO8<'static>
}

pub struct IoPair {
    led: esp_hal::gpio::Output<'static>,
    btn: esp_hal::gpio::Input<'static>
}

impl GameObject {
    pub fn new(
        leds: IoLedMap,
        btns: IoBtnMap
    ) -> Self {

        let red_led = Output::new(leds.red, Level::Low, OutputConfig::default());
        let green_led = Output::new(leds.green, Level::Low, OutputConfig::default());
        let blue_led = Output::new(leds.blue, Level::Low, OutputConfig::default());
        let yellow_led = Output::new(leds.yellow, Level::Low, OutputConfig::default());

        let red_btn = Input::new(btns.red, InputConfig::default().with_pull(Pull::Up));
        let green_btn = Input::new(btns.green, InputConfig::default().with_pull(Pull::Up));
        let blue_btn = Input::new(btns.blue, InputConfig::default().with_pull(Pull::Up));
        let yellow_btn = Input::new(btns.yellow, InputConfig::default().with_pull(Pull::Up));

        Self {
            io_arr: [
                IoPair {
                    led: red_led,
                    btn: red_btn
                },
                IoPair {
                    led: green_led,
                    btn: green_btn
                }, 
                IoPair {
                    led: blue_led,
                    btn: blue_btn
                },
                IoPair {
                    led: yellow_led,
                    btn: yellow_btn
                },
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
        for _ in 0..self.io_arr.len() {
            self.io_arr[GpioIndexs::Red as usize].led.set_high();
            Timer::after_millis(500).await;
            self.io_arr[GpioIndexs::Red as usize].led.set_low();
            Timer::after_millis(500).await;
        }
    }

    pub async fn set_display_success(self: &mut Self) {
        for _ in 0..3 {
            self.io_arr[GpioIndexs::Green as usize].led.set_high();
            Timer::after_millis(500).await;
            self.io_arr[GpioIndexs::Green as usize].led.set_low();
            Timer::after_millis(500).await;
        }
    }

    async fn set_display_scroll(self: &mut Self) {
       for _ in 0..3 {
            for o in &mut self.io_arr {
                o.led.set_high();
                Timer::after_millis(250).await;
                o.led.set_low();
            }
       }
    }

    async fn set_display_blink(self: &mut Self){
            for o in &mut self.io_arr {
                o.led.set_high();
            }
            Timer::after_millis(500).await;

            for o in &mut self.io_arr {
                o.led.set_low();
            }
            Timer::after_millis(500).await;
    }

    async fn set_display_strobe(self: &mut Self){
       for _ in 0..self.io_arr.len() {
           self.set_display_blink().await;
        }
    }

    fn set_display_off(self: &mut Self){
        for led_index in 0..self.io_arr.len() {
            self.io_arr[led_index as usize].led.set_low();
        }
    }

    pub async fn display_stage(self: &mut Self) {
        let rng = Rng::new();

        self.is_buttons_locked = true;
        self.set_display_off();

        self.set_display_blink().await;

        for sequence in 0..(self.stage + 3) {
            let random_led_index = rng.random() as u8 % 4;
            let io = &mut self.io_arr[random_led_index as usize];
            self.game_sequence[sequence as usize] = random_led_index;
            io.led.set_high();
            Timer::after_millis(500).await;
            io.led.set_low();
            Timer::after_millis(200).await;
        }
        
        self.set_display_blink().await;
        self.is_buttons_locked = false;
    }

    pub async fn handle_button_press(self: &mut Self, io_index: GpioIndexs){
        let io = &mut self.io_arr[io_index as usize];

        loop {
            io.btn.wait_for_falling_edge().await;
            io.led.set_high();

            io.btn.wait_for_rising_edge().await;
            io.led.set_low();
        }
        
    }
}
