extern crate gilrs;
extern crate rlbot;

pub use gilrs::Gilrs;
use gilrs::{Axis, Button, Event, EventType};

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Gamepad {
    pub rt2: f32,
    pub lt2: f32,
    pub lsx: f32,
    pub lsy: f32,

    pub select_toggled: bool,

    pub rt: bool,
    pub lt: bool,
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

pub fn update_gamepad(gilrs: &mut Gilrs, gamepad: &mut Gamepad) {
    while let Some(Event { event, .. }) = gilrs.next_event() {
        match event {
            EventType::ButtonChanged(button, value, _code) => match button {
                Button::RightTrigger2 => gamepad.rt2 = value,
                Button::LeftTrigger2 => gamepad.lt2 = value,
                _ => {}
            },
            EventType::ButtonPressed(button, _code) => match button {
                Button::Select => gamepad.select_toggled = !gamepad.select_toggled,
                Button::RightTrigger => gamepad.rt = true,
                Button::LeftTrigger => gamepad.lt = true,
                Button::North => gamepad.north = true,
                Button::East => gamepad.east = true,
                Button::South => gamepad.south = true,
                Button::West => gamepad.west = true,
                _ => {}
            },
            EventType::ButtonReleased(button, _code) => match button {
                Button::RightTrigger => gamepad.rt = false,
                Button::LeftTrigger => gamepad.lt = false,
                Button::North => gamepad.north = false,
                Button::East => gamepad.east = false,
                Button::South => gamepad.south = false,
                Button::West => gamepad.west = false,
                _ => {}
            },
            EventType::AxisChanged(axis, value, _code) => match axis {
                Axis::LeftStickX => gamepad.lsx = value,
                Axis::LeftStickY => gamepad.lsy = value,
                _ => {}
            },
            _ => {}
        }
    }
}

/// hard-coded my personal mapping here. someone with a different controller mapping can just
/// ignore this fucntion and make their own.
pub fn human_input(gamepad: &Gamepad) -> rlbot::ControllerState {
    let mut input = rlbot::ControllerState::default();
    input.jump = gamepad.south;
    input.boost = gamepad.east;
    input.handbrake = gamepad.west;
    input.throttle = gamepad.rt2 - gamepad.lt2;
    input.steer = gamepad.lsx;
    input.yaw = gamepad.lsx;
    input.pitch = -gamepad.lsy;

    input.roll = if gamepad.rt {
        1.0
    } else if gamepad.lt {
        -1.0
    } else {
        0.0
    };
    input
}
