#![no_main]
#![no_std]

// PMW3360 optical sensor trackball peripheral (id = 2)
// Uses SPI2 on nice!nano v2: SCK=P0_17, MOSI=P0_20, MISO=P0_29, CS=P1_13

use rmk::macros::rmk_peripheral;

mod spim_irq {
    embassy_nrf::bind_interrupts!(
        pub struct Irqs {
            TWISPI1 => embassy_nrf::spim::InterruptHandler<embassy_nrf::peripherals::TWISPI1>;
        }
    );
}

#[rmk_peripheral(id = 2)]
mod keyboard_peripheral {
    use rmk::controller::PollingController;

    #[controller(poll)]
    fn trackball_reader() {
        struct TrackballReader {
            spi: embassy_nrf::spim::Spim<'static>,
            cs: embassy_nrf::gpio::Output<'static>,
        }

        impl rmk::controller::Controller for TrackballReader {
            type Event = rmk::event::ControllerEvent;
            async fn process_event(&mut self, _event: Self::Event) {}
            async fn next_message(&mut self) -> Self::Event {
                core::future::pending().await
            }
        }

        impl PollingController for TrackballReader {
            const INTERVAL: embassy_time::Duration =
                embassy_time::Duration::from_hz(125); // 125 Hz report rate

            async fn update(&mut self) {
                // Read Motion register (0x02). Bit 7 = motion occurred.
                let motion = self.read_reg(0x02).await;
                if motion & 0x80 == 0 {
                    return;
                }

                let dx = self.read_delta_x().await;
                let dy = self.read_delta_y().await;

                if dx == 0 && dy == 0 {
                    return;
                }

                let event = rmk::event::Event::Joystick([
                    rmk::event::AxisEvent {
                        typ: rmk::event::AxisValType::Rel,
                        axis: rmk::event::Axis::X,
                        value: dx,
                    },
                    rmk::event::AxisEvent {
                        typ: rmk::event::AxisValType::Rel,
                        axis: rmk::event::Axis::Y,
                        value: dy,
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

        impl TrackballReader {
            /// PMW3360 SPI read protocol:
            /// 1. CS low
            /// 2. Send address byte (0x80 | reg)
            /// 3. Send dummy byte (0x00) while receiving data on MISO
            /// 4. CS high
            /// 5. Delay ≥120 ns (use 1 µs for safety)
            async fn read_reg(&mut self, reg: u8) -> u8 {
                let tx = [0x80 | reg, 0x00];
                let mut rx = [0u8; 2];
                self.cs.set_low();
                let _ = self.spi.transfer(&mut rx, &tx).await;
                self.cs.set_high();
                embassy_time::Timer::after_micros(1).await;
                rx[1]
            }

            /// PMW3360 SPI write protocol:
            /// 1. CS low
            /// 2. Send address byte (reg, MSB=0 for write)
            /// 3. Send data byte
            /// 4. CS high
            /// 5. Delay ≥120 ns
            async fn write_reg(&mut self, reg: u8, val: u8) {
                let tx = [reg, val];
                self.cs.set_low();
                let _ = self.spi.write(&tx).await;
                self.cs.set_high();
                embassy_time::Timer::after_micros(1).await;
            }

            async fn read_delta_x(&mut self) -> i16 {
                let xl = self.read_reg(0x03).await;
                let xh = self.read_reg(0x04).await;
                ((xh as i16) << 8) | (xl as i16)
            }

            async fn read_delta_y(&mut self) -> i16 {
                let yl = self.read_reg(0x05).await;
                let yh = self.read_reg(0x06).await;
                ((yh as i16) << 8) | (yl as i16)
            }
        }

        // SPI config: Mode 3 (CPOL=1, CPHA=1), 2 MHz
        let mut spim_config = embassy_nrf::spim::Config::default();
        spim_config.frequency = embassy_nrf::spim::Frequency::M2;
        spim_config.mode = embassy_nrf::spim::MODE_3;

        let spim = embassy_nrf::spim::Spim::new(
            p.TWISPI1,
            crate::spim_irq::Irqs,
            p.P0_17, // SCK
            p.P0_29, // MISO
            p.P0_20, // MOSI
            spim_config,
        );

        let cs = embassy_nrf::gpio::Output::new(
            p.P1_13,
            embassy_nrf::gpio::Level::High, // CS active low
            embassy_nrf::gpio::OutputDrive::Standard,
        );

        let mut trackball = TrackballReader { spi: spim, cs };

        // --- PMW3360 initialization ---
        // After power-on, the sensor may need up to 50 ms for SROM to boot.
        // Poll Product ID until it reads 0x42.
        loop {
            let pid = trackball.read_reg(0x00).await;
            if pid == 0x42 {
                break;
            }
            embassy_time::Timer::after_millis(50).await;
        }

        // Set CPI via Config1 (0x0F): value * 100 CPI.
        // 0x10 = 1600 CPI (good starting point for a trackball).
        trackball.write_reg(0x0F, 0x10).await;

        trackball
    }
}
