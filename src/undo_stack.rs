pub trait Command {
    fn execute(&mut self);
    fn unexecute(&mut self);
}

pub struct UndoStack<Cmd> {
    undo_stack: Vec<Cmd>,
    redo_stack: Vec<Cmd>,
}

impl<Cmd: Command> UndoStack<Cmd> {
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

    pub fn push(&mut self, mut command: Cmd) {
        command.execute();
        self.undo_stack.push(command);
    }
}
