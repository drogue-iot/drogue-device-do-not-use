pub(crate) trait Sink<M> {
    fn send(&self, message: M);
}

/// Trait indicating that a `Kernel` or `Component` can handle
/// a child component/interrupt's `::OutboundMessage`.
pub trait Handler<M> {
    /// Handle a child's `::OutboundMessage` synchronously.
    /// As this is synchronous, it *must* be non-blocking and
    /// quick to complete.
    ///
    /// An implementation *may* call `send(...)` on children
    /// `ConnectedComponent<C>` or `ConnectedInterrupt<I>` safely,
    /// as each of those calls are considered *non-blocking* and return
    /// immediately, as they only enqueue a message on a FIFO.
    fn on_message(&mut self, message: M);
}
