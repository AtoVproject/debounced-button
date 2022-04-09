#![cfg_attr(not(test), no_std)]

use embedded_hal::digital::v2::InputPin;

#[derive(Copy, Clone)]
pub enum ButtonState {
    Press,
    LongPress,
}

/// Whether the pin is pulled up or pulled down by default. A button press would pull it into the
/// inverted state
pub enum ButtonPull {
    PullUp,
    PullDown,
}

pub struct ButtonConfig {
    long_press_threshold: f32,
    pull: ButtonPull,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            /// The time a button press should last to be recognized as a long press (in seconds)
            long_press_threshold: 2.0,
            /// The idle button pin state (pulled up or pulled down)
            pull: ButtonPull::PullUp,
        }
    }
}

pub struct Button<PIN> {
    counter: u16,
    debounced: u8,
    pin: PIN,
    pull: ButtonPull,
    state: Option<ButtonState>,
    long_press_threshold: u16,
}

impl<PIN> Button<PIN>
where
    PIN: InputPin,
{
    pub fn new(pin: PIN, f_refresh: u16, config: ButtonConfig) -> Self {
        Self {
            counter: 0,
            debounced: 0,
            pin,
            pull: config.pull,
            state: None,
            long_press_threshold: (f_refresh as f32 / (1.0 / config.long_press_threshold)) as u16,
        }
    }

    fn raw_state(&self) -> u8 {
        match (&self.pull, self.pin.is_low()) {
            (ButtonPull::PullUp, Ok(true)) => 1,
            (ButtonPull::PullDown, Ok(false)) => 1,
            _ => 0,
        }
    }

    pub fn poll(&mut self) {
        let state = self.raw_state();
        self.debounced |= state;
        // Button was pressed at least once
        if self.debounced > 0 {
            self.counter = self.counter.wrapping_add(1);
            // Button was let go of
            if state == 0 {
                if self.counter >= self.long_press_threshold {
                    self.state = Some(ButtonState::LongPress);
                } else {
                    self.state = Some(ButtonState::Press);
                }
                self.debounced = 0;
                self.counter = 0;
            }
        }
    }

    pub fn read(&mut self) -> Option<ButtonState> {
        if let Some(state) = self.state {
            self.state = None;
            return Some(state);
        }
        None
    }
}
