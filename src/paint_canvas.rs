use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tiny_skia::PixmapMut;

trait BufferExt {
    fn image(&self) -> Image;
    fn pixmap_mut(&mut self) -> PixmapMut;
    fn fill(&mut self, color: tiny_skia::Color) {
        let mut pixmap = self.pixmap_mut();
        pixmap.fill(color);
    }
}

impl BufferExt for SharedPixelBuffer<Rgba8Pixel> {
    fn image(&self) -> Image {
        Image::from_rgba8_premultiplied(self.clone())
    }

    fn pixmap_mut(&mut self) -> PixmapMut {
        let width = self.width();
        let height = self.height();
        let pixmap_opt: Option<PixmapMut> =
            PixmapMut::from_bytes(self.make_mut_bytes(), width, height);
        pixmap_opt.expect("Couldn't create pixmap image")
    }
}

#[derive(Debug, Clone, Default)]
struct DrawingState {
    /// Start of the mouse needed for Line, Rect, Ellipse
    start: Option<(f32, f32)>,

    /// Current path needed for Freehand
    path: Option<tiny_skia::PathBuilder>,

    /// Finalized path
    path_finalized: Option<tiny_skia::Path>,
}

impl DrawingState {
    fn clear(&mut self) {
        self.start = None;
        self.path = None;
        self.path_finalized = None;
    }
}

#[derive(Debug, Clone)]
pub struct DrawnPath {
    path: tiny_skia::Path,
    tool_properties: ToolProperties,
}

impl DrawnPath {
    pub fn new(path: tiny_skia::Path, tool_properties: ToolProperties) -> Self {
        Self {
            path,
            tool_properties,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Tool {
    #[default]
    Freehand,
    Line,
    Rect,
    Ellipse,
}

#[derive(Debug, Clone, Copy)]
pub struct ToolProperties {
    pub size: f32,
    pub color: tiny_skia::Color,
}

impl Default for ToolProperties {
    fn default() -> Self {
        Self {
            size: 5.0,
            color: tiny_skia::Color::from_rgba8(255, 0, 0, 255),
        }
    }
}

/// Paint canvas API.
///
/// # Example
///
/// Create it with the builder pattern:
///
/// ```
/// use paint_canvas::PaintCanvas;
///
/// let paint_canvas = Rc::new(RefCell::new(
///     PaintCanvas::builder(600, 800)
///         .tool_type(Tool::Freehand)
///         .tool_size(ui.get_brush_value() as f32)
///         .tool_color(tiny_skia::Color::from_rgba8(255, 0, 0, 255))
///         .build(),
/// ));
/// ```
#[derive(Debug, Clone)]
pub struct PaintCanvas {
    /// A collection of the displayed paths
    pub paths: Vec<DrawnPath>,

    /// The real buffer of the image
    buffer: SharedPixelBuffer<Rgba8Pixel>,

    /// The preview buffer of the image
    path_preview: SharedPixelBuffer<Rgba8Pixel>,

    /// Current drawing state
    drawing_state: DrawingState,

    /// Selected Tool
    pub selected_tool: Tool,

    /// Tool properties
    pub tool_properties: ToolProperties,

    /// Tool preview color
    pub tool_preview_color: tiny_skia::Color,
}

impl PaintCanvas {
    /// Builder pattern to create a `PaintCanvas`.
    pub fn builder(width: u32, height: u32) -> PaintCanvasBuilder {
        PaintCanvasBuilder::new(width, height)
    }

    pub fn set_start(&mut self, mouse_x: f32, mouse_y: f32) {
        self.drawing_state.start = Some((mouse_x, mouse_y));
    }

    pub fn set_tool(&mut self, tool: Tool) {
        self.selected_tool = tool;
    }

    pub fn set_tool_size(&mut self, size: f32) {
        self.tool_properties.size = size;
    }

    pub fn set_tool_color(&mut self, color: tiny_skia::Color) {
        self.tool_properties.color = color;
    }

    pub fn set_tool_preview_color(&mut self, color: tiny_skia::Color) {
        self.tool_preview_color = color;
    }

    pub fn drawn_path(&self) -> Option<DrawnPath> {
        self.drawing_state
            .path_finalized
            .clone()
            .map(|path| DrawnPath::new(path, self.tool_properties))
    }

    pub fn clear_state(&mut self) {
        self.drawing_state.clear();
    }

    pub fn add_path(&mut self, path: DrawnPath) {
        self.paths.push(path);
    }

    pub fn pop_path(&mut self) {
        self.paths.pop();
    }

    pub fn apply(&mut self) {
        self.path_preview.fill(tiny_skia::Color::TRANSPARENT);
        let mut pixmap = self.buffer.pixmap_mut();
        pixmap.fill(tiny_skia::Color::TRANSPARENT);
        for drawn_path in &self.paths {
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(255, 0, 0, 255);
            let stroke = tiny_skia::Stroke {
                width: drawn_path.tool_properties.size,
                ..Default::default()
            };
            pixmap.stroke_path(&drawn_path.path, &paint, &stroke, Default::default(), None);
        }
    }

    pub fn image(&self) -> Image {
        Image::from_rgba8_premultiplied(self.buffer.clone())
    }

    pub fn image_buffer(&self) -> Image {
        Image::from_rgba8_premultiplied(self.path_preview.clone())
    }

    pub fn draw_preview(&mut self, mouse_x: f32, mouse_y: f32) {
        match self.selected_tool {
            Tool::Freehand => self.draw_freehand_buffer(mouse_x, mouse_y),
            Tool::Line => self.draw_line_buffer(mouse_x, mouse_y),
            Tool::Rect => self.draw_rect_buffer(mouse_x, mouse_y),
            Tool::Ellipse => self.draw_circle_buffer(mouse_x, mouse_y),
        };
    }

    pub fn draw_freehand_buffer(&mut self, mouse_x: f32, mouse_y: f32) {
        let mut pixmap = self.path_preview.pixmap_mut();
        pixmap.fill(tiny_skia::Color::TRANSPARENT);

        match self.drawing_state.path {
            None => {
                let mut builder = tiny_skia::PathBuilder::new();
                builder.move_to(mouse_x, mouse_y);
                self.drawing_state.path = Some(builder);
            }
            Some(ref mut builder) => builder.line_to(mouse_x, mouse_y),
        }

        if let Some(path) = self.drawing_state.path.clone().unwrap().finish() {
            self.drawing_state.path_finalized = Some(path.clone());
            let mut paint = tiny_skia::Paint::default();
            paint.set_color_rgba8(212, 212, 216, 255);
            let stroke = tiny_skia::Stroke {
                width: self.tool_properties.size,
                ..Default::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, Default::default(), None);
        }
    }

    pub fn draw_line_buffer(&mut self, mouse_x: f32, mouse_y: f32) {
        let Some((start_x, start_y)) = self.drawing_state.start else {
            return;
        };
        let mut pixmap = self.path_preview.pixmap_mut();
        pixmap.fill(tiny_skia::Color::TRANSPARENT);

        let mut path = tiny_skia::PathBuilder::new();
        path.move_to(start_x, start_y);
        path.line_to(mouse_x, mouse_y);
        let path = path.finish().unwrap();
        self.drawing_state.path_finalized = Some(path.clone());
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(212, 212, 216, 255);
        let stroke = tiny_skia::Stroke {
            width: self.tool_properties.size,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Default::default(), None);
    }

    pub fn draw_rect_buffer(&mut self, mouse_x: f32, mouse_y: f32) {
        let Some((start_x, start_y)) = self.drawing_state.start else {
            return;
        };
        let mut pixmap = self.path_preview.pixmap_mut();
        pixmap.fill(tiny_skia::Color::TRANSPARENT);

        let left = start_x.min(mouse_x);
        let right = start_x.max(mouse_x);
        let top = start_y.min(mouse_y);
        let bottom = start_y.max(mouse_y);

        let rect = tiny_skia::Rect::from_ltrb(left, top, right, bottom).unwrap();
        let path = tiny_skia::PathBuilder::from_rect(rect);
        self.drawing_state.path_finalized = Some(path.clone());

        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(212, 212, 216, 255);
        let stroke = tiny_skia::Stroke {
            width: self.tool_properties.size,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Default::default(), None);
    }

    pub fn draw_circle_buffer(&mut self, mouse_x: f32, mouse_y: f32) {
        let Some((start_x, start_y)) = self.drawing_state.start else {
            return;
        };
        let mut pixmap = self.path_preview.pixmap_mut();
        pixmap.fill(tiny_skia::Color::TRANSPARENT);

        let left = start_x.min(mouse_x);
        let right = start_x.max(mouse_x);
        let top = start_y.min(mouse_y);
        let bottom = start_y.max(mouse_y);

        let rect = tiny_skia::Rect::from_ltrb(left, top, right, bottom).unwrap();
        let path = tiny_skia::PathBuilder::from_oval(rect).unwrap();
        self.drawing_state.path_finalized = Some(path.clone());

        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(212, 212, 216, 255);
        let stroke = tiny_skia::Stroke {
            width: self.tool_properties.size,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Default::default(), None);
    }
}

/// Builder pattern to create a `PaintCanvas`
pub struct PaintCanvasBuilder {
    canvas: PaintCanvas,
}

impl PaintCanvasBuilder {
    pub fn new(width: u32, height: u32) -> Self {
        let mut path_preview = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
        path_preview.fill(tiny_skia::Color::TRANSPARENT);

        let buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);

        Self {
            canvas: PaintCanvas {
                paths: Vec::new(),
                buffer,
                path_preview,
                drawing_state: DrawingState::default(),
                selected_tool: Tool::default(),
                tool_properties: ToolProperties::default(),
                tool_preview_color: tiny_skia::Color::from_rgba8(212, 212, 216, 255),
            },
        }
    }

    /// Sets the tool properties directly.
    pub fn tool_properties(mut self, tool_properties: ToolProperties) -> Self {
        self.canvas.tool_properties = tool_properties;
        self
    }

    /// Sets the color of the tool.
    pub fn tool_color(mut self, color: tiny_skia::Color) -> Self {
        self.canvas.tool_properties.color = color;
        self
    }

    /// Sets the size of the tool.
    pub fn tool_size(mut self, size: f32) -> Self {
        self.canvas.tool_properties.size = size;
        self
    }

    /// Sets the preview color of the tool.
    pub fn tool_preview_color(mut self, color: tiny_skia::Color) -> Self {
        self.canvas.tool_preview_color = color;
        self
    }

    /// Sets the type of the tool
    pub fn tool_type(mut self, tool: Tool) -> Self {
        self.canvas.selected_tool = tool;
        self
    }

    /// Finalizes the builder and returns the `PaintCanvas`.
    pub fn build(self) -> PaintCanvas {
        self.canvas
    }
}

