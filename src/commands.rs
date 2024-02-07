//! This module includes commands needed for undo and redo operations for the painter.

use crate::paint_canvas::{DrawnPath, PaintCanvas};
use crate::undo_stack::Command;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;

/// Command to add an path to the `PaintCanvas`.
pub struct AddPath {
    paint_canvas: Rc<RefCell<PaintCanvas>>,
    path: DrawnPath,
}

impl AddPath {
    pub fn new(paint_canvas: Rc<RefCell<PaintCanvas>>, path: DrawnPath) -> Self {
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

/// Command to clear all paths on a `PaintCanvas`.
pub struct ClearPaths {
    paint_canvas: Rc<RefCell<PaintCanvas>>,
    paths: Vec<DrawnPath>,
}

impl ClearPaths {
    pub fn new(paint_canvas: Rc<RefCell<PaintCanvas>>) -> Self {
        Self {
            paint_canvas,
            paths: Default::default(),
        }
    }
}

impl Command for ClearPaths {
    fn execute(&mut self) {
        let mut paint_canvas = self.paint_canvas.borrow_mut();
        self.paths = mem::take(&mut paint_canvas.paths);
        paint_canvas.apply();
    }

    fn unexecute(&mut self) {
        let mut paint_canvas = self.paint_canvas.borrow_mut();
        paint_canvas.paths = mem::take(&mut self.paths);
        paint_canvas.apply();
    }
}
