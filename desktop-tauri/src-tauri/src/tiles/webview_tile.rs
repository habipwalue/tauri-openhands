use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::{Tile, TileData, TileType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewTile {
    pub url: String,
    pub webview_id: Option<String>,
}

impl WebViewTile {
    pub fn new(url: String) -> Self {
        Self {
            url,
            webview_id: None,
        }
    }

    pub fn to_tile(self, position: Vec2, size: Vec2, title: String) -> Tile {
        Tile::new(
            position,
            size,
            TileType::WebView,
            TileData::WebView {
                url: self.url,
                webview_id: self.webview_id,
            },
            title,
        )
    }

    pub fn from_tile(tile: &Tile) -> Option<Self> {
        if let TileData::WebView { url, webview_id } = &tile.data {
            Some(Self {
                url: url.clone(),
                webview_id: webview_id.clone(),
            })
        } else {
            None
        }
    }

    pub fn set_webview_id(&mut self, id: String) {
        self.webview_id = Some(id);
    }
}