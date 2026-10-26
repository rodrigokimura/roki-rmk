//! Shared RoKi hardware: analog joystick reader and piezo buzzer.
//!
//! Written against the RMK v0.9 event model: the joystick is an
//! `#[input_device]` that publishes `PointingEvent`s, and the buzzer is a
//! `#[processor]` that reacts to connection events.

#![allow(dead_code)]

use embassy_nrf::gpio::Output;
use embassy_nrf::saadc::Saadc;
use embassy_time::{Duration, Instant, Timer};
use rmk::event::{Axis, AxisEvent, AxisValType, BatteryAdcEvent, PointingEvent};
use rmk::macros::{Event, input_device};

// === Joystick =============================================================

/// Which way the stick is physically rotated on the PCB.
#[derive(Clone, Copy)]
pub enum Rotation {
    /// Left half: counter-clockwise 45°
    Ccw45,
    /// Right half: clockwise 45°
    Cw45,
}

/// Events produced by [`JoystickReader`].
#[derive(Event, Clone, Debug)]
pub enum JoystickAdcEvent {
    Pointing(PointingEvent),
    Battery(BatteryAdcEvent),
}

/// Reads a 2-axis analog thumbstick (channels 0/1) plus the battery voltage
/// (channel 2, VDDH/5) from SAADC.
///
/// Mouse deltas are published as `PointingEvent`; centering, scaling,
/// rotation and a circular dead zone are all applied here so the central only
/// needs a pass-through `PointingProcessor`. The battery reading is published
/// as `BatteryAdcEvent` every [`Self::BATTERY_INTERVAL`].
#[input_device(publish = JoystickAdcEvent)]
pub struct JoystickReader {
    adc: Saadc<'static, 3>,
    rotation: Rotation,
    device_id: u8,
    last_battery: Instant,
}

impl JoystickReader {
    // Tune these by observing raw values at rest
    const CENTER_X: i32 = 1900;
    const CENTER_Y: i32 = 1900;
    const SCALE: i32 = 128;
    const DEAD_ZONE: i32 = 2;
    const POLL: Duration = Duration::from_hz(100);
    const BATTERY_INTERVAL: Duration = Duration::from_secs(30);

    pub fn new(adc: Saadc<'static, 3>, rotation: Rotation, device_id: u8) -> Self {
        Self { adc, rotation, device_id, last_battery: Instant::MIN }
    }

    async fn read_joystick_adc_event(&mut self) -> JoystickAdcEvent {
        loop {
            Timer::after(Self::POLL).await;

            let mut buf = [0i16; 3];
            self.adc.sample(&mut buf).await;

            if self.last_battery.elapsed() >= Self::BATTERY_INTERVAL {
                self.last_battery = Instant::now();
                return JoystickAdcEvent::Battery(BatteryAdcEvent(buf[2].max(0) as u16));
            }

            let x = buf[0] as i32 - Self::CENTER_X;
            let y = buf[1] as i32 - Self::CENTER_Y;

            // 45° rotation, cos = sin = 0.7071
            const COS: i32 = 7071;
            const SIN: i32 = 7071;
            const DIV: i32 = 10000;
            let (x_rot, y_rot) = match self.rotation {
                Rotation::Ccw45 => ((x * COS - y * SIN) / DIV, (x * SIN + y * COS) / DIV),
                Rotation::Cw45 => ((x * COS + y * SIN) / DIV, (-x * SIN + y * COS) / DIV),
            };

            let x_mouse = (x_rot / Self::SCALE).clamp(-127, 127) as i16;
            let y_mouse = (y_rot / Self::SCALE).clamp(-127, 127) as i16;

            let dist_sq = (x_mouse as i32).pow(2) + (y_mouse as i32).pow(2);
            if dist_sq < Self::DEAD_ZONE * Self::DEAD_ZONE {
                // Idle: don't flood the split link / HID queue with zero reports
                continue;
            }

            return JoystickAdcEvent::Pointing(PointingEvent {
                device_id: self.device_id,
                axes: [
                    AxisEvent { typ: AxisValType::Rel, axis: Axis::X, value: x_mouse },
                    AxisEvent { typ: AxisValType::Rel, axis: Axis::Y, value: y_mouse },
                    AxisEvent { typ: AxisValType::Rel, axis: Axis::Z, value: 0 },
                ],
            });
        }
    }
}

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
        self.beep(2794, 100).await;
        self.beep(2637, 100).await;
        self.beep(2794, 100).await;
        self.beep(2637, 100).await;
        self.beep(2794, 100).await;
        self.beep(2637, 100).await;
    }

    pub async fn disconnected(&mut self) {
        self.beep(247, 200).await;
        Timer::after_millis(400).await;
        self.beep(247, 200).await;
        Timer::after_millis(400).await;
        self.beep(247, 200).await;
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

/// Peripheral-side buzzer: chirps when the link to the central comes/goes.
#[rmk::macros::processor(subscribe = [rmk::event::CentralConnectedEvent])]
pub struct PeripheralBuzzer {
    buzzer: Buzzer,
    last: Option<bool>,
}

impl PeripheralBuzzer {
    pub fn new(pin: Output<'static>) -> Self {
        Self { buzzer: Buzzer::new(pin), last: None }
    }

    async fn on_central_connected_event(&mut self, event: rmk::event::CentralConnectedEvent) {
        if self.last == Some(event.connected) {
            return;
        }
        self.last = Some(event.connected);
        if event.connected {
            self.buzzer.connected().await;
        } else {
            self.buzzer.disconnected().await;
        }
    }
}

/// Central-side buzzer: chirps when the right half connects/disconnects and
/// when a host link (BLE profile or dongle) is established/lost.
#[rmk::macros::processor(subscribe = [rmk::event::PeripheralConnectedEvent, rmk::event::ConnectionStatusChangeEvent])]
pub struct CentralBuzzer {
    buzzer: Buzzer,
    peripheral_connected: bool,
    host_connected: bool,
}

impl CentralBuzzer {
    pub fn new(pin: Output<'static>) -> Self {
        Self { buzzer: Buzzer::new(pin), peripheral_connected: false, host_connected: false }
    }

    async fn on_peripheral_connected_event(&mut self, event: rmk::event::PeripheralConnectedEvent) {
        if self.peripheral_connected == event.connected {
            return;
        }
        self.peripheral_connected = event.connected;
        if event.connected {
            self.buzzer.connected().await;
        } else {
            self.buzzer.disconnected().await;
        }
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
