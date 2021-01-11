use crate::interrupt::Interruptable;

pub trait UpstreamContext<M> {
    fn send(&self, message: M);
    fn register_irq(&self, irq: u8, interrupt: &'static dyn Interruptable);
}