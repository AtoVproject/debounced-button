#![cfg_attr(not(test), no_std)]
#![no_main]

use embedded_hal::digital::v2::InputPin;

#[derive(Copy, Clone)]
pub enum ButtonState {
    Press,
    LongPress,
}

pub enum ButtonPull {
    PullUp,
    PullDown,
}

pub struct ButtonConfig {
    press_threshold: f32,
    long_press_threshold: f32,
    pull: ButtonPull,
}

pub struct Button<PIN> {
    counter: u16,
    debounced: u8,
    pin: PIN,
    pull: ButtonPull,
    state: Option<ButtonState>,
    press_threshold: u16,
    long_press_threshold: u16,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            // TODO: maybe remove this
            /// The time a button press should last to be recognized (in seconds)
            press_threshold: 0.01,
            /// The time a button press should last to be recognized as a long press (in seconds)
            long_press_threshold: 2.0,
            pull: ButtonPull::PullUp,
        }
    }
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
            press_threshold: (f_refresh as f32 / (1.0 / config.press_threshold)) as u16,
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
                } else if self.counter >= self.press_threshold {
                    self.state = Some(ButtonState::Press);
                }
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

#[cfg(test)]
mod tests {
    use super::*;
}
