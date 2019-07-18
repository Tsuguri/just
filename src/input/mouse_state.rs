pub struct MouseState
{
    pub mouse_moved : bool,
    pub current_position: [f32; 2],
    pub previous_position: [f32; 2],
    pub button_pressed: [bool; 8]

}

impl MouseState
{
    pub fn get_mouse_move(&self) -> [f32; 2]
    {
        if self.mouse_moved { [self.current_position[0] - self.previous_position[0], self.current_position[1] - self.previous_position[1]] } else { [0.0, 0.0]}
    }

    pub fn left_button_down(&self) -> bool
    {
        self.button_pressed[0]
    }

    pub fn right_button_down(&self) -> bool
    {
        self.button_pressed[1]
    }

    pub fn set_button_state(&mut self, button : usize, value: bool)
    {
        self.button_pressed[button] = value;
    }

    pub fn set_new_position(&mut self, new_position: [f32;2] )
    {
        self.mouse_moved = true;
        self.previous_position = self.current_position;
        self.current_position = new_position;
    }
    pub fn next_frame(&mut self)
    {
        self.mouse_moved = false;
    }
}

impl Default for MouseState
{
    fn default() -> MouseState
    {
        MouseState
            {
                current_position: [0.0; 2],
                previous_position: [0.0; 2],
                button_pressed: [false; 8],
                mouse_moved: false
            }
    }
}
