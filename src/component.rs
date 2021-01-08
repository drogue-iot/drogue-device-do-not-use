use core::cell::{RefCell, UnsafeCell};
use crate::fifo::{AsyncConsumer, AsyncFifo, AsyncProducer};
use crate::sink::{Sink, Handler};
use crate::container::{Container, ContainerStartContext};
use heapless::{
    consts::*,
};

pub trait Component: Sized {
    type InboundMessage;
    type OutboundMessage;

    fn start(&'static mut self, ctx: ComponentStartContext<Self>);
}

pub struct ComponentStartContext<C: Component>
    where C::OutboundMessage: 'static
{
    consumer: RefCell<AsyncConsumer<'static, C::InboundMessage, U32>>,
    sink: &'static dyn Sink<C::OutboundMessage>,
}

impl<C: Component> ComponentStartContext<C> {
    pub fn send(&self, message: C::OutboundMessage) {
        self.sink.send(message)
    }
}

impl<C: Component> Sink<C::OutboundMessage> for ComponentStartContext<C> {
    fn send(&self, message: <C as Component>::OutboundMessage) {
        self.sink.send(message);
    }
}

impl<C: Component> ComponentContext<C> {}

/// Wraps and takes ownership of a component.
/// Provides an async FIFO between the holder of the context
/// and the component's spawned tasks.
pub struct ComponentContext<C: Component> {
    component: UnsafeCell<C>,
    fifo: UnsafeCell<AsyncFifo<C, U32>>,
    producer: RefCell<Option<AsyncProducer<'static, C::InboundMessage, U32>>>,
}

impl<C: Component> ComponentContext<C> {
    pub fn new(component: C) -> Self {
        Self {
            component: UnsafeCell::new(component),
            fifo: UnsafeCell::new(AsyncFifo::new()),
            producer: RefCell::new(None),
        }
    }

    pub fn start<CN: Container>(&'static self, container: &'static ContainerStartContext<CN>)
        where CN: Handler<C::OutboundMessage>
    {
        let (producer, consumer) = unsafe { &mut *self.fifo.get() }.split();
        self.producer.borrow_mut().replace(producer);

        let start_context = ComponentStartContext {
            consumer: RefCell::new(consumer),
            sink: container,
        };

        unsafe { (&mut *self.component.get()) }.start(start_context);
    }

    pub fn send(&self, message: C::InboundMessage) {
        // TODO: critical section/lock
        self.producer.borrow_mut().as_mut().unwrap().enqueue(message)
    }
}

impl<C: Component> Sink<C::InboundMessage> for ComponentContext<C> {
    fn send(&self, message: <C as Component>::InboundMessage) {
        self.producer.borrow_mut().as_mut().unwrap().enqueue(message)
    }
}