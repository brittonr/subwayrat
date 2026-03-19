//! Infinite canvas viewport with pan/zoom and coordinate mapping.
//!
//! This crate provides coordinate math for infinite canvas widgets - mapping between
//! screen coordinates (visible terminal area) and canvas coordinates (infinite space).
//! It has no dependency on ratatui; rendering is the caller's responsibility.
//!
//! ## Types
//!
//! - [`Position`]: A coordinate on an infinite canvas (can be negative)
//! - [`Viewport`]: Pan/zoom camera for viewing the canvas
//!
//! ## Example
//!
//! ```rust
//! use rat_canvas::{Position, Viewport};
//!
//! let mut viewport = Viewport::new(80, 24);
//! 
//! // Convert screen click to canvas position
//! let canvas_pos = viewport.screen_to_canvas(10, 5);
//! assert_eq!(canvas_pos, Position::new(10, 5));
//!
//! // Pan the viewport
//! viewport.pan(50, 25);
//! let canvas_pos = viewport.screen_to_canvas(10, 5);
//! assert_eq!(canvas_pos, Position::new(60, 30));
//! ```

/// A position on the canvas (can be negative for infinite canvas).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// Create a new position.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Minimum zoom level (zoomed out).
pub const MIN_ZOOM: f32 = 0.25;

/// Maximum zoom level (zoomed in).
pub const MAX_ZOOM: f32 = 4.0;

/// Zoom step for zoom in/out operations.
pub const ZOOM_STEP: f32 = 0.25;

// Compile-time assertions for zoom constants
const _: () = assert!(MIN_ZOOM > 0.0, "MIN_ZOOM must be positive");
const _: () = assert!(MAX_ZOOM > MIN_ZOOM, "MAX_ZOOM must be greater than MIN_ZOOM");
const _: () = assert!(ZOOM_STEP > 0.0, "ZOOM_STEP must be positive");

/// Viewport - what part of the canvas is visible.
///
/// Maps between screen coordinates (u16, visible terminal area) and canvas 
/// coordinates (i32, infinite space). Tracks pan position (offset), dimensions 
/// (terminal size), and zoom level.
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Canvas X offset (pan position)
    pub offset_x: i32,
    /// Canvas Y offset (pan position)
    pub offset_y: i32,
    /// Screen width in columns
    pub width: u16,
    /// Screen height in rows
    pub height: u16,
    /// Zoom level: 1.0 = normal, 2.0 = zoomed in, 0.5 = zoomed out
    pub zoom: f32,
}

impl Viewport {
    /// Create a new viewport with the given dimensions.
    /// 
    /// The viewport starts at offset (0, 0) with zoom 1.0.
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            offset_x: 0,
            offset_y: 0,
            width,
            height,
            zoom: 1.0,
        }
    }

    /// Convert screen coordinates to canvas coordinates.
    ///
    /// At zoom > 1.0, screen coordinates map to smaller canvas areas.
    /// At zoom < 1.0, screen coordinates map to larger canvas areas.
    pub fn screen_to_canvas(&self, screen_x: u16, screen_y: u16) -> Position {
        debug_assert!(self.zoom > 0.0, "zoom must be positive");
        debug_assert!(self.width > 0, "width must be positive");
        debug_assert!(self.height > 0, "height must be positive");
        
        let canvas_x = (f32::from(screen_x) / self.zoom)
            .round()
            .clamp(i32::MIN as f32, i32::MAX as f32) as i32;
        let canvas_y = (f32::from(screen_y) / self.zoom)
            .round()
            .clamp(i32::MIN as f32, i32::MAX as f32) as i32;
        
        Position::new(
            canvas_x.saturating_add(self.offset_x),
            canvas_y.saturating_add(self.offset_y),
        )
    }

    /// Convert canvas coordinates to screen coordinates (if visible).
    ///
    /// Returns `None` if the canvas position is outside the visible viewport.
    /// At zoom > 1.0, canvas positions map to larger screen areas.
    pub fn canvas_to_screen(&self, pos: Position) -> Option<(u16, u16)> {
        debug_assert!(self.zoom > 0.0, "zoom must be positive");
        debug_assert!(self.width > 0, "width must be positive");
        debug_assert!(self.height > 0, "height must be positive");
        
        let canvas_x_offset = pos.x.saturating_sub(self.offset_x);
        let canvas_y_offset = pos.y.saturating_sub(self.offset_y);
        
        let screen_x = (canvas_x_offset as f32 * self.zoom)
            .round()
            .clamp(i32::MIN as f32, i32::MAX as f32) as i32;
        let screen_y = (canvas_y_offset as f32 * self.zoom)
            .round()
            .clamp(i32::MIN as f32, i32::MAX as f32) as i32;

        if screen_x < 0 {
            return None;
        }
        if screen_y < 0 {
            return None;
        }
        if screen_x >= i32::from(self.width) {
            return None;
        }
        if screen_y >= i32::from(self.height) {
            return None;
        }

        Some((screen_x as u16, screen_y as u16))
    }

    /// Pan the viewport by the given delta in canvas units.
    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.offset_x = self.offset_x.saturating_add(dx);
        self.offset_y = self.offset_y.saturating_add(dy);
    }

    /// Zoom in by one step, clamped to MAX_ZOOM.
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + ZOOM_STEP).min(MAX_ZOOM);
    }

    /// Zoom out by one step, clamped to MIN_ZOOM.
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - ZOOM_STEP).max(MIN_ZOOM);
    }

    /// Reset zoom to 1.0 (100%).
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
    }

    /// Resize the viewport when terminal dimensions change.
    /// 
    /// Preserves offset and zoom level.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    /// Get the visible canvas area (number of canvas cells visible).
    ///
    /// At zoom > 1.0, fewer canvas cells are visible.
    /// At zoom < 1.0, more canvas cells are visible.
    pub fn visible_canvas_size(&self) -> (u16, u16) {
        debug_assert!(self.zoom > 0.0, "zoom must be positive");
        debug_assert!(self.width > 0, "width must be positive");
        debug_assert!(self.height > 0, "height must be positive");
        
        let visible_width = (f32::from(self.width) / self.zoom)
            .round()
            .clamp(0.0, u16::MAX as f32) as u16;
        let visible_height = (f32::from(self.height) / self.zoom)
            .round()
            .clamp(0.0, u16::MAX as f32) as u16;
        
        (visible_width, visible_height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_construction() {
        let pos = Position::new(-5, 10);
        assert_eq!(pos.x, -5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn position_equality() {
        let pos1 = Position::new(5, 10);
        let pos2 = Position::new(5, 10);
        let pos3 = Position::new(6, 10);
        
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn viewport_construction() {
        let viewport = Viewport::new(80, 24);
        assert_eq!(viewport.offset_x, 0);
        assert_eq!(viewport.offset_y, 0);
        assert_eq!(viewport.width, 80);
        assert_eq!(viewport.height, 24);
        assert_eq!(viewport.zoom, 1.0);
    }

    #[test]
    fn screen_to_canvas_default_zoom() {
        let viewport = Viewport::new(80, 24);
        let pos = viewport.screen_to_canvas(10, 5);
        assert_eq!(pos, Position::new(10, 5));
    }

    #[test]
    fn screen_to_canvas_with_offset() {
        let mut viewport = Viewport::new(80, 24);
        viewport.offset_x = 100;
        viewport.offset_y = 50;
        
        let pos = viewport.screen_to_canvas(10, 5);
        assert_eq!(pos, Position::new(110, 55));
    }

    #[test]
    fn screen_to_canvas_with_zoom() {
        let mut viewport = Viewport::new(80, 24);
        viewport.zoom = 2.0;
        
        let pos = viewport.screen_to_canvas(10, 5);
        assert_eq!(pos, Position::new(5, 3)); // rounded from 5.0, 2.5
    }

    #[test]
    fn canvas_to_screen_visible() {
        let viewport = Viewport::new(80, 24);
        let result = viewport.canvas_to_screen(Position::new(5, 3));
        assert_eq!(result, Some((5, 3)));
    }

    #[test]
    fn canvas_to_screen_outside_viewport() {
        let viewport = Viewport::new(80, 24);
        let result = viewport.canvas_to_screen(Position::new(100, 3));
        assert_eq!(result, None);
        
        let result = viewport.canvas_to_screen(Position::new(5, 30));
        assert_eq!(result, None);
    }

    #[test]
    fn canvas_to_screen_negative_with_offset() {
        let mut viewport = Viewport::new(80, 24);
        viewport.offset_x = -20;
        viewport.offset_y = 0;
        
        let result = viewport.canvas_to_screen(Position::new(-10, 0));
        assert_eq!(result, Some((10, 0)));
    }

    #[test]
    fn pan_shifts_offset() {
        let mut viewport = Viewport::new(80, 24);
        viewport.offset_x = 10;
        viewport.offset_y = 20;
        
        viewport.pan(5, -3);
        assert_eq!(viewport.offset_x, 15);
        assert_eq!(viewport.offset_y, 17);
    }

    #[test]
    fn zoom_in() {
        let mut viewport = Viewport::new(80, 24);
        viewport.zoom = 1.0;
        
        viewport.zoom_in();
        assert_eq!(viewport.zoom, 1.25);
    }

    #[test]
    fn zoom_clamped_to_max() {
        let mut viewport = Viewport::new(80, 24);
        viewport.zoom = MAX_ZOOM;
        
        viewport.zoom_in();
        assert_eq!(viewport.zoom, MAX_ZOOM);
    }

    #[test]
    fn zoom_out() {
        let mut viewport = Viewport::new(80, 24);
        viewport.zoom = 1.25;
        
        viewport.zoom_out();
        assert_eq!(viewport.zoom, 1.0);
    }

    #[test]
    fn zoom_clamped_to_min() {
        let mut viewport = Viewport::new(80, 24);
        viewport.zoom = MIN_ZOOM;
        
        viewport.zoom_out();
        assert_eq!(viewport.zoom, MIN_ZOOM);
    }

    #[test]
    fn reset_zoom() {
        let mut viewport = Viewport::new(80, 24);
        viewport.zoom = 2.5;
        
        viewport.reset_zoom();
        assert_eq!(viewport.zoom, 1.0);
    }

    #[test]
    fn resize_updates_dimensions() {
        let mut viewport = Viewport::new(80, 24);
        viewport.offset_x = 10;
        viewport.offset_y = 20;
        viewport.zoom = 1.5;
        
        viewport.resize(120, 40);
        assert_eq!(viewport.width, 120);
        assert_eq!(viewport.height, 40);
        // Preserve offset and zoom
        assert_eq!(viewport.offset_x, 10);
        assert_eq!(viewport.offset_y, 20);
        assert_eq!(viewport.zoom, 1.5);
    }

    #[test]
    fn visible_canvas_size_at_zoom_1() {
        let viewport = Viewport::new(80, 24);
        let (width, height) = viewport.visible_canvas_size();
        assert_eq!(width, 80);
        assert_eq!(height, 24);
    }

    #[test]
    fn visible_canvas_size_at_zoom_2() {
        let mut viewport = Viewport::new(80, 24);
        viewport.zoom = 2.0;
        
        let (width, height) = viewport.visible_canvas_size();
        assert_eq!(width, 40);
        assert_eq!(height, 12);
    }

    #[test]
    fn visible_canvas_size_at_zoom_half() {
        let mut viewport = Viewport::new(80, 24);
        viewport.zoom = 0.5;
        
        let (width, height) = viewport.visible_canvas_size();
        assert_eq!(width, 160);
        assert_eq!(height, 48);
    }

    #[test]
    fn screen_to_canvas_with_offset_and_zoom() {
        let mut viewport = Viewport::new(80, 24);
        viewport.offset_x = 100;
        viewport.offset_y = 50;
        viewport.zoom = 2.0;
        
        let pos = viewport.screen_to_canvas(10, 8);
        // 10 / 2.0 = 5, 8 / 2.0 = 4, then add offset
        assert_eq!(pos, Position::new(105, 54));
    }

    #[test]
    fn canvas_to_screen_with_offset_and_zoom() {
        let mut viewport = Viewport::new(80, 24);
        viewport.offset_x = 10;
        viewport.offset_y = 5;
        viewport.zoom = 2.0;
        
        let result = viewport.canvas_to_screen(Position::new(15, 8));
        // (15-10)*2 = 10, (8-5)*2 = 6
        assert_eq!(result, Some((10, 6)));
    }

    #[test]
    fn pan_saturating() {
        let mut viewport = Viewport::new(80, 24);
        viewport.offset_x = i32::MAX - 5;
        
        viewport.pan(10, 0); // Would overflow
        assert_eq!(viewport.offset_x, i32::MAX);
    }
}