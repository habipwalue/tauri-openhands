mod app;
mod canvas;
mod tiles;
mod webview;

use std::sync::{Arc, Mutex};
use glam::Vec2;

use app::AppState;
use canvas::InfiniteCanvas;
use webview::WebViewManager;

// Re-export the app commands
pub use app::{
    get_canvas_info, get_tiles, add_webview_tile, add_egui_tile, add_skia_tile,
    remove_tile, move_tile, resize_tile, set_tile_visibility,
    pan_camera, zoom_camera, rotate_camera, reset_camera,
};

// Legacy command for backward compatibility
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::init();
    
    // Create the app state
    let canvas_size = Vec2::new(800.0, 600.0);
    let canvas = InfiniteCanvas::new(canvas_size);
    let webview_manager = WebViewManager::new();
    
    let app_state = AppState {
        canvas: Arc::new(Mutex::new(canvas)),
        webview_manager: Arc::new(Mutex::new(webview_manager)),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_canvas_info,
            get_tiles,
            add_webview_tile,
            add_egui_tile,
            add_skia_tile,
            remove_tile,
            move_tile,
            resize_tile,
            set_tile_visibility,
            pan_camera,
            zoom_camera,
            rotate_camera,
            reset_camera,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
