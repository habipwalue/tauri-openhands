import { invoke } from "@tauri-apps/api/core";

// Canvas and rendering
let canvas: HTMLCanvasElement;
let ctx: CanvasRenderingContext2D;
let canvasWidth: number;
let canvasHeight: number;

// Camera state
let cameraPosition = { x: 0, y: 0 };
let cameraZoom = 1.0;
let cameraRotation = 0.0;

// Interaction state
let isDragging = false;
let lastMousePosition = { x: 0, y: 0 };
let selectedTileId: string | null = null;
let currentTool = 'pan'; // 'pan', 'select'

// Tiles
interface Tile {
  id: string;
  position: [number, number];
  size: [number, number];
  rotation: number;
  z_index: number;
  tile_type: string;
  title: string;
  visible: boolean;
}

let tiles: Tile[] = [];

// Initialize the application
async function initApp() {
  // Get canvas info from Rust
  try {
    const canvasInfo = await invoke("get_canvas_info");
    cameraPosition.x = canvasInfo.camera.position[0];
    cameraPosition.y = canvasInfo.camera.position[1];
    cameraZoom = canvasInfo.camera.zoom;
    cameraRotation = canvasInfo.camera.rotation;
    
    // Get tiles
    tiles = await invoke("get_tiles");
  } catch (error) {
    console.error("Failed to initialize app:", error);
  }
  
  // Set up canvas
  setupCanvas();
  
  // Set up event listeners
  setupEventListeners();
  
  // Start render loop
  requestAnimationFrame(render);
}

function setupCanvas() {
  canvas = document.getElementById('main-canvas') as HTMLCanvasElement;
  ctx = canvas.getContext('2d')!;
  
  // Set canvas size
  resizeCanvas();
  
  // Listen for window resize
  window.addEventListener('resize', resizeCanvas);
}

function resizeCanvas() {
  const container = document.getElementById('canvas-container')!;
  canvasWidth = container.clientWidth;
  canvasHeight = container.clientHeight;
  
  canvas.width = canvasWidth;
  canvas.height = canvasHeight;
}

function setupEventListeners() {
  // Canvas mouse events
  canvas.addEventListener('mousedown', onMouseDown);
  canvas.addEventListener('mousemove', onMouseMove);
  canvas.addEventListener('mouseup', onMouseUp);
  canvas.addEventListener('wheel', onMouseWheel);
  
  // Tool buttons
  document.getElementById('pan-tool-btn')?.addEventListener('click', () => setTool('pan'));
  document.getElementById('select-tool-btn')?.addEventListener('click', () => setTool('select'));
  document.getElementById('reset-view-btn')?.addEventListener('click', resetView);
  document.getElementById('delete-tile-btn')?.addEventListener('click', deleteSelectedTile);
  
  // Add tile buttons
  document.getElementById('add-webview-btn')?.addEventListener('click', () => showDialog('webview-dialog'));
  document.getElementById('add-egui-btn')?.addEventListener('click', () => showDialog('egui-dialog'));
  document.getElementById('add-skia-btn')?.addEventListener('click', () => showDialog('skia-dialog'));
  
  // Dialog buttons
  document.getElementById('webview-cancel')?.addEventListener('click', () => hideDialog('webview-dialog'));
  document.getElementById('webview-add')?.addEventListener('click', addWebViewTile);
  
  document.getElementById('egui-cancel')?.addEventListener('click', () => hideDialog('egui-dialog'));
  document.getElementById('egui-add')?.addEventListener('click', addEguiTile);
  
  document.getElementById('skia-cancel')?.addEventListener('click', () => hideDialog('skia-dialog'));
  document.getElementById('skia-add')?.addEventListener('click', addSkiaTile);
}

// Event handlers
function onMouseDown(e: MouseEvent) {
  isDragging = true;
  lastMousePosition = { x: e.clientX, y: e.clientY };
  
  if (currentTool === 'select') {
    // Convert screen coordinates to world coordinates
    const worldPos = screenToWorld(e.clientX, e.clientY);
    
    // Check if we clicked on a tile
    let clickedTileId = null;
    for (let i = tiles.length - 1; i >= 0; i--) {
      const tile = tiles[i];
      if (isPointInTile(worldPos.x, worldPos.y, tile)) {
        clickedTileId = tile.id;
        break;
      }
    }
    
    selectedTileId = clickedTileId;
  }
}

function onMouseMove(e: MouseEvent) {
  if (!isDragging) return;
  
  const deltaX = e.clientX - lastMousePosition.x;
  const deltaY = e.clientY - lastMousePosition.y;
  
  if (currentTool === 'pan') {
    // Pan the camera
    cameraPosition.x -= deltaX / cameraZoom;
    cameraPosition.y -= deltaY / cameraZoom;
    
    // Update camera in Rust
    invoke("pan_camera", { deltaX: -deltaX / cameraZoom, deltaY: -deltaY / cameraZoom })
      .catch(err => console.error("Failed to pan camera:", err));
  } else if (currentTool === 'select' && selectedTileId) {
    // Move the selected tile
    const worldDeltaX = deltaX / cameraZoom;
    const worldDeltaY = deltaY / cameraZoom;
    
    // Find the selected tile
    const tile = tiles.find(t => t.id === selectedTileId);
    if (tile) {
      // Update tile position
      tile.position[0] += worldDeltaX;
      tile.position[1] += worldDeltaY;
      
      // Update tile in Rust
      invoke("move_tile", { 
        tileIdStr: selectedTileId, 
        positionX: tile.position[0], 
        positionY: tile.position[1] 
      }).catch(err => console.error("Failed to move tile:", err));
    }
  }
  
  lastMousePosition = { x: e.clientX, y: e.clientY };
  
  // Update status bar
  updateStatusBar();
}

function onMouseUp() {
  isDragging = false;
}

function onMouseWheel(e: WheelEvent) {
  e.preventDefault();
  
  // Calculate zoom factor
  const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
  
  // Get mouse position in world space before zoom
  const mouseX = e.clientX;
  const mouseY = e.clientY;
  
  // Apply zoom
  cameraZoom *= zoomFactor;
  
  // Clamp zoom
  cameraZoom = Math.max(0.1, Math.min(10, cameraZoom));
  
  // Update zoom in Rust
  invoke("zoom_camera", { 
    factor: zoomFactor, 
    targetX: mouseX, 
    targetY: mouseY 
  }).catch(err => console.error("Failed to zoom camera:", err));
  
  // Update status bar
  updateStatusBar();
}

// Tool functions
function setTool(tool: string) {
  currentTool = tool;
  
  // Update UI
  document.getElementById('pan-tool-btn')?.classList.toggle('active', tool === 'pan');
  document.getElementById('select-tool-btn')?.classList.toggle('active', tool === 'select');
  
  // Update cursor
  canvas.style.cursor = tool === 'pan' ? 'grab' : 'default';
}

function resetView() {
  cameraPosition = { x: 0, y: 0 };
  cameraZoom = 1.0;
  cameraRotation = 0.0;
  
  // Update camera in Rust
  invoke("reset_camera").catch(err => console.error("Failed to reset camera:", err));
  
  // Update status bar
  updateStatusBar();
}

function deleteSelectedTile() {
  if (!selectedTileId) return;
  
  // Remove tile in Rust
  invoke("remove_tile", { tileIdStr: selectedTileId })
    .then(() => {
      // Remove from local array
      tiles = tiles.filter(t => t.id !== selectedTileId);
      selectedTileId = null;
    })
    .catch(err => console.error("Failed to delete tile:", err));
}

// Dialog functions
function showDialog(dialogId: string) {
  const dialog = document.getElementById(dialogId);
  if (dialog) {
    dialog.style.display = 'block';
  }
}

function hideDialog(dialogId: string) {
  const dialog = document.getElementById(dialogId);
  if (dialog) {
    dialog.style.display = 'none';
  }
}

// Tile creation functions
async function addWebViewTile() {
  const urlInput = document.getElementById('webview-url') as HTMLInputElement;
  const titleInput = document.getElementById('webview-title') as HTMLInputElement;
  const widthInput = document.getElementById('webview-width') as HTMLInputElement;
  const heightInput = document.getElementById('webview-height') as HTMLInputElement;
  
  const url = urlInput.value;
  const title = titleInput.value;
  const width = parseFloat(widthInput.value);
  const height = parseFloat(heightInput.value);
  
  // Center of the current view
  const centerX = cameraPosition.x;
  const centerY = cameraPosition.y;
  
  try {
    // Add tile in Rust
    const tileId = await invoke("add_webview_tile", {
      url,
      title,
      positionX: centerX,
      positionY: centerY,
      width,
      height
    });
    
    // Refresh tiles
    tiles = await invoke("get_tiles");
    
    // Select the new tile
    selectedTileId = tileId;
    
    // Hide dialog
    hideDialog('webview-dialog');
  } catch (error) {
    console.error("Failed to add WebView tile:", error);
  }
}

async function addEguiTile() {
  const typeSelect = document.getElementById('egui-type') as HTMLSelectElement;
  const titleInput = document.getElementById('egui-title') as HTMLInputElement;
  const widthInput = document.getElementById('egui-width') as HTMLInputElement;
  const heightInput = document.getElementById('egui-height') as HTMLInputElement;
  
  const widgetType = typeSelect.value;
  const title = titleInput.value;
  const width = parseFloat(widthInput.value);
  const height = parseFloat(heightInput.value);
  
  // Center of the current view
  const centerX = cameraPosition.x;
  const centerY = cameraPosition.y;
  
  // Default config based on widget type
  let config = {};
  switch (widgetType) {
    case 'button':
      config = { text: 'Click Me' };
      break;
    case 'slider':
      config = { min: 0, max: 100, value: 50 };
      break;
    case 'checkbox':
      config = { checked: false, text: 'Check me' };
      break;
    case 'textbox':
      config = { text: 'Edit me' };
      break;
    case 'colorpicker':
      config = { color: [1.0, 0.0, 0.0, 1.0] };
      break;
  }
  
  try {
    // Add tile in Rust
    const tileId = await invoke("add_egui_tile", {
      widgetType,
      config,
      title,
      positionX: centerX,
      positionY: centerY,
      width,
      height
    });
    
    // Refresh tiles
    tiles = await invoke("get_tiles");
    
    // Select the new tile
    selectedTileId = tileId;
    
    // Hide dialog
    hideDialog('egui-dialog');
  } catch (error) {
    console.error("Failed to add Egui tile:", error);
  }
}

async function addSkiaTile() {
  const titleInput = document.getElementById('skia-title') as HTMLInputElement;
  const widthInput = document.getElementById('skia-width') as HTMLInputElement;
  const heightInput = document.getElementById('skia-height') as HTMLInputElement;
  
  const title = titleInput.value;
  const width = parseFloat(widthInput.value);
  const height = parseFloat(heightInput.value);
  
  // Center of the current view
  const centerX = cameraPosition.x;
  const centerY = cameraPosition.y;
  
  try {
    // Add tile in Rust
    const tileId = await invoke("add_skia_tile", {
      title,
      positionX: centerX,
      positionY: centerY,
      width,
      height
    });
    
    // Refresh tiles
    tiles = await invoke("get_tiles");
    
    // Select the new tile
    selectedTileId = tileId;
    
    // Hide dialog
    hideDialog('skia-dialog');
  } catch (error) {
    console.error("Failed to add Skia tile:", error);
  }
}

// Rendering
function render() {
  // Clear canvas
  ctx.fillStyle = '#121212';
  ctx.fillRect(0, 0, canvasWidth, canvasHeight);
  
  // Save context state
  ctx.save();
  
  // Apply camera transform
  ctx.translate(canvasWidth / 2, canvasHeight / 2);
  ctx.scale(cameraZoom, cameraZoom);
  ctx.rotate(cameraRotation);
  ctx.translate(-cameraPosition.x, -cameraPosition.y);
  
  // Draw grid
  drawGrid();
  
  // Draw tiles
  drawTiles();
  
  // Restore context state
  ctx.restore();
  
  // Request next frame
  requestAnimationFrame(render);
}

function drawGrid() {
  const gridSize = 50;
  const gridColor = '#1a1a1a';
  
  // Calculate grid bounds based on camera view
  const viewWidth = canvasWidth / cameraZoom;
  const viewHeight = canvasHeight / cameraZoom;
  
  const startX = Math.floor((cameraPosition.x - viewWidth / 2) / gridSize) * gridSize;
  const startY = Math.floor((cameraPosition.y - viewHeight / 2) / gridSize) * gridSize;
  const endX = Math.ceil((cameraPosition.x + viewWidth / 2) / gridSize) * gridSize;
  const endY = Math.ceil((cameraPosition.y + viewHeight / 2) / gridSize) * gridSize;
  
  ctx.strokeStyle = gridColor;
  ctx.lineWidth = 1 / cameraZoom;
  
  // Draw vertical lines
  for (let x = startX; x <= endX; x += gridSize) {
    ctx.beginPath();
    ctx.moveTo(x, startY);
    ctx.lineTo(x, endY);
    ctx.stroke();
  }
  
  // Draw horizontal lines
  for (let y = startY; y <= endY; y += gridSize) {
    ctx.beginPath();
    ctx.moveTo(startX, y);
    ctx.lineTo(endX, y);
    ctx.stroke();
  }
  
  // Draw origin
  ctx.fillStyle = '#ff0000';
  ctx.beginPath();
  ctx.arc(0, 0, 5 / cameraZoom, 0, Math.PI * 2);
  ctx.fill();
}

function drawTiles() {
  // Sort tiles by z-index
  const sortedTiles = [...tiles].sort((a, b) => a.z_index - b.z_index);
  
  for (const tile of sortedTiles) {
    if (!tile.visible) continue;
    
    // Calculate tile bounds
    const x = tile.position[0] - tile.size[0] / 2;
    const y = tile.position[1] - tile.size[1] / 2;
    const width = tile.size[0];
    const height = tile.size[1];
    
    // Save context state
    ctx.save();
    
    // Apply tile transform
    ctx.translate(tile.position[0], tile.position[1]);
    ctx.rotate(tile.rotation);
    ctx.translate(-tile.position[0], -tile.position[1]);
    
    // Draw tile background
    ctx.fillStyle = '#2a2a2a';
    ctx.strokeStyle = tile.id === selectedTileId ? '#0066cc' : '#3a3a3a';
    ctx.lineWidth = 2 / cameraZoom;
    
    // Draw rounded rectangle
    const radius = 6 / cameraZoom;
    ctx.beginPath();
    ctx.moveTo(x + radius, y);
    ctx.lineTo(x + width - radius, y);
    ctx.arcTo(x + width, y, x + width, y + radius, radius);
    ctx.lineTo(x + width, y + height - radius);
    ctx.arcTo(x + width, y + height, x + width - radius, y + height, radius);
    ctx.lineTo(x + radius, y + height);
    ctx.arcTo(x, y + height, x, y + height - radius, radius);
    ctx.lineTo(x, y + radius);
    ctx.arcTo(x, y, x + radius, y, radius);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
    
    // Draw tile header
    const headerHeight = 30 / cameraZoom;
    ctx.fillStyle = '#3a3a3a';
    ctx.beginPath();
    ctx.moveTo(x + radius, y);
    ctx.lineTo(x + width - radius, y);
    ctx.arcTo(x + width, y, x + width, y + radius, radius);
    ctx.lineTo(x + width, y + headerHeight);
    ctx.lineTo(x, y + headerHeight);
    ctx.lineTo(x, y + radius);
    ctx.arcTo(x, y, x + radius, y, radius);
    ctx.closePath();
    ctx.fill();
    
    // Draw tile title
    ctx.fillStyle = '#f6f6f6';
    ctx.font = `${14 / cameraZoom}px sans-serif`;
    ctx.textBaseline = 'middle';
    ctx.fillText(tile.title, x + 10 / cameraZoom, y + headerHeight / 2);
    
    // Draw tile type indicator
    let typeColor = '#f6f6f6';
    switch (tile.tile_type) {
      case 'WebView':
        typeColor = '#4caf50';
        break;
      case 'Egui':
        typeColor = '#2196f3';
        break;
      case 'Skia':
        typeColor = '#ff9800';
        break;
    }
    
    ctx.fillStyle = typeColor;
    ctx.beginPath();
    ctx.arc(
      x + width - 15 / cameraZoom, 
      y + headerHeight / 2, 
      5 / cameraZoom, 
      0, 
      Math.PI * 2
    );
    ctx.fill();
    
    // Draw resize handle if selected
    if (tile.id === selectedTileId) {
      ctx.fillStyle = '#0066cc';
      ctx.beginPath();
      ctx.arc(
        x + width, 
        y + height, 
        5 / cameraZoom, 
        0, 
        Math.PI * 2
      );
      ctx.fill();
    }
    
    // Restore context state
    ctx.restore();
  }
}

// Utility functions
function updateStatusBar() {
  const zoomInfo = document.getElementById('zoom-info');
  const positionInfo = document.getElementById('position-info');
  
  if (zoomInfo) {
    zoomInfo.textContent = `Zoom: ${Math.round(cameraZoom * 100)}%`;
  }
  
  if (positionInfo) {
    positionInfo.textContent = `Position: ${Math.round(cameraPosition.x)}, ${Math.round(cameraPosition.y)}`;
  }
}

function screenToWorld(screenX: number, screenY: number) {
  // Adjust for canvas position
  const rect = canvas.getBoundingClientRect();
  const x = screenX - rect.left;
  const y = screenY - rect.top;
  
  // Apply inverse camera transform
  const worldX = (x - canvasWidth / 2) / cameraZoom + cameraPosition.x;
  const worldY = (y - canvasHeight / 2) / cameraZoom + cameraPosition.y;
  
  return { x: worldX, y: worldY };
}

function isPointInTile(x: number, y: number, tile: Tile) {
  // Simple AABB check (ignoring rotation for simplicity)
  const halfWidth = tile.size[0] / 2;
  const halfHeight = tile.size[1] / 2;
  
  return (
    x >= tile.position[0] - halfWidth &&
    x <= tile.position[0] + halfWidth &&
    y >= tile.position[1] - halfHeight &&
    y <= tile.position[1] + halfHeight
  );
}

// Initialize the app when the DOM is loaded
window.addEventListener("DOMContentLoaded", initApp);
