use core::cell::UnsafeCell;
use crate::sink::{Sink, Handler};
use crate::component::{Component, ComponentContext, ComponentStartContext};
use crate::kernel::{KernelStartContext, Kernel};
use crate::fifo::{AsyncFifo, AsyncProducer};
use core::cell::RefCell;
use heapless::{
    consts::*,
};

pub trait Container: Component + Sized {
    fn start<K:Kernel>(&'static self, context: &'static ContainerStartContext<Self>);
}

pub struct ContainerContext<C: Container + 'static> {
    container: UnsafeCell<C>,
    context: UnsafeCell<Option<ContainerStartContext<C>>>,

    component_context: UnsafeCell<Option<ComponentStartContext<C>>>,
    fifo: UnsafeCell<AsyncFifo<C, U32>>,
    producer: RefCell<Option<AsyncProducer<'static, C::InboundMessage, U32>>>,
}

impl<C: Container> ContainerContext<C> {
    pub fn new(container: C) -> Self {
        Self {
            container: UnsafeCell::new(container),
            context: UnsafeCell::new(None),

            component_context: UnsafeCell::new(None),
            fifo: UnsafeCell::new(AsyncFifo::new()),
            producer: RefCell::new(None),
        }
    }

    pub fn start<K:Kernel>(&'static self, kernel: &'static KernelStartContext<K>) {
        let start_context = ContainerStartContext::new(self);
        unsafe {
            // move the context to be static just like self
            (&mut *self.context.get()).replace(start_context);

            Container::start::<K>(
                (&mut *self.container.get()),
                (&mut *self.context.get()).as_ref().unwrap()
            );

            let (producer, consumer) = unsafe { &mut *self.fifo.get() }.split();
            self.producer.borrow_mut().replace(producer);

            let start_context = ComponentStartContext {
                consumer: RefCell::new(consumer),
                sink: kernel,
            };

            unsafe {
                (&mut *self.component_context.get()).replace(start_context);
                (&mut *self.component.get()).start(
                    (&*self.component_context.get()).as_ref().unwrap()
                );
            }

            Component::start(
                (&mut *self.container.get()),
                (&*self.component_context.get()).as_ref().unwrap()
            )
            //(&mut *self.container.get()).start(
                //(&mut *self.context.get()).as_ref().unwrap()
            //);
            /*
            (&mut *self.container.get()).start(
                (&mut *self.context.get()).as_ref().unwrap()
            );
             */
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