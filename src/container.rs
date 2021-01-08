use core::cell::UnsafeCell;
use crate::sink::{Sink, Handler};

pub trait Container: Sized {
    fn start(&'static self, context: &'static ContainerStartContext<Self>);
}

pub struct ContainerContext<C: Container + 'static> {
    container: UnsafeCell<C>,
    context: UnsafeCell<Option<ContainerStartContext<C>>>,
}

impl<C: Container> ContainerContext<C> {
    pub fn new(container: C) -> Self {
        Self {
            container: UnsafeCell::new(container),
            context: UnsafeCell::new(None),
        }
    }

    pub fn start(&'static self) {
        let start_context = ContainerStartContext::new(self);
        unsafe {
            // move the context to be static just like self
            (&mut *self.context.get()).replace(start_context);

            (&mut *self.container.get()).start(
                (&mut *self.context.get()).as_ref().unwrap()
            );
        };
    }
}

pub struct ContainerStartContext<C: Container>
    where C: 'static
{
    container: &'static ContainerContext<C>,
}

impl<C: Container> ContainerStartContext<C> {
    fn new(container: &'static ContainerContext<C>) -> Self {
        Self {
            container
        }
    }
}

impl<M, C: Container> Sink<M> for ContainerStartContext<C>
    where C: Handler<M>
{
    fn send(&self, message: M) {
        unsafe { &mut *self.container.container.get() }.on_message(message)
    }
}

impl<C: Container> Handler<()> for C {
    fn on_message(&mut self, message: ()) {
        // discard
    }
}