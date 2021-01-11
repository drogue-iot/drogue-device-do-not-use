use crate::sink::{Sink};
use core::cell::UnsafeCell;
use crate::context::UpstreamContext;
use crate::kernel::IrqRegistry;

pub trait Interrupt: Sized {
    type OutboundMessage;

    fn irq(&self) -> u8;
    fn on_interrupt(&mut self, context: &InterruptContext<Self>);
}

pub struct InterruptContext<I: Interrupt>
    where I: 'static
{
    interrupt: &'static ConnectedInterrupt<I>,
    upstream: &'static dyn UpstreamContext<I::OutboundMessage>,
}

impl<I: Interrupt> InterruptContext<I> {
    pub fn new(interrupt: &'static ConnectedInterrupt<I>, upstream: &'static dyn UpstreamContext<I::OutboundMessage>) -> Self {
        Self {
            interrupt,
            upstream,
        }
    }

    pub fn send(&self, message: I::OutboundMessage) {
        self.upstream.send(message)
    }
}

/// Wraps and takes ownership of an interrupt.
/// Provides a synchronous sink for outbound messages.
pub struct ConnectedInterrupt<I: Interrupt>
    where I: 'static
{
    interrupt: UnsafeCell<I>,
    context: UnsafeCell<Option<InterruptContext<I>>>,
}

impl<I: Interrupt> ConnectedInterrupt<I> {
    pub fn new(interrupt: I) -> Self {
        Self {
            interrupt: UnsafeCell::new(interrupt),
            context: UnsafeCell::new(None),
        }
    }

    pub fn start(&'static self, upstream: &'static dyn UpstreamContext<I::OutboundMessage>)
    {
        let context = InterruptContext::new(self, upstream);

        unsafe {
            context.upstream.register_irq(
                (&*self.interrupt.get()).irq(),
                self,
            );

            (&mut *self.context.get()).replace(context);
        }
    }
}

impl<I: Interrupt> Interruptable for ConnectedInterrupt<I> {
    fn interrupt(&self, irqn: i16) {
        unsafe {
            (&mut *self.interrupt.get()).on_interrupt( (&*self.context.get()).as_ref().unwrap() );
        }
    }
}

pub trait Interruptable {
    fn interrupt(&self, irqn: i16);
}
