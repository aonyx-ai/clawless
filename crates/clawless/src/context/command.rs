use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::CommandResult;
use crate::context::ContextParameter;
use getset::MutGetters;
use typed_builder::TypedBuilder;

pub trait Command {
    fn run(&mut self, contexts: &mut HashMap<TypeId, Box<dyn Any>>) -> CommandResult;
}

pub trait IntoCommand<Input> {
    type Command: Command;

    fn into_command(self) -> Self::Command;
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, MutGetters, TypedBuilder,
)]
pub struct InjectableCommand<Input, Function> {
    #[getset(get_mut = "pub")]
    function: Function,

    marker: PhantomData<Input>,
}

macro_rules! impl_command {
  (
      $($params:ident),*
  ) => {
    #[allow(non_snake_case)]
    #[allow(unused)]
    impl<F, $($params: ContextParameter),*> Command for InjectableCommand<($($params,)*), F>
      where
        for<'a, 'b> &'a mut F:
          FnMut( $($params),* ) -> CommandResult +
          FnMut( $(<$params as ContextParameter>::Item<'b>),* ) -> CommandResult
    {
      fn run(&mut self, contexts: &mut HashMap<TypeId, Box<dyn Any>>) -> CommandResult {
        // This call_inner is necessary to tell rust which function impl to call
        fn call_inner<$($params),*>(
          mut f: impl FnMut($($params),*) -> CommandResult,
          $($params: $params),*
        ) -> CommandResult {
          f($($params),*)
        }

        $(
          let $params = $params::retrieve(contexts);
        )*

        call_inner(self.function_mut(), $($params),*)
      }
    }
  }
}

macro_rules! impl_into_command {
  (
    $($params:ident),*
  ) => {
    impl<F, $($params: ContextParameter),*> IntoCommand<($($params,)*)> for F
      where
        for<'a, 'b> &'a mut F:
          FnMut( $($params),* ) -> CommandResult +
          FnMut( $(<$params as ContextParameter>::Item<'b>),* ) -> CommandResult
    {
      type Command = InjectableCommand<($($params,)*), Self>;

      fn into_command(self) -> Self::Command {
        InjectableCommand::builder()
            .function(self)
            .marker(Default::default())
            .build()
      }
    }
  }
}

impl_command!();
impl_command!(T1);
impl_command!(T1, T2);

impl_into_command!();
impl_into_command!(T1);
impl_into_command!(T1, T2);
