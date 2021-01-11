use core::cell::{RefCell, UnsafeCell};
use crate::fifo::{AsyncConsumer, AsyncFifo, AsyncProducer};
use crate::sink::{Sink, Handler};
use heapless::{
    consts::*,
};
use crate::context::UpstreamContext;
use crate::interrupt::Interruptable;

pub trait Component: Sized {
    type InboundMessage;
    type OutboundMessage;

    fn start(&'static mut self, ctx: &'static ComponentContext<Self>);
}

pub struct ComponentContext<C: Component>
    where C: 'static
{
    component: &'static ConnectedComponent<C>,
    pub(crate) consumer: UnsafeCell<AsyncConsumer<'static, C::InboundMessage, U32>>,
    pub(crate) upstream: &'static dyn UpstreamContext<C::OutboundMessage>,
}

impl<C: Component> ComponentContext<C> {
    fn new(component: &'static ConnectedComponent<C>, consumer: AsyncConsumer<'static, C::InboundMessage, U32>, upstream: &'static dyn UpstreamContext<C::OutboundMessage>) -> Self {
        Self {
            component: component,
            consumer: UnsafeCell::new(consumer),
            upstream,
        }
    }

    pub fn send(&self, message: C::OutboundMessage) {
        self.upstream.send(message)
    }

    pub async fn receive(&'static self) -> C::InboundMessage {
        unsafe {
            (&mut *self.consumer.get()).dequeue().await
        }
    }
}

impl<C: Component> Sink<C::OutboundMessage> for ComponentContext<C> {
    fn send(&self, message: C::OutboundMessage) {
        self.upstream.send(message);
    }
}

/*
impl<M, C:Component> HandlerSink<M> for ComponentContext<C>
    where C: Handler<M>
{
    fn send(&self, message: M) {
        //self.component.on_message( message );
        unsafe {
            HandlerSink::send(&(**self.component.get()), message);
        }
    }
}

impl<M, C:Component> HandlerSink<M> for ConnectedComponent<C>
    where C: Handler<M>
{
    fn send(&self, message: M) {
        //self.component.on_message( message );
        unsafe {
            &mut (*self.component.get()).on_message(message);
        }
    }
}
 */


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

    pub fn start(&'static self, upstream: &'static dyn UpstreamContext<C::OutboundMessage>)
    {
        let (producer, consumer) = unsafe { &mut *self.fifo.get() }.split();
        self.producer.borrow_mut().replace(producer);

        let context = ComponentContext::new(&self, consumer, upstream);

        unsafe {
            (&mut *self.context.get()).replace(context);
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

impl<M, C:Component> UpstreamContext<M> for ComponentContext<C>
    where C: Handler<M>
{
    fn send(&self, message: M) {
        unsafe { &mut *self.component.component.get() }.on_message(message)
    }

    fn register_irq(&self, irq: u8, interrupt: &'static dyn Interruptable) {
        self.upstream.register_irq(irq, interrupt)
    }
}

/*
impl<C> Handler<()> for C {
    fn on_message(&mut self, message: ()) {
        // discard
    }
}

 */