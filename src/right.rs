#![no_main]
#![no_std]

use rmk::macros::rmk_peripheral;

mod saadc_irq {
    embassy_nrf::bind_interrupts!(pub struct SaadcIrqs {
        SAADC => embassy_nrf::saadc::InterruptHandler;
    });
}

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
                        self.beep(3000, 50).await;
                        embassy_time::Timer::after_millis(15).await;
                        self.beep(2200, 70).await;
                        embassy_time::Timer::after_millis(15).await;
                        self.beep(1600, 90).await;
                        embassy_time::Timer::after_millis(20).await;
                        self.beep(1000, 120).await;
                    } else {
                        self.beep(2000, 30).await;
                        embassy_time::Timer::after_millis(10).await;
                        self.beep(2600, 40).await;
                        embassy_time::Timer::after_millis(10).await;
                        self.beep(3200, 50).await;
                        embassy_time::Timer::after_millis(15).await;
                        self.beep(2400, 60).await;
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

    #[controller(poll)]
    fn joystick_reader() {
        struct JoystickReader {
            adc: embassy_nrf::saadc::Saadc<'static, 2>,
        }

        impl rmk::controller::Controller for JoystickReader {
            type Event = rmk::event::ControllerEvent;
            async fn process_event(&mut self, _event: Self::Event) {}
            async fn next_message(&mut self) -> Self::Event {
                core::future::pending().await
            }
        }

        impl PollingController for JoystickReader {
            const INTERVAL: embassy_time::Duration =
                embassy_time::Duration::from_hz(100);

            async fn update(&mut self) {
                let mut buf = [0i16; 2];
                self.adc.sample(&mut buf).await;

                let raw_x = buf[0] as i32;
                let raw_y = buf[1] as i32;

                // Tune these by observing raw values at rest
                const CENTER_X: i32 = 1900;
                const CENTER_Y: i32 = 1900;
                const SCALE: i32 = 128;

                let x = raw_x - CENTER_X;
                let y = raw_y - CENTER_Y;

                // CW 45° rotation
                const COS: i32 = 7071;
                const SIN: i32 = 7071;
                const DIV: i32 = 10000;

                let x_rot = (x * COS + y * SIN) / DIV;
                let y_rot = (-x * SIN + y * COS) / DIV;

                let x_mouse = (x_rot / SCALE).clamp(-127, 127) as i16;
                let y_mouse = (y_rot / SCALE).clamp(-127, 127) as i16;

                const DEAD_ZONE: i32 = 1;
                let dist_sq = (x_mouse as i32).pow(2) + (y_mouse as i32).pow(2);
                let (x_final, y_final) = if dist_sq < DEAD_ZONE * DEAD_ZONE {
                    (0i16, 0i16)
                } else {
                    (x_mouse, y_mouse)
                };

                let event = rmk::event::Event::Joystick([
                    rmk::event::AxisEvent {
                        typ: rmk::event::AxisValType::Rel,
                        axis: rmk::event::Axis::X,
                        value: x_final,
                    },
                    rmk::event::AxisEvent {
                        typ: rmk::event::AxisValType::Rel,
                        axis: rmk::event::Axis::Y,
                        value: y_final,
                    },
                    rmk::event::AxisEvent {
                        typ: rmk::event::AxisValType::Rel,
                        axis: rmk::event::Axis::Z,
                        value: 0,
                    },
                ]);

                if rmk::channel::EVENT_CHANNEL.is_full() {
                    let _ = rmk::channel::EVENT_CHANNEL.receive().await;
                }
                rmk::channel::EVENT_CHANNEL.send(event).await;
            }
        }

        let adc = embassy_nrf::saadc::Saadc::new(
            p.SAADC,
            crate::saadc_irq::SaadcIrqs,
            embassy_nrf::saadc::Config::default(),
            [
                embassy_nrf::saadc::ChannelConfig::single_ended(p.P0_31.degrade_saadc()),
                embassy_nrf::saadc::ChannelConfig::single_ended(p.P0_29.degrade_saadc()),
            ],
        );

        JoystickReader { adc }
    }
}
