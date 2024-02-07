//! This module contains a simple implementation and interface for an undo stack.

/// Interface for commands used in an `UndoStack`.
pub trait Command {
    /// Function to
    fn execute(&mut self);
    fn unexecute(&mut self);
}

/// Simple implementation of an undo stack.
pub struct UndoStack {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            undo_stack: Default::default(),
            redo_stack: Default::default(),
        }
    }

    /// Undo last operation and put command on redo stack.
    pub fn undo(&mut self) {
        if let Some(mut command) = self.undo_stack.pop() {
            command.unexecute();
            self.redo_stack.push(command);
        }
    }

    /// Redo last operation (execute it) and put command on undo stack.
    pub fn redo(&mut self) {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute();
            self.undo_stack.push(command);
        }
    }

    /// Adds a new command and executes it.
    pub fn push(&mut self, mut command: Box<dyn Command>) {
        command.execute();
        self.undo_stack.push(command);
    }
}

