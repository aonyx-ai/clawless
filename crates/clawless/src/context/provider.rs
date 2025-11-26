use crate::CommandResult;
use crate::context::{Command, IntoCommand};
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ContextProvider {
    contexts: HashMap<TypeId, Box<dyn Any>>,
}

impl ContextProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute<I, S: Command + 'static>(
        &mut self,
        command: impl IntoCommand<I, Command = S>,
    ) -> CommandResult {
        let mut injectable_command = command.into_command();
        injectable_command.run(&mut self.contexts)
    }

    pub fn add_context<C: 'static>(&mut self, context: C) {
        self.contexts.insert(TypeId::of::<C>(), Box::new(context));
    }
}
