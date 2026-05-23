#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;

#[rmk_peripheral(id = 1)]
mod keyboard_peripheral {
    #[controller(event)]
    fn buzzer_controller() {
        struct BuzzerController<'d> {
            pin: embassy_nrf::gpio::Output<'d>,
        }

        impl<'d> rmk::controller::Controller for BuzzerController<'d> {
            type Event = rmk::event::ControllerEvent;

            async fn process_event(&mut self, event: Self::Event) {
                if let rmk::event::ControllerEvent::SplitCentral(connected) = event {
                    if connected {
                        // === CONNECTED tone: rising, cheerful ===
                        self.beep(1500, 80).await;
                        embassy_time::Timer::after_millis(40).await;
                        self.beep(2000, 120).await;
                    } else {
                        // === DISCONNECTED tone: falling, sad ===
                        self.beep(2000, 80).await;
                        embassy_time::Timer::after_millis(40).await;
                        self.beep(1200, 200).await;
                    }
                }
            }

            async fn next_message(&mut self) -> Self::Event {
                let mut sub = rmk::channel::CONTROLLER_CHANNEL.subscriber().unwrap();
                sub.next_message_pure().await
            }
        }

        impl<'d> BuzzerController<'d> {
            async fn beep(&mut self, frequency_hz: u32, duration_ms: u64) {
                if frequency_hz == 0 {
                    embassy_time::Timer::after_millis(duration_ms).await;
                    return;
                }
                let period_us = 1_000_000u64 / frequency_hz as u64;
                let half = period_us / 2;
                let end = embassy_time::Instant::now()
                    + embassy_time::Duration::from_millis(duration_ms);

                while embassy_time::Instant::now() < end {
                    self.pin.set_high();
                    embassy_time::Timer::after_micros(half).await;
                    self.pin.set_low();
                    embassy_time::Timer::after_micros(half).await;
                }
                self.pin.set_low();
            }
        }

        BuzzerController {
            pin: embassy_nrf::gpio::Output::new(
                p.P0_06,
                embassy_nrf::gpio::Level::Low,
                embassy_nrf::gpio::OutputDrive::Standard,
            ),
        }
    }
}
