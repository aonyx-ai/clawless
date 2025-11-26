use crate::CommandResult;
use crate::context::ContextParameter;
use clap::ArgMatches;
use getset::MutGetters;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use typed_builder::TypedBuilder;

pub trait Command {
    async fn run(
        &mut self,
        args: ArgMatches,
        contexts: &mut HashMap<TypeId, Box<dyn Any>>,
    ) -> CommandResult;
}

pub trait IntoCommand<Context> {
    type Command: Command;

    fn into_command(self) -> Self::Command;
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, MutGetters, TypedBuilder,
)]
pub struct InjectableCommand<Function, Context> {
    #[getset(get_mut = "pub")]
    function: Function,
    context: PhantomData<Context>,
}

macro_rules! impl_command {
  (
      $($params:ident),*
  ) => {
    #[allow(non_snake_case)]
    #[allow(unused)]
    impl<Function, $($params: ContextParameter),*> Command for InjectableCommand<Function, ($($params,)*)>
      where
        for<'a, 'b> &'a mut Function:
          AsyncFnMut(ArgMatches, $($params),* ) -> CommandResult +
          AsyncFnMut(ArgMatches, $(<$params as ContextParameter>::Item<'b>),* ) -> CommandResult
    {
      async fn run(&mut self, args: ArgMatches, contexts: &mut HashMap<TypeId, Box<dyn Any>>) -> CommandResult {
        // This call_inner is necessary to tell rust which function impl to call
        async fn call_inner<$($params),*>(
          mut f: impl AsyncFnMut(ArgMatches, $($params),*) -> CommandResult,
          args: ArgMatches,
          $($params: $params),*
        ) -> CommandResult {
          f(args, $($params),*).await
        }

        $(
          let $params = $params::retrieve(contexts);
        )*

        call_inner(self.function_mut(), args, $($params),*).await
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
    impl<Function, $($params: ContextParameter),*> IntoCommand<($($params,)*)> for Function
      where
        for<'a, 'b> &'a mut Function:
          AsyncFnMut(ArgMatches, $($params),* ) -> CommandResult +
          AsyncFnMut(ArgMatches, $(<$params as ContextParameter>::Item<'b>),* ) -> CommandResult
    {
      type Command = InjectableCommand<Self, ($($params,)*)>;

      fn into_command(self) -> Self::Command {
        InjectableCommand::builder()
            .function(self)
            .context(Default::default())
            .build()
      }
    }
  }
}

impl_into_command!();
impl_into_command!(T1);
impl_into_command!(T1, T2);
