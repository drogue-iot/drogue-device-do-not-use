use crate::context::UpstreamContext;
use crate::fifo::{AsyncConsumer, AsyncFifo, AsyncProducer};
use crate::handler::{Handler, Sink};
use crate::interrupt::Interruptable;
use core::cell::{RefCell, UnsafeCell};
use heapless::consts::*;
pub use drogue_async::task::spawn;

/// A non-root, but possibly leaf (or middle) portion of the component tree.
///
/// Each `Component` may have `::InboundMessage` and `::OutboundMessage` types
/// for communication with whichever `Component` or `Kernel` is _upstream_ from
/// itself. Typically these types will be `enum`s, and may be `()` if no applicable
/// message exists.  All messages are considered in relation to it's parent, regardless
/// of its children (if any), which are separately dealt with using `Handler<M>`
/// trait implementations.
pub trait Component: Sized {
    /// The type of message expected from its parent.
    type InboundMessage;

    /// The type of message sent to its parent.
    type OutboundMessage;

    /// Start this component and any children.
    ///
    /// Each child should be started in an application-appropriate order
    /// passing the `ctx` down the tree.
    ///
    /// `spawn(...)` may be used to initiate asynchronous tasks (generally loops)
    /// and `ctx.receive().await` may be used to asynchronously receive
    /// messages of `::InboundMessage` type using futures.
    fn start(&'static mut self, ctx: &'static ComponentContext<Self>);
}

/// Context provided to the component upon `start(...)`.
pub struct ComponentContext<C: Component>
where
    C: 'static,
{
    component: &'static ConnectedComponent<C>,
    consumer: UnsafeCell<AsyncConsumer<'static, C::InboundMessage, U32>>,
    upstream: &'static dyn UpstreamContext<C::OutboundMessage>,
}

impl<C: Component> ComponentContext<C> {
    fn new(
        component: &'static ConnectedComponent<C>,
        consumer: AsyncConsumer<'static, C::InboundMessage, U32>,
        upstream: &'static dyn UpstreamContext<C::OutboundMessage>,
    ) -> Self {
        Self {
            component,
            consumer: UnsafeCell::new(consumer),
            upstream,
        }
    }

    /// Send a message, *synchronously*, upstream to the containing
    /// parent `Component` or `Kernel`, which *must* implement
    /// `Handler<C::OutboundMessage>` to be able to accomodate
    /// messages from this child.
    ///
    /// This method is immediate and synchronous, avoiding any
    /// FIFOs. By the time it returns, the parent's associated
    /// `Handler<M>` will have been called and fulled executed.
    ///
    /// The component is *not* directly linked to the outbound
    /// messages, so if differentiation between components that
    /// can produce similar messages is required, a discriminant
    /// (possibly using a `PhantomData` field) may be required.
    pub fn send(&self, message: C::OutboundMessage) {
        self.upstream.send(message)
    }

    /// Receive a message, *asynchronously*, from the upstream
    /// `Component` or `Kernel` of type `C::InboundMessage`.
    pub async fn receive(&'static self) -> C::InboundMessage {
        unsafe { (&mut *self.consumer.get()).dequeue().await }
    }
}

impl<C: Component> Sink<C::OutboundMessage> for ComponentContext<C> {
    fn send(&self, message: C::OutboundMessage) {
        self.upstream.send(message);
    }
}

/// Wrapper for a `Component` to be held by the `Kernel`
/// or `Component` parent of this component. Components
/// shall not be held directly, but only through a `ConnectedComponent<C>`
/// which handles message routing and asynchronous FIFO configuration.
pub struct ConnectedComponent<C: Component>
where
    C: 'static,
{
    component: UnsafeCell<C>,
    context: UnsafeCell<Option<ComponentContext<C>>>,
    fifo: UnsafeCell<AsyncFifo<C, U32>>,
    producer: RefCell<Option<AsyncProducer<'static, C::InboundMessage, U32>>>,
}

impl<C: Component> ConnectedComponent<C> {
    /// Create a new wrapped `ConnectedComponent<C>` from a `Component`.
    pub fn new(component: C) -> Self {
        Self {
            component: UnsafeCell::new(component),
            context: UnsafeCell::new(None),
            fifo: UnsafeCell::new(AsyncFifo::new()),
            producer: RefCell::new(None),
        }
    }

    /// Start this component and it's associated asynchronous FIFO.
    ///
    /// This method should be invoked with the `ctx` passed to it's
    /// parent's own `start(...)` method.
    pub fn start(&'static self, upstream: &'static dyn UpstreamContext<C::OutboundMessage>) {
        let (producer, consumer) = unsafe { &mut *self.fifo.get() }.split();
        self.producer.borrow_mut().replace(producer);

        let context = ComponentContext::new(&self, consumer, upstream);

        unsafe {
            (&mut *self.context.get()).replace(context);
            (&mut *self.component.get()).start((&*self.context.get()).as_ref().unwrap());
        }
    }

    /// Send a message of type `::InboundMessag` to the contained component.
    ///
    /// This method should be used only by the directly-owneding parent of
    /// the wrapped component.
    ///
    /// TODO: There is currently no policy regarding full FIFOs and messages will simply be dropped.
    pub fn send(&self, message: C::InboundMessage) {
        // TODO: critical section/lock
        self.producer
            .borrow_mut()
            .as_mut()
            .unwrap()
            .enqueue(message)
    }
}

impl<C: Component> Sink<C::InboundMessage> for ConnectedComponent<C> {
    fn send(&self, message: <C as Component>::InboundMessage) {
        self.producer
            .borrow_mut()
            .as_mut()
            .unwrap()
            .enqueue(message)
    }
}

impl<M, C: Component> UpstreamContext<M> for ComponentContext<C>
where
    C: Handler<M>,
{
    fn send(&self, message: M) {
        unsafe { &mut *self.component.component.get() }.on_message(message)
    }

    fn register_irq(&self, irq: u8, interrupt: &'static dyn Interruptable) {
        self.upstream.register_irq(irq, interrupt)
    }
}
