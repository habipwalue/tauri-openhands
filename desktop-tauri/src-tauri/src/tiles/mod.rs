mod tile;
mod webview_tile;
mod egui_tile;
mod skia_tile;

pub use tile::{Tile, TileId, TileType, TileData};
pub use webview_tile::WebViewTile;
pub use egui_tile::EguiTile;
pub use skia_tile::SkiaTile;