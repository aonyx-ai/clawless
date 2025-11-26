use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::context::ContextParameter;
use crate::{CommandArguments, CommandResult};
use getset::MutGetters;
use typed_builder::TypedBuilder;

pub trait Command<Arguments: CommandArguments> {
    fn run(
        &mut self,
        args: Arguments,
        contexts: &mut HashMap<TypeId, Box<dyn Any>>,
    ) -> CommandResult;
}

pub trait IntoCommand<Arguments: CommandArguments, Context> {
    type Command: Command<Arguments>;

    fn into_command(self) -> Self::Command;
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, MutGetters, TypedBuilder,
)]
pub struct InjectableCommand<Function, Arguments, Context> {
    #[getset(get_mut = "pub")]
    function: Function,

    arguments: PhantomData<Arguments>,
    context: PhantomData<Context>,
}

macro_rules! impl_command {
  (
      $($params:ident),*
  ) => {
    #[allow(non_snake_case)]
    #[allow(unused)]
    impl<Function, Arguments, $($params: ContextParameter),*> Command<Arguments> for InjectableCommand<Function, Arguments, ($($params,)*)>
      where
        Arguments: CommandArguments,
        for<'a, 'b> &'a mut Function:
          FnMut(Arguments, $($params),* ) -> CommandResult +
          FnMut(Arguments, $(<$params as ContextParameter>::Item<'b>),* ) -> CommandResult
    {
      fn run(&mut self, args: Arguments, contexts: &mut HashMap<TypeId, Box<dyn Any>>) -> CommandResult {
        // This call_inner is necessary to tell rust which function impl to call
        fn call_inner<Arguments, $($params),*>(
          mut f: impl FnMut(Arguments, $($params),*) -> CommandResult,
          args: Arguments,
          $($params: $params),*
        ) -> CommandResult {
          f(args, $($params),*)
        }

        $(
          let $params = $params::retrieve(contexts);
        )*

        call_inner(self.function_mut(), args, $($params),*)
      }
    }
  }
}

impl_command!();
impl_command!(T1);
impl_command!(T1, T2);

macro_rules! impl_into_command {
  (
    $($params:ident),*
  ) => {
    impl<Function, Arguments, $($params: ContextParameter),*> IntoCommand<Arguments, ($($params,)*)> for Function
      where
        Arguments: CommandArguments,
        for<'a, 'b> &'a mut Function:
          FnMut(Arguments, $($params),* ) -> CommandResult +
          FnMut(Arguments, $(<$params as ContextParameter>::Item<'b>),* ) -> CommandResult
    {
      type Command = InjectableCommand<Self, Arguments, ($($params,)*)>;

      fn into_command(self) -> Self::Command {
        InjectableCommand::builder()
            .function(self)
            .arguments(Default::default())
            .context(Default::default())
            .build()
      }
    }
  }
}

impl_into_command!();
impl_into_command!(T1);
impl_into_command!(T1, T2);
