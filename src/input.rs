use gru_misc::math::Vec2;
use winit::{window::{Window, CursorGrabMode}, event::{DeviceEvent, WindowEvent}, dpi::PhysicalPosition};
#[cfg(feature = "gru-ui")]
use gru_ui::event::{HardwareEvent, MouseButton, Key};
#[cfg(feature = "gru-ui")]
use winit::{event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta}, keyboard::{PhysicalKey, KeyCode}};

pub enum RawEvent
{
    Device(DeviceEvent),
    Window(WindowEvent),
}

pub struct Input
{
    cam_mode: bool,
    pub pointer_pos: Vec2,
    #[cfg(not(feature = "gru-ui"))]
    events: Vec<RawEvent>,
    #[cfg(feature = "gru-ui")]
    events: Vec<HardwareEvent>,
}

impl Input
{
    pub(crate) fn new() -> Self
    {
        Self
        {
            cam_mode: false,
            pointer_pos: Vec2(0.0, 0.0),
            events: Vec::new(),
        }
    }

    pub(crate) fn event(&mut self, event: RawEvent)
    {
        #[cfg(not(feature = "gru-ui"))]
        {
            if let RawEvent::Window(WindowEvent::CursorMoved { position, .. }) = &event && !self.cam_mode
            {
                self.pointer_pos = Vec2(position.x as f32, position.y as f32);
            }
            self.events.push(event);
        }
        #[cfg(feature = "gru-ui")]
        convert(self.cam_mode, &mut self.pointer_pos, &event, |event| self.events.push(event));
    }

    pub(crate) fn clear(&mut self)
    {
        self.events.clear();
    }

    #[cfg(not(feature = "gru-ui"))]
    pub fn events(&self) -> &[RawEvent]
    {
        &self.events
    }

    #[cfg(feature = "gru-ui")]
    pub fn events(&self) -> &[HardwareEvent]
    {
        &self.events
    }

    pub fn mouse_cam_mode(&mut self, window: &Window, enable: bool)
    {
        if enable
        {
            window.set_cursor_visible(false);
            window.set_cursor_grab(CursorGrabMode::Confined).unwrap();
        } else
        {
            window.set_cursor_grab(CursorGrabMode::None).unwrap();
            window.set_cursor_visible(true);
            window.set_cursor_position(PhysicalPosition::new(self.pointer_pos.0 as f64, self.pointer_pos.1 as f64)).unwrap();
        }
        self.cam_mode = enable;
    }
}

#[cfg(feature = "gru-ui")]
fn convert(cam_mode: bool, pointer_pos: &mut Vec2, raw_event: &RawEvent, mut accept: impl FnMut(HardwareEvent))
{
    match raw_event
    {
        RawEvent::Device(event) => match event
        {
            DeviceEvent::MouseMotion { delta } if cam_mode =>
            {
                let delta = Vec2(delta.0 as f32, delta.1 as f32);
                let event = HardwareEvent::RawMouseDelta(delta);
                accept(event);
            },
            _ => {},
        },
        RawEvent::Window(event) => match event
        {
            WindowEvent::CloseRequested => accept(HardwareEvent::CloseWindow),
            WindowEvent::CursorMoved { position, .. } if !cam_mode =>
            {
                let new_pos = Vec2(position.x as f32, position.y as f32);
                let delta = new_pos - *pointer_pos;
                *pointer_pos = new_pos;
                let event = HardwareEvent::PointerMoved { pos: *pointer_pos, delta };
                accept(event);
            },
            WindowEvent::MouseInput { state, button, .. } =>
            {
                let button = match button
                {
                    WinitMouseButton::Left => MouseButton::Primary,
                    WinitMouseButton::Right => MouseButton::Secondary,
                    WinitMouseButton::Middle => MouseButton::Terciary,
                    _ => MouseButton::Terciary,
                };
                let event = HardwareEvent::PointerClicked { pos: *pointer_pos, button, pressed: *state == ElementState::Pressed };
                accept(event);
            },
            WindowEvent::CursorLeft { .. } => accept(HardwareEvent::PointerGone),
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(dx, dy), .. } => accept(HardwareEvent::Scroll { pos: *pointer_pos, delta: Vec2(*dx, *dy) }),
            WindowEvent::KeyboardInput { event, .. } =>
            {
                if let PhysicalKey::Code(keycode) = event.physical_key
                {
                    let key = match keycode
                    {
                        KeyCode::Digit1 => Some(Key::Key1),
                        KeyCode::Digit3 => Some(Key::Key3),
                        KeyCode::Digit2 => Some(Key::Key2),
                        KeyCode::Digit4 => Some(Key::Key4),
                        KeyCode::Digit5 => Some(Key::Key5),
                        KeyCode::Digit6 => Some(Key::Key6),
                        KeyCode::Digit7 => Some(Key::Key7),
                        KeyCode::Digit8 => Some(Key::Key8),
                        KeyCode::Digit9 => Some(Key::Key9),
                        KeyCode::Digit0 => Some(Key::Key0),
                        KeyCode::KeyA => Some(Key::A),
                        KeyCode::KeyB => Some(Key::B),
                        KeyCode::KeyC => Some(Key::C),
                        KeyCode::KeyD => Some(Key::D),
                        KeyCode::KeyE => Some(Key::E),
                        KeyCode::KeyF => Some(Key::F),
                        KeyCode::KeyG => Some(Key::G),
                        KeyCode::KeyH => Some(Key::H),
                        KeyCode::KeyI => Some(Key::I),
                        KeyCode::KeyJ => Some(Key::J),
                        KeyCode::KeyK => Some(Key::K),
                        KeyCode::KeyL => Some(Key::L),
                        KeyCode::KeyM => Some(Key::M),
                        KeyCode::KeyN => Some(Key::N),
                        KeyCode::KeyO => Some(Key::O),
                        KeyCode::KeyP => Some(Key::P),
                        KeyCode::KeyQ => Some(Key::Q),
                        KeyCode::KeyR => Some(Key::R),
                        KeyCode::KeyS => Some(Key::S),
                        KeyCode::KeyT => Some(Key::T),
                        KeyCode::KeyU => Some(Key::U),
                        KeyCode::KeyV => Some(Key::V),
                        KeyCode::KeyW => Some(Key::W),
                        KeyCode::KeyX => Some(Key::X),
                        KeyCode::KeyY => Some(Key::Y),
                        KeyCode::KeyZ => Some(Key::Z),
                        KeyCode::Escape => Some(Key::Escape),
                        KeyCode::F1 => Some(Key::F1),
                        KeyCode::F2 => Some(Key::F2),
                        KeyCode::F3 => Some(Key::F3),
                        KeyCode::F4 => Some(Key::F4),
                        KeyCode::F5 => Some(Key::F5),
                        KeyCode::F6 => Some(Key::F6),
                        KeyCode::F7 => Some(Key::F7),
                        KeyCode::F8 => Some(Key::F8),
                        KeyCode::F9 => Some(Key::F9),
                        KeyCode::F10 => Some(Key::F10),
                        KeyCode::F11 => Some(Key::F11),
                        KeyCode::F12 => Some(Key::F12),
                        KeyCode::F13 => Some(Key::F13),
                        KeyCode::F14 => Some(Key::F14),
                        KeyCode::F15 => Some(Key::F15),
                        KeyCode::F16 => Some(Key::F16),
                        KeyCode::F17 => Some(Key::F17),
                        KeyCode::F18 => Some(Key::F18),
                        KeyCode::F19 => Some(Key::F19),
                        KeyCode::F20 => Some(Key::F20),
                        KeyCode::F21 => Some(Key::F21),
                        KeyCode::F22 => Some(Key::F22),
                        KeyCode::F23 => Some(Key::F23),
                        KeyCode::F24 => Some(Key::F24),
                        KeyCode::Pause => Some(Key::Pause),
                        KeyCode::Insert => Some(Key::Insert),
                        KeyCode::Home => Some(Key::Home),
                        KeyCode::Delete => Some(Key::Delete),
                        KeyCode::End => Some(Key::End),
                        KeyCode::PageDown => Some(Key::PageDown),
                        KeyCode::PageUp => Some(Key::PageUp),
                        KeyCode::ArrowLeft => Some(Key::Left),
                        KeyCode::ArrowUp => Some(Key::Up),
                        KeyCode::ArrowRight => Some(Key::Right),
                        KeyCode::ArrowDown => Some(Key::Down),
                        KeyCode::Backspace => Some(Key::Back),
                        KeyCode::Enter => Some(Key::Return),
                        KeyCode::Space => Some(Key::Space),
                        KeyCode::NumLock => Some(Key::Numlock),
                        KeyCode::Numpad0 => Some(Key::Numpad0),
                        KeyCode::Numpad1 => Some(Key::Numpad1),
                        KeyCode::Numpad2 => Some(Key::Numpad2),
                        KeyCode::Numpad3 => Some(Key::Numpad3),
                        KeyCode::Numpad4 => Some(Key::Numpad4),
                        KeyCode::Numpad5 => Some(Key::Numpad5),
                        KeyCode::Numpad6 => Some(Key::Numpad6),
                        KeyCode::Numpad7 => Some(Key::Numpad7),
                        KeyCode::Numpad8 => Some(Key::Numpad8),
                        KeyCode::Numpad9 => Some(Key::Numpad9),
                        KeyCode::NumpadAdd => Some(Key::NumpadAdd),
                        KeyCode::NumpadDivide => Some(Key::NumpadDivide),
                        KeyCode::NumpadDecimal => Some(Key::NumpadDecimal),
                        KeyCode::NumpadComma => Some(Key::NumpadComma),
                        KeyCode::NumpadEnter => Some(Key::NumpadEnter),
                        KeyCode::NumpadEqual => Some(Key::NumpadEquals),
                        KeyCode::NumpadMultiply => Some(Key::NumpadMultiply),
                        KeyCode::NumpadSubtract => Some(Key::NumpadSubtract),
                        KeyCode::AltLeft => Some(Key::LAlt),
                        KeyCode::ControlLeft => Some(Key::LControl),
                        KeyCode::ShiftLeft => Some(Key::LShift),
                        KeyCode::AltRight => Some(Key::RAlt),
                        KeyCode::ControlRight => Some(Key::RControl),
                        KeyCode::ShiftRight => Some(Key::RShift),
                        KeyCode::Tab => Some(Key::Tab),
                        _ => None
                    };
                    if let Some(key) = key
                    {
                        let event = HardwareEvent::Key { key, pressed: event.state == ElementState::Pressed };
                        accept(event);
                    }
                    if let Some(text) = &event.text
                    {
                        let ch = text.chars().next().unwrap();
                        let event = HardwareEvent::Char(ch);
                        accept(event);
                    }
                }
            },
            _ => {}
        }
    }
}
