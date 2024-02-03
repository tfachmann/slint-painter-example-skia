slint::include_modules!();
use core::cell::{Ref, RefCell, RefMut};
use slint::platform::PointerEventButton;
use slint::private_unstable_api::re_exports::PointerEventKind;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::rc::Rc;
use tiny_skia::PixmapMut;

#[derive(Debug, Clone, Copy)]
enum Tool {
    Freehand,
    Line,
    Rect,
    Circle,
}

#[derive(Debug, Clone, Copy)]
struct ToolProperties{
    size: f32,
}

#[derive(Debug, Clone)]
struct Drawing {
    buffer: SharedPixelBuffer<Rgba8Pixel>,
    buffer_draw: SharedPixelBuffer<Rgba8Pixel>,
}

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

#[derive(Debug, Clone)]
struct DrawingState {
    start: Option<(f32, f32)>,
    path: Option<tiny_skia::PathBuilder>,
}

impl DrawingState {
    fn new() -> Self {
        Self {
            start: None,
            path: None,
        }
    }
}

impl Drawing {
    fn new(width: u32, height: u32) -> Self {
        let mut buffer_draw = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
        buffer_draw.fill(tiny_skia::Color::TRANSPARENT);

        let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
        buffer.fill(tiny_skia::Color::from_rgba8(31, 41, 55, 255));
        Self {
            buffer,
            buffer_draw,
        }
    }

    fn apply_buffer(&mut self) {
        let mut pixmap = self.buffer.pixmap_mut();
        let mut pixmap_buffer = self.buffer_draw.pixmap_mut();

        pixmap_buffer.pixels_mut().iter_mut().for_each(|pixel| {
            let alpha = pixel.alpha();
            if alpha > 0 {
                let r = alpha.min(255);
                *pixel = tiny_skia::PremultipliedColorU8::from_rgba(r, 0, 0, alpha).unwrap();
            }
        });

        let paint = tiny_skia::PixmapPaint::default();
        pixmap.draw_pixmap(
            0,
            0,
            pixmap_buffer.as_ref(),
            &paint,
            Default::default(),
            None,
        );

        pixmap_buffer.fill(tiny_skia::Color::TRANSPARENT);
    }

    fn image(&self) -> Image {
        Image::from_rgba8_premultiplied(self.buffer.clone())
    }

    fn image_buffer(&self) -> Image {
        Image::from_rgba8_premultiplied(self.buffer_draw.clone())
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let drawing = Rc::new(RefCell::new(Drawing::new(500, 500)));
    // let selected_tool: Rc<Mutex<Tool>> = Rc::new(Mutex::new(Tool::Freehand));
    let selected_tool = Rc::new(RefCell::new(Tool::Freehand));
    let tool_properties = Rc::new(RefCell::new(ToolProperties {
        size: ui.get_brush_value() as f32,
    }));
    let drawing_state = Rc::new(RefCell::new(DrawingState::new()));
    let window_clone = ui.clone_strong();
    render_drawing(&window_clone, drawing.borrow());

    ui.on_mouse_event({
        let selected_tool = Rc::clone(&selected_tool);
        let ui_handle = ui.as_weak();
        move |pointer_event, mouse_x, mouse_y, pressed| {
            if pointer_event.button == PointerEventButton::Left {
                match pointer_event.kind {
                    PointerEventKind::Down => match *selected_tool.borrow() {
                        Tool::Line | Tool::Rect | Tool::Circle => {
                            drawing_state.borrow_mut().start = Some((mouse_x, mouse_y));
                        }
                        _ => (),
                    },
                    PointerEventKind::Up => {
                        drawing.borrow_mut().apply_buffer();
                        drawing_state.borrow_mut().path = None;
                        render_drawing(&window_clone, drawing.borrow());
                        render_drawing_buffer(&window_clone, drawing.borrow());
                    }
                    _ => (),
                }
            }
            if pressed {
                // TODO: Only update on spin box update
                let mut tool_properties = tool_properties.borrow_mut();
                tool_properties.size = ui_handle.unwrap().get_brush_value() as f32;
                match *selected_tool.borrow() {
                    Tool::Freehand => draw_freehand_buffer(
                        &mut drawing.borrow_mut(),
                        &mut drawing_state.borrow_mut().path,
                        &tool_properties,
                        mouse_x,
                        mouse_y,
                    ),
                    Tool::Line => draw_line_buffer(
                        &mut drawing.borrow_mut(),
                        &drawing_state.borrow_mut().start,
                        &tool_properties,
                        mouse_x,
                        mouse_y,
                    ),
                    Tool::Rect => draw_rect_buffer(
                        &mut drawing.borrow_mut(),
                        &drawing_state.borrow_mut().start,
                        &tool_properties,
                        mouse_x,
                        mouse_y,
                    ),
                    Tool::Circle => draw_circle_buffer(
                        &mut drawing.borrow_mut(),
                        &drawing_state.borrow_mut().start,
                        &tool_properties,
                        mouse_x,
                        mouse_y,
                    ),
                };
                render_drawing_buffer(&window_clone, drawing.borrow());
            }
        }
    });

    ui.on_select_freehand({
        let selected_tool = Rc::clone(&selected_tool);
        move || *selected_tool.borrow_mut() = Tool::Freehand
    });

    ui.on_select_line({
        let selected_tool = Rc::clone(&selected_tool);
        move || *selected_tool.borrow_mut() = Tool::Line
    });

    ui.on_select_rect({
        let selected_tool = Rc::clone(&selected_tool);
        move || *selected_tool.borrow_mut() = Tool::Rect
    });

    ui.on_select_circle({
        let selected_tool = Rc::clone(&selected_tool);
        move || *selected_tool.borrow_mut() = Tool::Circle
    });

    ui.run()
}

fn draw_freehand_buffer(
    drawing: &mut RefMut<Drawing>,
    path_builder: &mut Option<tiny_skia::PathBuilder>,
    tool_properties: &ToolProperties,
    mouse_x: f32,
    mouse_y: f32,
) {
    let mut pixmap = drawing.buffer_draw.pixmap_mut();
    pixmap.fill(tiny_skia::Color::TRANSPARENT);

    match path_builder {
        None => {
            let mut builder = tiny_skia::PathBuilder::new();
            builder.move_to(mouse_x, mouse_y);
            *path_builder = Some(builder);
        }
        Some(ref mut builder) => builder.line_to(mouse_x, mouse_y),
    }

    if let Some(path) = path_builder.clone().unwrap().finish() {
        let mut paint = tiny_skia::Paint::default();
        paint.set_color_rgba8(212, 212, 216, 255);
        let stroke = tiny_skia::Stroke{
            width: tool_properties.size,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Default::default(), None);
    }
}

fn draw_line_buffer(
    drawing: &mut RefMut<Drawing>,
    start: &Option<(f32, f32)>,
    tool_properties: &ToolProperties,
    mouse_x: f32,
    mouse_y: f32,
) {
    let Some((start_x, start_y)) = start else {
        return;
    };
    let mut pixmap = drawing.buffer_draw.pixmap_mut();
    pixmap.fill(tiny_skia::Color::TRANSPARENT);

    let mut path = tiny_skia::PathBuilder::new();
    path.move_to(*start_x, *start_y);
    path.line_to(mouse_x, mouse_y);
    let path = path.finish().unwrap();
    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(212, 212, 216, 255);
    let stroke = tiny_skia::Stroke{
        width: tool_properties.size,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Default::default(), None);
}

fn draw_rect_buffer(
    drawing: &mut RefMut<Drawing>,
    start: &Option<(f32, f32)>,
    tool_properties: &ToolProperties,
    mouse_x: f32,
    mouse_y: f32,
) {
    let Some((start_x, start_y)) = start else {
        return;
    };
    let mut pixmap = drawing.buffer_draw.pixmap_mut();
    pixmap.fill(tiny_skia::Color::TRANSPARENT);

    let left = start_x.min(mouse_x);
    let right = start_x.max(mouse_x);
    let top = start_y.min(mouse_y);
    let bottom = start_y.max(mouse_y);

    let rect = tiny_skia::Rect::from_ltrb(left, top, right, bottom).unwrap();
    let path = tiny_skia::PathBuilder::from_rect(rect);

    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(212, 212, 216, 255);
    let stroke = tiny_skia::Stroke{
        width: tool_properties.size,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Default::default(), None);
}

fn draw_circle_buffer(
    drawing: &mut RefMut<Drawing>,
    start: &Option<(f32, f32)>,
    tool_properties: &ToolProperties,
    mouse_x: f32,
    mouse_y: f32,
) {
    let Some((start_x, start_y)) = start else {
        return;
    };
    let mut pixmap = drawing.buffer_draw.pixmap_mut();
    pixmap.fill(tiny_skia::Color::TRANSPARENT);

    let left = start_x.min(mouse_x);
    let right = start_x.max(mouse_x);
    let top = start_y.min(mouse_y);
    let bottom = start_y.max(mouse_y);

    let rect = tiny_skia::Rect::from_ltrb(left, top, right, bottom).unwrap();
    let path = tiny_skia::PathBuilder::from_oval(rect).unwrap();

    let mut paint = tiny_skia::Paint::default();
    paint.set_color_rgba8(212, 212, 216, 255);
    let stroke = tiny_skia::Stroke{
        width: tool_properties.size,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Default::default(), None);
}

fn render_drawing_buffer(main_window: &AppWindow, drawing: Ref<Drawing>) {
    main_window.set_canvas_buffer_source(drawing.image_buffer())
}

fn render_drawing(main_window: &AppWindow, drawing: Ref<Drawing>) {
    main_window.set_canvas_source(drawing.image())
}
