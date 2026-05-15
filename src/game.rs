use esp_hal::gpio::{Output, Level, OutputConfig};
use embassy_time::{Timer};


pub struct GameGpios {
    red: Output<'static>,
    green: Output<'static>,
    blue: Output<'static>,
    yellow: Output<'static>,
}

pub struct GameObject  {
    outputs: GameGpios,
    outputs_arr: [Output<'static>; 4],
    stage: u8,
    player_index: u8,
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
        let red = &mut Output::new(red_gpio, Level::Low, OutputConfig::default());
        let green = Output::new(green_gpio, Level::Low, OutputConfig::default());
        let blue = Output::new(blue_gpio, Level::Low, OutputConfig::default());
        let yellow = Output::new(yellow_gpio, Level::Low, OutputConfig::default());

        Self {
            outputs: GameGpios {
                red,
                green, 
                blue,
                yellow,
            },
        outputs_arr: [
            red,
            green,
            blue,
            yellow
        ],
            stage: 0,
            player_index: 0,
            player_sequence: [0; 10],
            game_sequence: [0; 10],
        }
    }

    pub async fn reset(&mut self) -> bool {
       self.stage = 0;
       self.player_index = 0;
       let mut outputs = [
           &mut self.outputs.red,
           &mut self.outputs.green,
           &mut self.outputs.blue,
           &mut self.outputs.yellow
       ];

       for o in &mut outputs {
            o.set_low();
       }


       for _ in 0..3 {
            for o in &mut outputs {
                o.set_high();
            }
            Timer::after_millis(500).await;

            for o in &mut outputs {
                o.set_low();
            }
            Timer::after_millis(500).await;
        }

       for _ in 0..3 {
            for o in &mut outputs {
                o.set_high();
                Timer::after_millis(250).await;
                o.set_low();
            }
       }
       true
    }

    pub async display_stage(){

    }
}
