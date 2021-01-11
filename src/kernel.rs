use core::cell::UnsafeCell;
use crate::sink::{Sink, Handler};
use crate::component::Component;
use crate::context::UpstreamContext;
use crate::interrupt::Interruptable;
use heapless::{
    Vec,
    consts::*,
};

pub trait Kernel : Sized {
    fn start(&'static self, ctx: &'static KernelContext<Self>);
}

pub struct ConnectedKernel<K: Kernel>
where K: 'static
{
    kernel: UnsafeCell<K>,
    context: UnsafeCell<Option<KernelContext<K>>>,
    irq_registry: UnsafeCell<IrqRegistry>,
}

impl<K: Kernel> ConnectedKernel<K> {
    pub fn new(kernel: K) -> Self {
        Self {
            kernel: UnsafeCell::new(kernel),
            context: UnsafeCell::new(None),
            irq_registry: UnsafeCell::new(IrqRegistry::new()),
        }
    }

    pub fn start(&'static self) {
        let context = KernelContext::new(&self);
        unsafe {
            (&mut *self.context.get()).replace(context);
            (&mut *self.kernel.get()).start(
                (&*self.context.get()).as_ref().unwrap()
            );
        }
    }

    pub fn interrupt(&self, irqn: i16) {
        unsafe {
            (&*self.irq_registry.get()).interrupt( irqn );
        }
    }
}

pub struct KernelContext<K: Kernel>
    where K: 'static
{
    kernel: &'static ConnectedKernel<K>,
}

impl<K: Kernel> KernelContext<K> {
    pub fn new(kernel: &'static ConnectedKernel<K>) -> Self {
        Self {
            kernel
        }
    }
}

impl<M, K: Kernel> Sink<M> for KernelContext<K>
    where K: Handler<M>
{
    fn send(&self, message: M) {
        unsafe { &mut *self.kernel.kernel.get() }.on_message(message)
    }
}

impl<M, K:Kernel> UpstreamContext<M> for KernelContext<K>
    where K: Handler<M>
{
    fn send(&self, message: M) {
        unsafe { &mut *self.kernel.kernel.get() }.on_message(message)
    }

    fn register_irq(&self, irq: u8, interrupt: &'static dyn Interruptable) {
        unsafe {
            (&mut *self.kernel.irq_registry.get()).register( irq, interrupt);
        }
    }
}

impl<K:Kernel> Handler<()> for K {
    fn on_message(&mut self, message: ()) {
        // discard

    }
}

pub struct IrqRegistry {
    entries: Vec<IrqEntry, U16>,
}

impl IrqRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register(&mut self, irq: u8, interrupt: &'static dyn Interruptable) {
        self.entries.push(
            IrqEntry {
                irq,
                interrupt,
            }
        );
    }

    pub fn interrupt(&self, irqn: i16) {
        for interrupt in self.entries.iter().filter( |e| e.irq == irqn as u8) {
            interrupt.interrupt.interrupt(irqn)
        }
    }
}

pub struct IrqEntry {
    irq: u8,
    interrupt: &'static dyn Interruptable,
}

