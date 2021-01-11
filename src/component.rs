use core::cell::{RefCell, UnsafeCell};
use crate::fifo::{AsyncConsumer, AsyncFifo, AsyncProducer};
use crate::sink::{Sink, Handler};
use heapless::{
    consts::*,
};

pub trait Component: Sized {
    type InboundMessage;
    type OutboundMessage;

    fn start(&'static mut self, ctx: &'static ComponentContext<Self>) {}
}

pub struct ComponentContext<C: Component>
    where C: 'static
{
    pub(crate) consumer: UnsafeCell<AsyncConsumer<'static, C::InboundMessage, U32>>,
    pub(crate) sink: &'static dyn Sink<C::OutboundMessage>,
}

impl<C: Component> ComponentContext<C> {
    pub fn send(&self, message: C::OutboundMessage) {
        self.sink.send(message)
    }

    pub async fn receive(&'static self) -> C::InboundMessage {
        unsafe {
            (&mut *self.consumer.get()).dequeue().await
        }
    }
}

impl<C: Component> Sink<C::OutboundMessage> for ComponentContext<C> {
    fn send(&self, message: C::OutboundMessage) {
        self.sink.send(message);
    }
}

/// Wraps and takes ownership of a component.
/// Provides an async FIFO between the holder of the context
/// and the component's spawned tasks.
pub struct ConnectedComponent<C: Component>
    where C: 'static
{
    component: UnsafeCell<C>,
    context: UnsafeCell<Option<ComponentContext<C>>>,
    fifo: UnsafeCell<AsyncFifo<C, U32>>,
    producer: RefCell<Option<AsyncProducer<'static, C::InboundMessage, U32>>>,
}

impl<C: Component> ConnectedComponent<C> {
    pub fn new(component: C) -> Self {
        Self {
            component: UnsafeCell::new(component),
            context: UnsafeCell::new(None),
            fifo: UnsafeCell::new(AsyncFifo::new()),
            producer: RefCell::new(None),
        }
    }

    pub fn start(&'static self, container: &'static dyn Sink<C::OutboundMessage>)
    {
        let (producer, consumer) = unsafe { &mut *self.fifo.get() }.split();
        self.producer.borrow_mut().replace(producer);

        let start_context = ComponentContext {
            consumer: UnsafeCell::new(consumer),
            sink: container,
        };

        unsafe {
            (&mut *self.context.get()).replace(start_context);
            (&mut *self.component.get()).start(
                (&*self.context.get()).as_ref().unwrap()
            );
        }
    }

    pub fn send(&self, message: C::InboundMessage) {
        // TODO: critical section/lock
        self.producer.borrow_mut().as_mut().unwrap().enqueue(message)
    }
}

impl<C: Component> Sink<C::InboundMessage> for ConnectedComponent<C> {
    fn send(&self, message: <C as Component>::InboundMessage) {
        self.producer.borrow_mut().as_mut().unwrap().enqueue(message)
    }
}