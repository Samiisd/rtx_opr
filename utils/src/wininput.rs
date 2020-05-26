use glutin::event::{DeviceEvent, ElementState, KeyboardInput, MouseScrollDelta, VirtualKeyCode, MouseButton};
use nalgebra::Vector2;

pub enum StateChange {
    Keyboard,
    MouseButton,
    MouseScroll,
    MouseMotion,
}

const CST_MAX_NUMBER_STATE_CHANGE: usize = 4;
const CST_MAX_NUMBER_KEY: usize = u8::max_value() as usize;
const CST_MAX_NUMBER_BUTTON: usize = u8::max_value() as usize;

type StateChangeArray = [bool; CST_MAX_NUMBER_STATE_CHANGE];
type KeyStateArray = [ElementState; CST_MAX_NUMBER_KEY];
type ButtonStateArray = [ElementState; CST_MAX_NUMBER_BUTTON];

pub struct WinInput {
    states_changes: StateChangeArray,

    mouse_scroll: f32,
    mouse_scroll_sensitivity: f32,

    mouse_motion_offset: Vector2<f32>,
    mouse_motion_sensitivity: f32,

    keys_states: KeyStateArray,
    buttons_states: ButtonStateArray,
}

impl WinInput {
    fn set_state_updated(&mut self, state: StateChange) {
        self.states_changes[state as usize] = true;
    }

    pub fn updated(&mut self, state: StateChange) -> bool {
        let k = state as usize;
        if self.states_changes[k] {
            self.states_changes[k] = false;
            true
        } else {
            false
        }
    }

    fn handle_mouse_wheel(&mut self, input: MouseScrollDelta) {
        match input {
            MouseScrollDelta::LineDelta(dx, dy) => {
                self.mouse_scroll += self.mouse_scroll_sensitivity * (dx + dy);
                self.mouse_scroll = self.mouse_scroll.clamp(0., 1.);

                // FIXME: should update state only if scroll change from previous
                self.set_state_updated(StateChange::MouseScroll);
                // FIXME-END
            }
            _ => (),
        }
    }

    fn handle_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.mouse_motion_offset = Vector2::new(dx, -dy) * self.mouse_motion_sensitivity;
        self.set_state_updated(StateChange::MouseMotion);
    }

    pub fn get_scroll(&self) -> f32 {
        self.mouse_scroll
    }

    pub fn get_mouse_offset(&self) -> Vector2<f32> {
        self.mouse_motion_offset
    }

    pub fn is_pressed(&self, k: VirtualKeyCode) -> bool {
        self.keys_states[k as usize] == ElementState::Pressed
    }

    fn button_id(&self, button: MouseButton) -> usize {
        match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Other(v) => v as usize,
        }
    }

    pub fn is_button_pressed(&self, k: MouseButton) -> bool {
        self.buttons_states[self.button_id(k)] == ElementState::Pressed
    }

    pub fn on_device_event(&mut self, input: DeviceEvent) {
        match input {
            DeviceEvent::MouseWheel { delta } => self.handle_mouse_wheel(delta),
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                self.handle_mouse_motion(dx as f32, dy as f32)
            }
            _ => (),
        }
    }

    pub fn on_mouse_input(&mut self, input: MouseButton, state: ElementState) {
        let k = self.button_id(input);

        if state != self.buttons_states[k] {
            self.buttons_states[k] = state;
            self.set_state_updated(StateChange::MouseButton);
        }
    }

    pub fn on_keyboard_input(&mut self, input: KeyboardInput) {
        if let Some(k) = input.virtual_keycode {
            let k = k as usize;
            if input.state != self.keys_states[k] {
                self.keys_states[k] = input.state;
                self.set_state_updated(StateChange::Keyboard);
            }
        }
    }

    pub fn new(mouse_scroll_sensitivity: f32, mouse_motion_sensitivity: f32) -> Self {
        WinInput {
            mouse_scroll: 0.5,
            mouse_scroll_sensitivity,

            mouse_motion_offset: Vector2::zeros(),
            mouse_motion_sensitivity,

            keys_states: [ElementState::Released; CST_MAX_NUMBER_KEY],
            buttons_states: [ElementState::Released; CST_MAX_NUMBER_BUTTON],
            states_changes: [false; CST_MAX_NUMBER_STATE_CHANGE],
        }
    }
}

impl Default for WinInput {
    fn default() -> Self {
        WinInput::new(0.005, 1.)
    }
}
