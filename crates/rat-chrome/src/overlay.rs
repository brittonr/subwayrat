use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear};

/// Anchoring positions for overlay placement within a viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAnchor {
    /// Center both horizontally and vertically.
    Center,
    /// Anchor to the top edge and center horizontally.
    Top,
    /// Anchor to the top-right corner.
    TopRight,
    /// Anchor to the right edge and center vertically.
    Right,
    /// Anchor to the bottom-right corner.
    BottomRight,
    /// Anchor to the bottom edge and center horizontally.
    Bottom,
    /// Anchor to the bottom-left corner.
    BottomLeft,
    /// Anchor to the left edge and center vertically.
    Left,
    /// Anchor to the top-left corner.
    TopLeft,
}

/// Width or height sizing relative to a viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlaySize {
    /// Fixed cell count.
    Fixed(u16),
    /// Percentage of the viewport, clamped to `0..=100`.
    Percent(u16),
}

impl OverlaySize {
    fn resolve(self, available: u16) -> u16 {
        match self {
            Self::Fixed(size) => size.min(available),
            Self::Percent(percent) => {
                let percent = percent.min(100) as u32;
                ((available as u32 * percent) / 100) as u16
            }
        }
    }
}

/// Structural configuration for an overlay frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayModel {
    /// Overlay anchor within the viewport.
    pub anchor: OverlayAnchor,
    /// Requested overlay width.
    pub width: OverlaySize,
    /// Requested overlay height.
    pub height: OverlaySize,
    /// Horizontal offset applied after anchor positioning.
    pub offset_x: i16,
    /// Vertical offset applied after anchor positioning.
    pub offset_y: i16,
    /// Clear the overlay region before applying fill and chrome.
    pub clear: bool,
    /// Style the viewport outside the overlay rect with the backdrop style.
    pub dim_backdrop: bool,
    /// Draw border/title chrome.
    pub border: bool,
    /// Optional title rendered with border chrome.
    pub title: Option<String>,
}

impl Default for OverlayModel {
    fn default() -> Self {
        Self {
            anchor: OverlayAnchor::Center,
            width: OverlaySize::Percent(50),
            height: OverlaySize::Percent(50),
            offset_x: 0,
            offset_y: 0,
            clear: false,
            dim_backdrop: false,
            border: true,
            title: None,
        }
    }
}

impl OverlayModel {
    /// Set the overlay anchor.
    pub fn with_anchor(mut self, anchor: OverlayAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set the overlay width.
    pub fn with_width(mut self, width: OverlaySize) -> Self {
        self.width = width;
        self
    }

    /// Set the overlay height.
    pub fn with_height(mut self, height: OverlaySize) -> Self {
        self.height = height;
        self
    }

    /// Set both x/y offsets.
    pub fn with_offsets(mut self, x: i16, y: i16) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    /// Set the horizontal offset.
    pub fn with_offset_x(mut self, x: i16) -> Self {
        self.offset_x = x;
        self
    }

    /// Set the vertical offset.
    pub fn with_offset_y(mut self, y: i16) -> Self {
        self.offset_y = y;
        self
    }

    /// Enable or disable clearing the overlay region before drawing.
    pub fn with_clear(mut self, clear: bool) -> Self {
        self.clear = clear;
        self
    }

    /// Enable or disable backdrop styling outside the overlay rect.
    pub fn with_backdrop(mut self, dim_backdrop: bool) -> Self {
        self.dim_backdrop = dim_backdrop;
        self
    }

    /// Enable or disable border chrome.
    pub fn with_border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    /// Set the overlay title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Visual styling for an overlay frame.
#[derive(Debug, Clone)]
pub struct OverlayStyle {
    /// Style applied to the border chrome.
    pub border: Style,
    /// Style applied to the title text.
    pub title: Style,
    /// Style applied to viewport cells outside the overlay rect when enabled.
    pub backdrop: Style,
    /// Style applied to the returned inner content rect.
    pub fill: Style,
}

impl Default for OverlayStyle {
    fn default() -> Self {
        Self {
            border: Style::default().fg(Color::Gray),
            title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            backdrop: Style::default().bg(Color::Black),
            fill: Style::default().bg(Color::Black),
        }
    }
}

impl OverlayStyle {
    /// Set the border style.
    pub fn with_border(mut self, border: Style) -> Self {
        self.border = border;
        self
    }

    /// Set the title style.
    pub fn with_title(mut self, title: Style) -> Self {
        self.title = title;
        self
    }

    /// Set the backdrop style.
    pub fn with_backdrop(mut self, backdrop: Style) -> Self {
        self.backdrop = backdrop;
        self
    }

    /// Set the fill style.
    pub fn with_fill(mut self, fill: Style) -> Self {
        self.fill = fill;
        self
    }
}

/// Computed layout returned by the overlay primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OverlayLayout {
    /// Full overlay rect including border/title chrome.
    pub outer: Rect,
    /// Child content rect after chrome is applied.
    pub inner: Rect,
}

/// Compute the overlay layout for a viewport without rendering it.
pub fn compute_overlay_layout(viewport: Rect, model: &OverlayModel) -> OverlayLayout {
    let outer = compute_outer_rect(viewport, model);
    let inner = if model.border {
        Block::default().borders(Borders::ALL).inner(outer)
    } else {
        outer
    };

    OverlayLayout { outer, inner }
}

/// Render an overlay frame and return its computed layout.
pub fn overlay_frame(
    frame: &mut Frame,
    viewport: Rect,
    model: &OverlayModel,
    style: &OverlayStyle,
) -> OverlayLayout {
    let layout = compute_overlay_layout(viewport, model);

    if viewport.width == 0 || viewport.height == 0 {
        return layout;
    }

    if model.dim_backdrop {
        paint_backdrop(frame.buffer_mut(), viewport, layout.outer, style.backdrop);
    }

    if model.clear && layout.outer.width > 0 && layout.outer.height > 0 {
        frame.render_widget(Clear, layout.outer);
    }

    if layout.inner.width > 0 && layout.inner.height > 0 {
        paint_rect(frame.buffer_mut(), layout.inner, style.fill);
    }

    if model.border {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(style.border);

        if let Some(title) = &model.title {
            block = block.title(Line::from(Span::styled(title.clone(), style.title)));
        }

        frame.render_widget(block, layout.outer);
    }

    layout
}

fn compute_outer_rect(viewport: Rect, model: &OverlayModel) -> Rect {
    if viewport.width == 0 || viewport.height == 0 {
        return Rect::new(viewport.x, viewport.y, 0, 0);
    }

    let width = model.width.resolve(viewport.width);
    let height = model.height.resolve(viewport.height);

    let base_x = match model.anchor {
        OverlayAnchor::Center | OverlayAnchor::Top | OverlayAnchor::Bottom => {
            viewport.x as i32 + ((viewport.width.saturating_sub(width)) / 2) as i32
        }
        OverlayAnchor::TopRight | OverlayAnchor::Right | OverlayAnchor::BottomRight => {
            rect_right(viewport) as i32 - width as i32
        }
        OverlayAnchor::BottomLeft | OverlayAnchor::Left | OverlayAnchor::TopLeft => {
            viewport.x as i32
        }
    };

    let base_y = match model.anchor {
        OverlayAnchor::Center | OverlayAnchor::Left | OverlayAnchor::Right => {
            viewport.y as i32 + ((viewport.height.saturating_sub(height)) / 2) as i32
        }
        OverlayAnchor::Bottom | OverlayAnchor::BottomLeft | OverlayAnchor::BottomRight => {
            rect_bottom(viewport) as i32 - height as i32
        }
        OverlayAnchor::Top | OverlayAnchor::TopLeft | OverlayAnchor::TopRight => viewport.y as i32,
    };

    let min_x = viewport.x as i32;
    let max_x = rect_right(viewport) as i32 - width as i32;
    let min_y = viewport.y as i32;
    let max_y = rect_bottom(viewport) as i32 - height as i32;

    let x = (base_x + model.offset_x as i32).clamp(min_x, max_x.max(min_x));
    let y = (base_y + model.offset_y as i32).clamp(min_y, max_y.max(min_y));

    Rect::new(x as u16, y as u16, width, height)
}

fn paint_backdrop(buf: &mut ratatui::buffer::Buffer, viewport: Rect, overlay: Rect, style: Style) {
    for y in viewport.y..rect_bottom(viewport) {
        for x in viewport.x..rect_right(viewport) {
            if !contains(overlay, x, y) {
                buf[(x, y)].set_style(style);
            }
        }
    }
}

fn paint_rect(buf: &mut ratatui::buffer::Buffer, area: Rect, style: Style) {
    for y in area.y..rect_bottom(area) {
        for x in area.x..rect_right(area) {
            buf[(x, y)].set_style(style);
        }
    }
}

fn contains(rect: Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect_right(rect) && y >= rect.y && y < rect_bottom(rect)
}

fn rect_right(rect: Rect) -> u16 {
    rect.x.saturating_add(rect.width)
}

fn rect_bottom(rect: Rect) -> u16 {
    rect.y.saturating_add(rect.height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::buffer::Buffer;
    use ratatui::text::Line;
    use ratatui::widgets::Paragraph;

    #[test]
    fn centers_fixed_size_overlay() {
        let viewport = Rect::new(0, 0, 80, 24);
        let model = OverlayModel::default()
            .with_width(OverlaySize::Fixed(40))
            .with_height(OverlaySize::Fixed(10));

        let layout = compute_overlay_layout(viewport, &model);

        assert_eq!(layout.outer, Rect::new(20, 7, 40, 10));
    }

    #[test]
    fn anchors_percentage_overlay_to_right_edge() {
        let viewport = Rect::new(0, 0, 100, 40);
        let model = OverlayModel::default()
            .with_anchor(OverlayAnchor::Right)
            .with_width(OverlaySize::Percent(30))
            .with_height(OverlaySize::Percent(100));

        let layout = compute_overlay_layout(viewport, &model);

        assert_eq!(layout.outer, Rect::new(70, 0, 30, 40));
    }

    #[test]
    fn anchors_to_bottom_edge() {
        let viewport = Rect::new(4, 2, 80, 24);
        let model = OverlayModel::default()
            .with_anchor(OverlayAnchor::Bottom)
            .with_width(OverlaySize::Fixed(20))
            .with_height(OverlaySize::Fixed(5));

        let layout = compute_overlay_layout(viewport, &model);

        assert_eq!(layout.outer, Rect::new(34, 21, 20, 5));
    }

    #[test]
    fn clamps_oversized_requests_to_viewport() {
        let viewport = Rect::new(3, 5, 80, 24);
        let model = OverlayModel::default()
            .with_width(OverlaySize::Fixed(120))
            .with_height(OverlaySize::Fixed(40))
            .with_offsets(50, 50);

        let layout = compute_overlay_layout(viewport, &model);

        assert_eq!(layout.outer, viewport);
    }

    #[test]
    fn border_chrome_reduces_inner_rect() {
        let viewport = Rect::new(0, 0, 80, 24);
        let model = OverlayModel::default()
            .with_width(OverlaySize::Fixed(40))
            .with_height(OverlaySize::Fixed(10))
            .with_border(true);

        let layout = compute_overlay_layout(viewport, &model);

        assert_eq!(layout.inner, Rect::new(21, 8, 38, 8));
    }

    #[test]
    fn borderless_overlay_uses_full_area_as_inner_rect() {
        let viewport = Rect::new(0, 0, 80, 24);
        let model = OverlayModel::default()
            .with_width(OverlaySize::Fixed(40))
            .with_height(OverlaySize::Fixed(10))
            .with_border(false);

        let layout = compute_overlay_layout(viewport, &model);

        assert_eq!(layout.inner, layout.outer);
    }

    #[test]
    fn clear_resets_overlay_cells_before_chrome() {
        let viewport = Rect::new(0, 0, 20, 6);
        let model = OverlayModel::default()
            .with_width(OverlaySize::Fixed(8))
            .with_height(OverlaySize::Fixed(3))
            .with_border(false)
            .with_clear(true);

        let buffer = render_overlay(viewport, &model, &OverlayStyle::default());
        let layout = compute_overlay_layout(viewport, &model);

        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].symbol(), " ");
        assert_eq!(buffer[(0, 0)].symbol(), "x");
    }

    #[test]
    fn backdrop_styles_only_cells_outside_overlay() {
        let viewport = Rect::new(0, 0, 20, 6);
        let model = OverlayModel::default()
            .with_width(OverlaySize::Fixed(8))
            .with_height(OverlaySize::Fixed(3))
            .with_border(false)
            .with_backdrop(true);
        let style = OverlayStyle::default().with_backdrop(Style::default().bg(Color::Blue));

        let buffer = render_overlay(viewport, &model, &style);
        let layout = compute_overlay_layout(viewport, &model);

        assert_eq!(buffer[(0, 0)].bg, Color::Blue);
        assert_eq!(buffer[(layout.inner.x, layout.inner.y)].bg, Color::Black);
        assert_eq!(buffer[(0, 0)].symbol(), "x");
    }

    fn render_overlay(viewport: Rect, model: &OverlayModel, style: &OverlayStyle) -> Buffer {
        let backend = TestBackend::new(viewport.width.max(1), viewport.height.max(1));
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                let background =
                    vec![Line::from("x".repeat(area.width as usize)); area.height as usize];
                frame.render_widget(Paragraph::new(background), area);
                overlay_frame(frame, viewport, model, style);
            })
            .unwrap();
        terminal.backend().buffer().clone()
    }
}
