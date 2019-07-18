use rendy::wsi::winit::VirtualKeyCode as KC;

#[derive(Copy, Clone)]
pub enum KeyCode
{
    Key0 = 0,
    Key1 = 1,
    Key2 = 2,
    Key3 = 3,
    Key4 = 4,
    Key5 = 5,
    Key6 = 6,
    Key7 = 7,
    Key8 = 8,
    Key9 = 9,
    A = 10,
    B = 11,
    C = 12,
    D = 13,
    E = 14,
    F = 15,
    G = 16,
    H = 17,
    I = 18,
    J = 19,
    K = 20,
    L = 21,
    M = 22,
    N = 23,
    O = 24,
    P = 25,
    Q = 26,
    R = 27,
    S = 28,
    T = 29,
    U = 30,
    V = 31,
    W = 32,
    X = 33,
    Y = 34,
    Z = 35,
    Escape = 36,
    F1 = 37,
    F2 = 38,
    F3 = 39,
    F4 = 40,
    F5 = 41,
    F6 = 42,
    F7 = 43,
    F8 = 44,
    F9 = 45,
    F10 = 46,
    F11 = 47,
    F12 = 48,
    F13 = 49,
    F14 = 50,
    F15 = 51,
    Left = 52,
    Up = 53,
    Right = 54,
    Down = 55,
    LControl = 56,
    LShift = 57,
    RControl = 58,
    RShift = 59,
    Space = 60,
    None = 61
}

impl KeyCode
{
    pub fn to_usize(self) ->usize
    {
        match self
            {
                KeyCode::Key0 => 0,
                KeyCode::Key1 => 1,
                KeyCode::Key2 => 2,
                KeyCode::Key3 => 3,
                KeyCode::Key4 => 4,
                KeyCode::Key5 => 5,
                KeyCode::Key6 => 6,
                KeyCode::Key7 => 7,
                KeyCode::Key8 => 8,
                KeyCode::Key9 => 9,
                KeyCode::B => 10,
                KeyCode::A => 11,
                KeyCode::C => 12,
                KeyCode::D => 13,
                KeyCode::E => 14,
                KeyCode::F => 15,
                KeyCode::G => 16,
                KeyCode::H => 17,
                KeyCode::I => 18,
                KeyCode::J => 19,
                KeyCode::K => 20,
                KeyCode::L => 21,
                KeyCode::M => 22,
                KeyCode::N => 23,
                KeyCode::O => 24,
                KeyCode::P => 25,
                KeyCode::Q => 26,
                KeyCode::R => 27,
                KeyCode::S => 28,
                KeyCode::T => 29,
                KeyCode::U => 30,
                KeyCode::V => 31,
                KeyCode::W => 32,
                KeyCode::X => 33,
                KeyCode::Y => 34,
                KeyCode::Z => 35,
                KeyCode::Escape => 36,
                KeyCode::F1 => 37,
                KeyCode::F2 => 38,
                KeyCode::F3 => 39,
                KeyCode::F4 => 40,
                KeyCode::F5 => 41,
                KeyCode::F6 => 42,
                KeyCode::F7 => 43,
                KeyCode::F8 => 44,
                KeyCode::F9 => 45,
                KeyCode::F10 => 46,
                KeyCode::F11 => 47,
                KeyCode::F12 => 48,
                KeyCode::F13 => 49,
                KeyCode::F14 => 50,
                KeyCode::F15 => 51,
                KeyCode::Left => 52,
                KeyCode::Up => 53,
                KeyCode::Right => 54,
                KeyCode::Down => 55,
                KeyCode::LControl => 5,
                KeyCode::LShift => 57,
                KeyCode::RControl => 58,
                KeyCode::RShift => 59,
                KeyCode::Space => 60,
                KeyCode::None => 61
            }
    }
    pub fn from_kc_enum(value: KC) -> KeyCode
    {
        use KC::*;
        match value
            {
                Key0 => KeyCode::Key0,
                Key1 => KeyCode::Key1,
                Key2 => KeyCode::Key2,
                Key3 => KeyCode::Key3,
                Key4 => KeyCode::Key4,
                Key5 => KeyCode::Key5,
                Key6 => KeyCode::Key6,
                Key7 => KeyCode::Key7,
                Key8 => KeyCode::Key8,
                Key9 => KeyCode::Key9,
                B => KeyCode::B,
                A => KeyCode::A,
                C => KeyCode::C,
                D => KeyCode::D,
                E => KeyCode::E,
                F => KeyCode::F,
                G => KeyCode::G,
                H => KeyCode::H,
                I => KeyCode::I,
                J => KeyCode::J,
                K => KeyCode::K,
                L => KeyCode::L,
                M => KeyCode::M,
                N => KeyCode::N,
                O => KeyCode::O,
                P => KeyCode::P,
                Q => KeyCode::Q,
                R => KeyCode::R,
                S => KeyCode::S,
                T => KeyCode::T,
                U => KeyCode::U,
                V => KeyCode::V,
                W => KeyCode::W,
                X => KeyCode::X,
                Y => KeyCode::Y,
                Z => KeyCode::Z,
                Escape => KeyCode::Escape,
                F1 => KeyCode::F1,
                F2 => KeyCode::F2,
                F3 => KeyCode::F3,
                F4 => KeyCode::F4,
                F5 => KeyCode::F5,
                F6 => KeyCode::F6,
                F7 => KeyCode::F7,
                F8 => KeyCode::F8,
                F9 => KeyCode::F9,
                F10 => KeyCode::F10,
                F11 => KeyCode::F11,
                F12 => KeyCode::F12,
                F13 => KeyCode::F13,
                F14 => KeyCode::F14,
                F15 => KeyCode::F15,
                Left => KeyCode::Left,
                Up => KeyCode::Up,
                Right => KeyCode::Right,
                Down => KeyCode::Down,
                LControl => KeyCode::LControl,
                LShift => KeyCode::LShift,
                RControl => KeyCode::RControl,
                RShift => KeyCode::RShift,
                Space => KeyCode::Space,
                _ => KeyCode::None
            }
    }
}

pub struct KeyboardState
{
    keys:  [bool; 256],
    old_keys: [bool; 256]
}

impl KeyboardState
{
    pub fn set_button(&mut self, key: KeyCode, value: bool)
    {
        self.keys[key.to_usize()]=value;
    }

    pub fn is_button_down(&self, key: KeyCode) -> bool
    {
        self.keys[key.to_usize()]
    }

    pub fn button_pressed_in_last_frame(&self, key: KeyCode) -> bool
    {
        let k = key.to_usize();
        self.keys[k] && !self.old_keys[k]
    }

    pub fn next_frame(&mut self)
    {
        self.old_keys.copy_from_slice(&self.keys);
    }
}

impl Default for KeyboardState
{
    fn default() -> KeyboardState
    {
        KeyboardState
            {
                keys: [false; 256],
                old_keys: [false; 256],
            }
    }
}
