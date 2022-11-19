#![cfg_attr(not(test), no_std)]

use embedded_hal::digital::v2::InputPin;

#[derive(Copy, Clone)]
pub enum ButtonState {
    /// Triggered immediately when a button was pressed down
    Down,
    /// Triggered after a button was pressed down and came up again
    Press,
    /// Triggered when button is pressed down and held down for a defined time
    Pressing,
    /// Triggered when button came up again after a defined time
    LongPress,
    /// When button is depressed ;)
    Idle,
}

/// Whether the pin is pulled up or pulled down by default. A button press would pull it into the
/// inverted state
pub enum ButtonPull {
    PullUp,
    PullDown,
}

pub struct ButtonConfig {
    pub pressing_threshold: f32,
    pub long_press_threshold: f32,
    pub pull: ButtonPull,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            /// The time a button press should last to be recognized as a continuous press
            /// (in seconds)
            pressing_threshold: 0.2,
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
    long_press_threshold: u16,
    pin: PIN,
    pressing_threshold: u16,
    pull: ButtonPull,
    reset: bool,
    state: ButtonState,
}

impl<PIN> Button<PIN>
where
    PIN: InputPin,
{
    pub fn new(pin: PIN, f_refresh: u16, config: ButtonConfig) -> Self {
        Self {
            counter: 0,
            debounced: 0,
            long_press_threshold: (f_refresh as f32 / (1.0 / config.long_press_threshold)) as u16,
            pin,
            pressing_threshold: (f_refresh as f32 / (1.0 / config.pressing_threshold)) as u16,
            pull: config.pull,
            reset: false,
            state: ButtonState::Idle,
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
            if self.counter == 0 {
                self.state = ButtonState::Down;
            }
            self.counter = self.counter.wrapping_add(1);
            // Button is being held long enough to be considered a "pressing" action
            if self.counter >= self.pressing_threshold {
                self.state = ButtonState::Pressing;
            }
            // Button was let go of
            if state == 0 {
                if self.reset {
                    self.state = ButtonState::Idle;
                    self.reset = false;
                } else if self.counter <= self.pressing_threshold {
                    self.state = ButtonState::Press;
                } else if self.counter >= self.long_press_threshold {
                    self.state = ButtonState::LongPress;
                } else {
                    self.state = ButtonState::Idle;
                }

                self.counter = 0;
                self.debounced = 0;
            }
        }
    }

    pub fn read(&mut self) -> ButtonState {
        match self.state {
            ButtonState::Pressing | ButtonState::Idle => self.state,
            _ => {
                let state = self.state;
                self.state = ButtonState::Idle;
                state
            }
        }
    }

    pub fn reset(&mut self) {
        self.reset = true;
    }
}
