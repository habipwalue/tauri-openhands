use crate::tiles::{Tile, TileId};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::Camera;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfiniteCanvas {
    pub camera: Camera,
    pub tiles: HashMap<TileId, Tile>,
    pub selected_tile_id: Option<TileId>,
    pub canvas_size: Vec2,
    pub background_color: [f32; 4],
    pub grid_visible: bool,
    pub grid_size: f32,
    pub grid_color: [f32; 4],
}

impl Default for InfiniteCanvas {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            tiles: HashMap::new(),
            selected_tile_id: None,
            canvas_size: Vec2::new(800.0, 600.0),
            background_color: [0.1, 0.1, 0.1, 1.0],
            grid_visible: true,
            grid_size: 50.0,
            grid_color: [0.2, 0.2, 0.2, 1.0],
        }
    }
}

impl InfiniteCanvas {
    pub fn new(canvas_size: Vec2) -> Self {
        Self {
            canvas_size,
            ..Default::default()
        }
    }

    pub fn add_tile(&mut self, tile: Tile) -> TileId {
        let id = TileId(Uuid::new_v4());
        self.tiles.insert(id, tile);
        id
    }

    pub fn remove_tile(&mut self, id: TileId) -> Option<Tile> {
        if self.selected_tile_id == Some(id) {
            self.selected_tile_id = None;
        }
        self.tiles.remove(&id)
    }

    pub fn get_tile(&self, id: TileId) -> Option<&Tile> {
        self.tiles.get(&id)
    }

    pub fn get_tile_mut(&mut self, id: TileId) -> Option<&mut Tile> {
        self.tiles.get_mut(&id)
    }

    pub fn select_tile(&mut self, id: Option<TileId>) {
        self.selected_tile_id = id;
    }

    pub fn get_selected_tile(&self) -> Option<&Tile> {
        self.selected_tile_id.and_then(|id| self.tiles.get(&id))
    }

    pub fn get_selected_tile_mut(&mut self) -> Option<&mut Tile> {
        if let Some(id) = self.selected_tile_id {
            self.tiles.get_mut(&id)
        } else {
            None
        }
    }

    pub fn resize(&mut self, new_size: Vec2) {
        self.canvas_size = new_size;
    }

    pub fn tile_at_position(&self, screen_pos: Vec2) -> Option<TileId> {
        let world_pos = self.camera.screen_to_world(screen_pos, self.canvas_size);
        
        // Check tiles in reverse order (top to bottom in z-order)
        let mut tiles: Vec<(&TileId, &Tile)> = self.tiles.iter().collect();
        tiles.sort_by(|(_, a), (_, b)| b.z_index.partial_cmp(&a.z_index).unwrap());
        
        for (id, tile) in tiles {
            if tile.contains_point(world_pos) {
                return Some(*id);
            }
        }
        
        None
    }

    pub fn update_grid(&mut self, visible: bool, size: f32, color: [f32; 4]) {
        self.grid_visible = visible;
        self.grid_size = size;
        self.grid_color = color;
    }
}