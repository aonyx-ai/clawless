use crate::context::{Command, IntoCommand};
use crate::{CommandArguments, CommandResult};
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

    pub async fn execute<Arguments: CommandArguments, Context, S: Command<Arguments> + 'static>(
        &mut self,
        args: Arguments,
        command: impl IntoCommand<Arguments, Context, Command = S>,
    ) -> CommandResult {
        let mut injectable_command = command.into_command();
        injectable_command.run(args, &mut self.contexts).await
    }

    pub fn add_context<C: 'static>(&mut self, context: C) {
        self.contexts.insert(TypeId::of::<C>(), Box::new(context));
    }
}
