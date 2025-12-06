use anyhow::Result;
use rs_ws281x::ChannelBuilder;
use rs_ws281x::Controller;
use rs_ws281x::ControllerBuilder;
use rs_ws281x::StripType;

pub struct Neopixel {
    controller: Controller,
}

impl Neopixel {
    pub fn new() -> Result<Self> {
        let controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                0, // Channel Index
                ChannelBuilder::new()
                    .pin(12) // GPIO 10 = SPI0 MOSI
                    .count(6) // Number of LEDs
                    .strip_type(StripType::Ws2812)
                    .brightness(50) // default: 255
                    .build(),
            )
            .build()
            .unwrap();

        Ok(Self { controller })
    }

    pub fn set_pixels(&mut self, r: u8, g: u8, b: u8, w: u8) -> Result<()> {
        let leds = self.controller.leds_mut(0);
        for led in leds {
            *led = [b, g, r, w]; // This chipset is using BGRW
        }

        self.controller.render().unwrap();

        Ok(())
    }
}

impl Drop for Neopixel {
    fn drop(&mut self) {
        let _ = self.set_pixels(0, 0, 0, 0);
    }
}
