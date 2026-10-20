//! RoKi Trackball hardware: piezo buzzer that chirps on host connect/disconnect.

#![allow(dead_code)]

use embassy_nrf::gpio::Output;
use embassy_time::{Duration, Instant, Timer};

// === Buzzer ===============================================================

/// Bit-banged piezo. Plays R2-D2-ish chirps on connect/disconnect.
pub struct Buzzer {
    pin: Output<'static>,
}

impl Buzzer {
    pub fn new(pin: Output<'static>) -> Self {
        Self { pin }
    }

    pub async fn connected(&mut self) {
        self.beep(3000, 50).await;
        Timer::after_millis(15).await;
        self.beep(2200, 70).await;
        Timer::after_millis(15).await;
        self.beep(1600, 90).await;
        Timer::after_millis(20).await;
        self.beep(1000, 120).await;
    }

    pub async fn disconnected(&mut self) {
        self.beep(2000, 30).await;
        Timer::after_millis(10).await;
        self.beep(2600, 40).await;
        Timer::after_millis(10).await;
        self.beep(3200, 50).await;
        Timer::after_millis(15).await;
        self.beep(2400, 60).await;
    }

    async fn beep(&mut self, frequency_hz: u32, duration_ms: u64) {
        if frequency_hz == 0 {
            Timer::after_millis(duration_ms).await;
            return;
        }
        let half = (1_000_000u64 / frequency_hz as u64) / 2;
        let end = Instant::now() + Duration::from_millis(duration_ms);
        while Instant::now() < end {
            self.pin.set_high();
            Timer::after_micros(half).await;
            self.pin.set_low();
            Timer::after_micros(half).await;
        }
        self.pin.set_low();
    }
}

/// Chirps when a host link (USB or BLE) is established/lost.
#[rmk::macros::processor(subscribe = [rmk::event::ConnectionStatusChangeEvent])]
pub struct HostBuzzer {
    buzzer: Buzzer,
    host_connected: bool,
}

impl HostBuzzer {
    pub fn new(pin: Output<'static>) -> Self {
        Self { buzzer: Buzzer::new(pin), host_connected: false }
    }

    async fn on_connection_status_change_event(&mut self, event: rmk::event::ConnectionStatusChangeEvent) {
        use rmk::types::ble::BleState;
        let connected = event.0.ble.state == BleState::Connected;
        if self.host_connected == connected {
            return;
        }
        self.host_connected = connected;
        if connected {
            self.buzzer.connected().await;
        } else {
            self.buzzer.disconnected().await;
        }
    }
}
