use core::marker::PhantomData;
use crate::container::{Container, ContainerStartContext};
use crate::sink::Handler;

pub trait Interrupt {
    type OutboundMessage;
}

pub struct InterruptStartContext<I: Interrupt> {
    _marker: PhantomData<I>,
}

/// Wraps and takes ownership of an interrupt.
/// Provides a synchronous sink for outbound messages.
pub struct InterruptContext<I: Interrupt> {
    interrupt: I,
}

impl<I: Interrupt> InterruptContext<I> {
    pub fn new(interrupt: I) -> Self {
        Self {
            interrupt,
        }
    }

    pub fn start<CN: Container>(&self, container: &ContainerStartContext<CN>)
        where CN: Handler<I::OutboundMessage>
    {}
}