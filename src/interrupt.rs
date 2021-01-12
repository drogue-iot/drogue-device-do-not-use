use crate::context::UpstreamContext;
use core::cell::UnsafeCell;

/// A leaf component representing IRQ logic.
///
/// Being an interrupt, it has no sense of *inbound* messages, but
/// can producer `::OutboundMessage`s to its containing parent
/// `Component` or `Kernel`.
pub trait Interrupt: Sized {
    /// The type of message sent to its parent.
    type OutboundMessage;

    /// The IRQ number to which this `Interrupt` should respond.
    fn irq(&self) -> u8;

    /// The action to undertake when the associated interrupt line is triggered.
    fn on_interrupt(&mut self, context: &InterruptContext<Self>);
}

/// The context provided to the `Interrupt` during it's `on_interrupt(...)` invocation.
pub struct InterruptContext<I: Interrupt>
where
    I: 'static,
{
    _interrupt: &'static ConnectedInterrupt<I>,
    upstream: &'static dyn UpstreamContext<I::OutboundMessage>,
}

impl<I: Interrupt> InterruptContext<I> {
    fn new(
        interrupt: &'static ConnectedInterrupt<I>,
        upstream: &'static dyn UpstreamContext<I::OutboundMessage>,
    ) -> Self {
        Self {
            _interrupt: interrupt,
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
    pub fn send(&self, message: I::OutboundMessage) {
        self.upstream.send(message)
    }
}

/// Wrapper for an `Interrupt` to be held by the `Kernel`
/// or a `Component` parent of this interrupt. Interrupts shall
/// not be held directly, but only through a `ConnectedInterrupt<I>`
/// which handles message routing.
pub struct ConnectedInterrupt<I: Interrupt>
where
    I: 'static,
{
    interrupt: UnsafeCell<I>,
    context: UnsafeCell<Option<InterruptContext<I>>>,
}

impl<I: Interrupt> ConnectedInterrupt<I> {
    /// Create a new wrapped `ConnectedInterrupt<I>` from an `Interrupt`.
    pub fn new(interrupt: I) -> Self {
        Self {
            interrupt: UnsafeCell::new(interrupt),
            context: UnsafeCell::new(None),
        }
    }

    /// Start this interrupt.
    ///
    /// This method should be invoked with the `ctx` passed to it's
    /// parent's own `start(...)` method.
    pub fn start(&'static self, upstream: &'static dyn UpstreamContext<I::OutboundMessage>) {
        let context = InterruptContext::new(self, upstream);

        unsafe {
            context
                .upstream
                .register_irq((&*self.interrupt.get()).irq(), self);

            (&mut *self.context.get()).replace(context);
        }
    }
}

impl<I: Interrupt> Interruptable for ConnectedInterrupt<I> {
    fn interrupt(&self) {
        unsafe {
            (&mut *self.interrupt.get()).on_interrupt((&*self.context.get()).as_ref().unwrap());
        }
    }
}

#[doc(hidden)]
pub trait Interruptable {
    fn interrupt(&self);
}
