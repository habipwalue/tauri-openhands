use glam::Vec2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    WebView,
    Egui,
    Skia,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TileData {
    WebView {
        url: String,
        webview_id: Option<String>,
    },
    Egui {
        widget_type: String,
        config: serde_json::Value,
    },
    Skia {
        drawing_commands: Vec<DrawingCommand>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrawingCommand {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadTo(Vec2, Vec2),
    CubicTo(Vec2, Vec2, Vec2),
    ClosePath,
    SetFillColor([f32; 4]),
    SetStrokeColor([f32; 4]),
    SetStrokeWidth(f32),
    Fill,
    Stroke,
    DrawRect(Vec2, Vec2),
    DrawCircle(Vec2, f32),
    DrawText(String, Vec2, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub z_index: f32,
    pub tile_type: TileType,
    pub data: TileData,
    pub title: String,
    pub resizable: bool,
    pub movable: bool,
    pub visible: bool,
}

impl Tile {
    pub fn new(
        position: Vec2,
        size: Vec2,
        tile_type: TileType,
        data: TileData,
        title: String,
    ) -> Self {
        Self {
            position,
            size,
            rotation: 0.0,
            z_index: 0.0,
            tile_type,
            data,
            title,
            resizable: true,
            movable: true,
            visible: true,
        }
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        // For simplicity, we're ignoring rotation here
        let half_size = self.size * 0.5;
        let min = self.position - half_size;
        let max = self.position + half_size;
        
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }

    pub fn move_by(&mut self, delta: Vec2) {
        if self.movable {
            self.position += delta;
        }
    }

    pub fn resize(&mut self, new_size: Vec2) {
        if self.resizable {
            self.size = new_size.max(Vec2::new(50.0, 50.0)); // Minimum size
        }
    }

    pub fn set_z_index(&mut self, z_index: f32) {
        self.z_index = z_index;
    }

    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
        // Normalize rotation to [0, 2π)
        self.rotation = self.rotation % (2.0 * std::f32::consts::PI);
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.visible = visible;
    }
}