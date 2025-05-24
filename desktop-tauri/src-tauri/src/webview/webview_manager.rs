use std::collections::HashMap;
use tauri::{AppHandle, Manager, WebviewBuilder, WebviewUrl, Webview, WebviewWindowBuilder};
use uuid::Uuid;

use crate::tiles::{TileId, WebViewTile};

pub struct WebViewManager {
    webviews: HashMap<String, Webview>,
    tile_to_webview: HashMap<TileId, String>,
}

impl WebViewManager {
    pub fn new() -> Self {
        Self {
            webviews: HashMap::new(),
            tile_to_webview: HashMap::new(),
        }
    }

    pub fn create_webview(&mut self, app: &AppHandle, tile_id: TileId, url: &str, x: f64, y: f64, width: f64, height: f64) -> Result<String, tauri::Error> {
        let webview_id = Uuid::new_v4().to_string();
        
        // Create a hidden webview window
        let webview_window = WebviewWindowBuilder::new(app, &webview_id, WebviewUrl::App("index.html".into()))
            .title("Embedded WebView")
            .inner_size(width, height)
            .position(x, y)
            .visible(false)
            .build()?;
        
        // Navigate to the specified URL
        webview_window.eval(&format!("window.location.href = '{}'", url))?;
        
        // Store the webview and its association with the tile
        self.webviews.insert(webview_id.clone(), webview_window.webview().clone());
        self.tile_to_webview.insert(tile_id, webview_id.clone());
        
        Ok(webview_id)
    }

    pub fn update_webview_position(&self, webview_id: &str, x: f64, y: f64) -> Result<(), tauri::Error> {
        if let Some(webview) = self.webviews.get(webview_id) {
            webview.window().set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))?;
        }
        Ok(())
    }

    pub fn update_webview_size(&self, webview_id: &str, width: f64, height: f64) -> Result<(), tauri::Error> {
        if let Some(webview) = self.webviews.get(webview_id) {
            webview.window().set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))?;
        }
        Ok(())
    }

    pub fn navigate_webview(&self, webview_id: &str, url: &str) -> Result<(), tauri::Error> {
        if let Some(webview) = self.webviews.get(webview_id) {
            webview.eval(&format!("window.location.href = '{}'", url))?;
        }
        Ok(())
    }

    pub fn show_webview(&self, webview_id: &str, visible: bool) -> Result<(), tauri::Error> {
        if let Some(webview) = self.webviews.get(webview_id) {
            if visible {
                webview.window().show()?;
            } else {
                webview.window().hide()?;
            }
        }
        Ok(())
    }

    pub fn remove_webview(&mut self, tile_id: TileId) -> Result<(), tauri::Error> {
        if let Some(webview_id) = self.tile_to_webview.remove(&tile_id) {
            if let Some(webview) = self.webviews.remove(&webview_id) {
                webview.window().close()?;
            }
        }
        Ok(())
    }

    pub fn get_webview_id_for_tile(&self, tile_id: TileId) -> Option<String> {
        self.tile_to_webview.get(&tile_id).cloned()
    }
}