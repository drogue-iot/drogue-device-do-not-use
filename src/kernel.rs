use core::cell::{UnsafeCell, RefCell};
use crate::sink::{Sink, Handler};
use crate::context::UpstreamContext;
use crate::interrupt::Interruptable;
use heapless::{
    Vec,
    consts::*,
};
use cortex_m::peripheral::NVIC;
use cortex_m::interrupt::Nr;

pub trait Kernel: Sized {
    fn start(&'static self, ctx: &'static KernelContext<Self>);
}

pub struct ConnectedKernel<K: Kernel>
    where K: 'static
{
    kernel: UnsafeCell<K>,
    context: UnsafeCell<Option<KernelContext<K>>>,
    irq_registry: RefCell<IrqRegistry>,
}

impl<K: Kernel> ConnectedKernel<K> {
    pub fn new(kernel: K) -> Self {
        Self {
            kernel: UnsafeCell::new(kernel),
            context: UnsafeCell::new(None),
            irq_registry: RefCell::new(IrqRegistry::new()),
        }
    }

    pub fn start(&'static self) {
        let context = KernelContext::new(&self);
        unsafe {
            (&mut *self.context.get()).replace(context);
            (&*self.kernel.get()).start(
                (&*self.context.get()).as_ref().unwrap()
            );
        }
        self.irq_registry.borrow().unmask_all();
    }

    pub fn interrupt(&self, irqn: i16) {
        self.irq_registry.borrow().interrupt(irqn);
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

impl<M, K: Kernel> UpstreamContext<M> for KernelContext<K>
    where K: Handler<M>
{
    fn send(&self, message: M) {
        unsafe { &mut *self.kernel.kernel.get() }.on_message(message)
    }

    fn register_irq(&self, irq: u8, interrupt: &'static dyn Interruptable) {
        self.kernel.irq_registry.borrow_mut().register(irq, interrupt);
    }
}

impl<K: Kernel> Handler<()> for K {
    fn on_message(&mut self, _message: ()) {
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
        ).ok().unwrap();
    }

    pub fn interrupt(&self, irqn: i16) {
        for interrupt in self.entries.iter().filter(|e| e.irq == irqn as u8) {
            interrupt.interrupt.interrupt();
        }
    }

    pub fn unmask_all(&self) {
        for irq in self.entries.iter().map(|e| e.irq) {
            struct IrqNr(u8);
            unsafe impl Nr for IrqNr {
                fn nr(&self) -> u8 {
                    self.0
                }
            }

            unsafe {
                NVIC::unmask(IrqNr(irq));
            }
        }
    }
}

impl Default for IrqRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct IrqEntry {
    irq: u8,
    interrupt: &'static dyn Interruptable,
}

