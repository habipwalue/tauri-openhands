use glam::Vec2;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Tile, TileData, TileType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EguiTile {
    pub widget_type: String,
    pub config: Value,
}

impl EguiTile {
    pub fn new(widget_type: String, config: Value) -> Self {
        Self {
            widget_type,
            config,
        }
    }

    pub fn to_tile(self, position: Vec2, size: Vec2, title: String) -> Tile {
        Tile::new(
            position,
            size,
            TileType::Egui,
            TileData::Egui {
                widget_type: self.widget_type,
                config: self.config,
            },
            title,
        )
    }

    pub fn from_tile(tile: &Tile) -> Option<Self> {
        if let TileData::Egui { widget_type, config } = &tile.data {
            Some(Self {
                widget_type: widget_type.clone(),
                config: config.clone(),
            })
        } else {
            None
        }
    }
}