use core::marker::PhantomData;
use crate::sink::{Handler, Sink};

pub trait Interrupt {
    type OutboundMessage;
}

pub struct InterruptContext<I: Interrupt> {
    _marker: PhantomData<I>,
}

/// Wraps and takes ownership of an interrupt.
/// Provides a synchronous sink for outbound messages.
pub struct ConnectedInterrupt<I: Interrupt> {
    interrupt: I,
}

impl<I: Interrupt> ConnectedInterrupt<I> {
    pub fn new(interrupt: I) -> Self {
        Self {
            interrupt,
        }
    }

    pub fn start(&self, container: &dyn Sink<I::OutboundMessage>)
    {

    }
}