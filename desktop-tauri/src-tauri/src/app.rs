use std::sync::{Arc, Mutex};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::canvas::InfiniteCanvas;
use crate::tiles::{EguiTile, SkiaTile, Tile, TileId, WebViewTile};
use crate::webview::WebViewManager;

#[derive(Default)]
pub struct AppState {
    pub canvas: Arc<Mutex<InfiniteCanvas>>,
    pub webview_manager: Arc<Mutex<WebViewManager>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileInfo {
    pub id: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
    pub z_index: f32,
    pub tile_type: String,
    pub title: String,
    pub visible: bool,
}

impl From<(&TileId, &Tile)> for TileInfo {
    fn from((id, tile): (&TileId, &Tile)) -> Self {
        Self {
            id: id.0.to_string(),
            position: [tile.position.x, tile.position.y],
            size: [tile.size.x, tile.size.y],
            rotation: tile.rotation,
            z_index: tile.z_index,
            tile_type: format!("{:?}", tile.tile_type),
            title: tile.title.clone(),
            visible: tile.visible,
        }
    }
}

#[tauri::command]
pub fn get_canvas_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    
    let canvas_info = serde_json::json!({
        "size": [canvas.canvas_size.x, canvas.canvas_size.y],
        "camera": {
            "position": [canvas.camera.position.x, canvas.camera.position.y],
            "zoom": canvas.camera.zoom,
            "rotation": canvas.camera.rotation,
        },
        "background_color": canvas.background_color,
        "grid_visible": canvas.grid_visible,
        "grid_size": canvas.grid_size,
        "grid_color": canvas.grid_color,
    });
    
    Ok(canvas_info)
}

#[tauri::command]
pub fn get_tiles(state: State<'_, AppState>) -> Result<Vec<TileInfo>, String> {
    let canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    
    let tiles: Vec<TileInfo> = canvas.tiles.iter()
        .map(|(id, tile)| TileInfo::from((id, tile)))
        .collect();
    
    Ok(tiles)
}

#[tauri::command]
pub fn add_webview_tile(
    app: AppHandle,
    state: State<'_, AppState>,
    url: String,
    position_x: f32,
    position_y: f32,
    width: f32,
    height: f32,
    title: String,
) -> Result<String, String> {
    let position = Vec2::new(position_x, position_y);
    let size = Vec2::new(width, height);
    
    let webview_tile = WebViewTile::new(url.clone());
    let tile = webview_tile.to_tile(position, size, title);
    
    let tile_id = {
        let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
        canvas.add_tile(tile)
    };
    
    // Create the actual webview
    let webview_id = {
        let mut webview_manager = state.webview_manager.lock().map_err(|e| e.to_string())?;
        webview_manager.create_webview(
            &app,
            tile_id,
            &url,
            position_x as f64,
            position_y as f64,
            width as f64,
            height as f64,
        ).map_err(|e| e.to_string())?
    };
    
    // Update the tile with the webview ID
    {
        let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
        if let Some(tile) = canvas.get_tile_mut(tile_id) {
            if let crate::tiles::TileData::WebView { webview_id: id_ref, .. } = &mut tile.data {
                *id_ref = Some(webview_id);
            }
        }
    }
    
    Ok(tile_id.0.to_string())
}

#[tauri::command]
pub fn add_egui_tile(
    state: State<'_, AppState>,
    widget_type: String,
    config: serde_json::Value,
    position_x: f32,
    position_y: f32,
    width: f32,
    height: f32,
    title: String,
) -> Result<String, String> {
    let position = Vec2::new(position_x, position_y);
    let size = Vec2::new(width, height);
    
    let egui_tile = EguiTile::new(widget_type, config);
    let tile = egui_tile.to_tile(position, size, title);
    
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    let tile_id = canvas.add_tile(tile);
    
    Ok(tile_id.0.to_string())
}

#[tauri::command]
pub fn add_skia_tile(
    state: State<'_, AppState>,
    position_x: f32,
    position_y: f32,
    width: f32,
    height: f32,
    title: String,
) -> Result<String, String> {
    let position = Vec2::new(position_x, position_y);
    let size = Vec2::new(width, height);
    
    let skia_tile = SkiaTile::new();
    let tile = skia_tile.to_tile(position, size, title);
    
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    let tile_id = canvas.add_tile(tile);
    
    Ok(tile_id.0.to_string())
}

#[tauri::command]
pub fn remove_tile(
    app: AppHandle,
    state: State<'_, AppState>,
    tile_id_str: String,
) -> Result<(), String> {
    let tile_id = TileId(uuid::Uuid::parse_str(&tile_id_str).map_err(|e| e.to_string())?);
    
    // First, remove any associated webview
    {
        let mut webview_manager = state.webview_manager.lock().map_err(|e| e.to_string())?;
        webview_manager.remove_webview(tile_id).map_err(|e| e.to_string())?;
    }
    
    // Then remove the tile from the canvas
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    canvas.remove_tile(tile_id);
    
    Ok(())
}

#[tauri::command]
pub fn move_tile(
    app: AppHandle,
    state: State<'_, AppState>,
    tile_id_str: String,
    position_x: f32,
    position_y: f32,
) -> Result<(), String> {
    let tile_id = TileId(uuid::Uuid::parse_str(&tile_id_str).map_err(|e| e.to_string())?);
    let new_position = Vec2::new(position_x, position_y);
    
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    
    if let Some(tile) = canvas.get_tile_mut(tile_id) {
        let old_position = tile.position;
        tile.position = new_position;
        
        // If it's a webview, update the actual webview position
        if let crate::tiles::TileData::WebView { webview_id: Some(webview_id), .. } = &tile.data {
            let webview_manager = state.webview_manager.lock().map_err(|e| e.to_string())?;
            webview_manager.update_webview_position(
                webview_id,
                new_position.x as f64,
                new_position.y as f64,
            ).map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn resize_tile(
    app: AppHandle,
    state: State<'_, AppState>,
    tile_id_str: String,
    width: f32,
    height: f32,
) -> Result<(), String> {
    let tile_id = TileId(uuid::Uuid::parse_str(&tile_id_str).map_err(|e| e.to_string())?);
    let new_size = Vec2::new(width, height);
    
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    
    if let Some(tile) = canvas.get_tile_mut(tile_id) {
        tile.size = new_size;
        
        // If it's a webview, update the actual webview size
        if let crate::tiles::TileData::WebView { webview_id: Some(webview_id), .. } = &tile.data {
            let webview_manager = state.webview_manager.lock().map_err(|e| e.to_string())?;
            webview_manager.update_webview_size(
                webview_id,
                width as f64,
                height as f64,
            ).map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn set_tile_visibility(
    app: AppHandle,
    state: State<'_, AppState>,
    tile_id_str: String,
    visible: bool,
) -> Result<(), String> {
    let tile_id = TileId(uuid::Uuid::parse_str(&tile_id_str).map_err(|e| e.to_string())?);
    
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    
    if let Some(tile) = canvas.get_tile_mut(tile_id) {
        tile.visible = visible;
        
        // If it's a webview, update the actual webview visibility
        if let crate::tiles::TileData::WebView { webview_id: Some(webview_id), .. } = &tile.data {
            let webview_manager = state.webview_manager.lock().map_err(|e| e.to_string())?;
            webview_manager.show_webview(webview_id, visible).map_err(|e| e.to_string())?;
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn pan_camera(
    state: State<'_, AppState>,
    delta_x: f32,
    delta_y: f32,
) -> Result<(), String> {
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    canvas.camera.pan(Vec2::new(delta_x, delta_y));
    Ok(())
}

#[tauri::command]
pub fn zoom_camera(
    state: State<'_, AppState>,
    factor: f32,
    target_x: f32,
    target_y: f32,
) -> Result<(), String> {
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    canvas.camera.zoom_at(factor, Vec2::new(target_x, target_y));
    Ok(())
}

#[tauri::command]
pub fn rotate_camera(
    state: State<'_, AppState>,
    angle: f32,
) -> Result<(), String> {
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    canvas.camera.rotate(angle);
    Ok(())
}

#[tauri::command]
pub fn reset_camera(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut canvas = state.canvas.lock().map_err(|e| e.to_string())?;
    canvas.camera = crate::canvas::Camera::default();
    Ok(())
}