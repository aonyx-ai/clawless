use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait ContextParameter {
    type Item<'new>;

    fn retrieve<'r>(contexts: &'r HashMap<TypeId, Box<dyn Any>>) -> Self::Item<'r>;
}

pub struct Context<'a, T: 'static> {
    value: &'a T,
}

impl<'res, T: 'static> ContextParameter for Context<'res, T> {
    type Item<'new> = Context<'new, T>;

    fn retrieve<'r>(resources: &'r HashMap<TypeId, Box<dyn Any>>) -> Self::Item<'r> {
        Context {
            value: resources
                .get(&TypeId::of::<T>())
                .unwrap()
                .downcast_ref()
                .unwrap(),
        }
    }
}
