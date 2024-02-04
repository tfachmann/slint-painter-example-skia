pub trait Command {
    fn execute(&mut self);
    fn unexecute(&mut self);
}

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

    pub fn undo(&mut self) {
        if let Some(mut command) = self.undo_stack.pop() {
            command.unexecute();
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self) {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute();
            self.undo_stack.push(command);
        }
    }

    pub fn push(&mut self, mut command: Box<dyn Command>) {
        command.execute();
        self.undo_stack.push(command);
    }
}
