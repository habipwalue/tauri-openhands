use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
        }
    }
}

impl Camera {
    pub fn new(position: Vec2, zoom: f32, rotation: f32) -> Self {
        Self {
            position,
            zoom,
            rotation,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0));
        let rotation = Mat4::from_rotation_z(self.rotation);
        let scale = Mat4::from_scale(Vec3::new(self.zoom, self.zoom, 1.0));
        
        scale * rotation * translation
    }

    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2 {
        // Convert screen coordinates to normalized device coordinates
        let ndc_x = (2.0 * screen_pos.x) / screen_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / screen_size.y;
        
        // Apply inverse view matrix
        let inv_view = self.view_matrix().inverse();
        let world_pos = inv_view.transform_point3(Vec3::new(ndc_x, ndc_y, 0.0));
        
        Vec2::new(world_pos.x, world_pos.y)
    }

    pub fn world_to_screen(&self, world_pos: Vec2, screen_size: Vec2) -> Vec2 {
        // Apply view matrix
        let view = self.view_matrix();
        let ndc = view.transform_point3(Vec3::new(world_pos.x, world_pos.y, 0.0));
        
        // Convert normalized device coordinates to screen coordinates
        let screen_x = (ndc.x + 1.0) * screen_size.x / 2.0;
        let screen_y = (1.0 - ndc.y) * screen_size.y / 2.0;
        
        Vec2::new(screen_x, screen_y)
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.position -= delta / self.zoom;
    }

    pub fn zoom_at(&mut self, factor: f32, target: Vec2) {
        let old_zoom = self.zoom;
        self.zoom = (self.zoom * factor).max(0.1).min(10.0);
        
        // Adjust position to zoom at target point
        let zoom_delta = 1.0 / old_zoom - 1.0 / self.zoom;
        self.position += target * zoom_delta;
    }

    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
        // Normalize rotation to [0, 2π)
        self.rotation = self.rotation % (2.0 * std::f32::consts::PI);
    }
}