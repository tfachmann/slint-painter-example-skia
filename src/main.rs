slint::include_modules!();

mod undo_stack;
use crate::undo_stack::{Command, UndoStack};

mod paint_canvas;
use crate::paint_canvas::{DrawnPath, PaintCanvas, Tool, ToolProperties};

use core::cell::{Ref, RefCell};
use slint::platform::PointerEventButton;
use slint::private_unstable_api::re_exports::PointerEventKind;
use std::rc::Rc;
use std::mem;

// Command-specific
struct AddPath {
    paint_canvas: Rc<RefCell<PaintCanvas>>,
    path: DrawnPath,
}

impl AddPath {
    fn new(paint_canvas: Rc<RefCell<PaintCanvas>>, path: DrawnPath) -> Self {
        Self { paint_canvas, path }
    }
}

impl Command for AddPath {
    fn execute(&mut self) {
        let mut paint_canvas = self.paint_canvas.borrow_mut();
        paint_canvas.add_path(self.path.clone());
        paint_canvas.apply();
    }

    fn unexecute(&mut self) {
        let mut paint_canvas = self.paint_canvas.borrow_mut();
        paint_canvas.pop_path();
        paint_canvas.apply();
    }
}

struct ClearPaths {
    paint_canvas: Rc<RefCell<PaintCanvas>>,
    paths: Vec<DrawnPath>,
}

impl ClearPaths {
    fn new(paint_canvas: Rc<RefCell<PaintCanvas>>) -> Self {
        Self { paint_canvas, paths: Default::default() }
    }
}

impl Command for ClearPaths {
    fn execute(&mut self) {
        let mut paint_canvas = self.paint_canvas.borrow_mut();
        self.paths = mem::replace(&mut paint_canvas.paths, vec![]);
        paint_canvas.apply();
    }

    fn unexecute(&mut self) {
        let mut paint_canvas = self.paint_canvas.borrow_mut();
        paint_canvas.paths = mem::replace(&mut self.paths, vec![]);
        paint_canvas.apply();
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let tool_properties = ToolProperties {
        size: ui.get_brush_value() as f32,
    };
    let paint_canvas = Rc::new(RefCell::new(PaintCanvas::new(
        500,
        500,
        Tool::Freehand,
        tool_properties,
    )));
    let undo_stack = Rc::new(RefCell::new(UndoStack::new()));
    let window_clone = ui.clone_strong();
    render_drawing(&window_clone, paint_canvas.borrow());

    ui.on_mouse_event({
        let paint_canvas = Rc::clone(&paint_canvas);
        let undo_stack = Rc::clone(&undo_stack);
        let window_clone = ui.clone_strong();
        let ui_handle = ui.as_weak();
        move |pointer_event, mouse_x, mouse_y, pressed| {
            if pointer_event.button == PointerEventButton::Left {
                let tool = { paint_canvas.borrow().selected_tool.clone() };
                match pointer_event.kind {
                    PointerEventKind::Down => match tool {
                        Tool::Line | Tool::Rect | Tool::Circle => {
                            paint_canvas.borrow_mut().set_start(mouse_x, mouse_y);
                        }
                        _ => (),
                    },
                    PointerEventKind::Up => {
                        let drawn_path = { paint_canvas.borrow().drawn_path() };
                        if let Some(path) = drawn_path {
                            let command = AddPath::new(Rc::clone(&paint_canvas), path);
                            undo_stack.borrow_mut().push(Box::new(command));
                        }
                        paint_canvas.borrow_mut().clear_state();
                        render_drawing(&window_clone, paint_canvas.borrow());
                        render_drawing_buffer(&window_clone, paint_canvas.borrow());
                    }
                    _ => (),
                }
            }
            if pressed {
                // TODO: Only update on spin box update
                // tool_properties.size = ui_handle.unwrap().get_brush_value() as f32;
                paint_canvas.borrow_mut().draw_preview(mouse_x, mouse_y);
                render_drawing_buffer(&window_clone, paint_canvas.borrow());
            }
        }
    });

    ui.on_select_freehand({
        let paint_canvas = Rc::clone(&paint_canvas);
        move || paint_canvas.borrow_mut().set_tool(Tool::Freehand)
    });

    ui.on_select_line({
        let paint_canvas = Rc::clone(&paint_canvas);
        move || paint_canvas.borrow_mut().set_tool(Tool::Line)
    });

    ui.on_select_rect({
        let paint_canvas = Rc::clone(&paint_canvas);
        move || paint_canvas.borrow_mut().set_tool(Tool::Rect)
    });

    ui.on_select_circle({
        let paint_canvas = Rc::clone(&paint_canvas);
        move || paint_canvas.borrow_mut().set_tool(Tool::Circle)
    });

    ui.on_spin_box_edited({
        let paint_canvas = Rc::clone(&paint_canvas);
        move |value| paint_canvas.borrow_mut().set_tool_size(value as f32)
    });

    ui.on_clear({
        let paint_canvas = Rc::clone(&paint_canvas);
        let undo_stack = Rc::clone(&undo_stack);
        let window_clone = ui.clone_strong();
        move || {
            let command = ClearPaths::new(Rc::clone(&paint_canvas));
            undo_stack.borrow_mut().push(Box::new(command));
            render_drawing(&window_clone, paint_canvas.borrow());
        }
    });

    ui.on_undo({
        let paint_canvas = Rc::clone(&paint_canvas);
        let undo_stack = Rc::clone(&undo_stack);
        let window_clone = ui.clone_strong();
        move || {
            undo_stack.borrow_mut().undo();
            render_drawing(&window_clone, paint_canvas.borrow());
        }
    });

    ui.on_redo({
        let paint_canvas = Rc::clone(&paint_canvas);
        let undo_stack = Rc::clone(&undo_stack);
        let window_clone = ui.clone_strong();
        move || {
            undo_stack.borrow_mut().redo();
            render_drawing(&window_clone, paint_canvas.borrow());
        }
    });

    ui.run()
}

fn render_drawing_buffer(main_window: &AppWindow, paint_canvas: Ref<PaintCanvas>) {
    main_window.set_canvas_buffer_source(paint_canvas.image_buffer())
}

fn render_drawing(main_window: &AppWindow, paint_canvas: Ref<PaintCanvas>) {
    main_window.set_canvas_source(paint_canvas.image())
}
