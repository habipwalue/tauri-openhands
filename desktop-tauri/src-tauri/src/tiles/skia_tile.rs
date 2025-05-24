use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::{DrawingCommand, Tile, TileData, TileType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkiaTile {
    pub drawing_commands: Vec<DrawingCommand>,
}

impl SkiaTile {
    pub fn new() -> Self {
        Self {
            drawing_commands: Vec::new(),
        }
    }

    pub fn to_tile(self, position: Vec2, size: Vec2, title: String) -> Tile {
        Tile::new(
            position,
            size,
            TileType::Skia,
            TileData::Skia {
                drawing_commands: self.drawing_commands,
            },
            title,
        )
    }

    pub fn from_tile(tile: &Tile) -> Option<Self> {
        if let TileData::Skia { drawing_commands } = &tile.data {
            Some(Self {
                drawing_commands: drawing_commands.clone(),
            })
        } else {
            None
        }
    }

    pub fn add_command(&mut self, command: DrawingCommand) {
        self.drawing_commands.push(command);
    }

    pub fn clear_commands(&mut self) {
        self.drawing_commands.clear();
    }
}