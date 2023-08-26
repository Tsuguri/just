use just_core::math::Vec2;

#[derive(Clone)]
pub struct ViewportData {
    pub camera_lens_height: f32,
    pub height: f32,
    pub width: f32,
    pub ratio: f32,
}

impl ViewportData {
    pub fn viewport_resized(&mut self, new_size: &winit::dpi::PhysicalSize<u32>) {
        self.height = new_size.height as f32;
        self.width = new_size.width as f32;
    }

    pub fn viewport_pos_to_screen_space(&self, pos: &Vec2) -> Vec2 {
        let screen_size = Vec2::new(self.width, self.height);
        return (Vec2::new(pos.x, screen_size.y - pos.y) / screen_size) * 2.0f32 - Vec2::new(1.0, 1.0);
    }
}
