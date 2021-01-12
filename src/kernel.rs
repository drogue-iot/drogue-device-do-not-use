use crate::context::UpstreamContext;
use crate::handler::{Handler, Sink};
use crate::interrupt::Interruptable;
use core::cell::{RefCell, UnsafeCell};
use cortex_m::interrupt::Nr;
use cortex_m::peripheral::NVIC;
use heapless::{consts::*, Vec};

#[doc(hidden)]
pub use drogue_async::executor::run_forever;

#[doc(hidden)]
pub use drogue_async::init_executor;

/// A the root of a Component/Interrupt tree of devices.
/// An application should implement this trait on their
/// base component. Additionally, the `Handler<M>` trait
/// should be implemented for all `::OutboundMessage` types
/// of children that the kernel holds, in order to route
/// messages.
///
/// Each child should be stored in a `ConnectedComponent<C>`
/// or a `ConnectedInterrupt<I>` as appropriate, to facilitate
/// the set up of the message topology and FIFOs.
pub trait Kernel: Sized {
    /// Start the tree of devices.
    ///
    /// For all children held by this kernel, they should be
    /// started in an application-appropriate order, passing
    /// the `ctx` through to them.
    fn start(&'static self, ctx: &'static KernelContext<Self>);
}

#[doc(hidden)]
pub struct ConnectedKernel<K: Kernel>
where
    K: 'static,
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
            (&*self.kernel.get()).start((&*self.context.get()).as_ref().unwrap());
        }
        self.irq_registry.borrow().unmask_all();
    }

    pub fn interrupt(&self, irqn: i16) {
        self.irq_registry.borrow().interrupt(irqn);
    }
}

/// Context used when calling `start(...)` on a `Kernel` implementation.
pub struct KernelContext<K: Kernel>
where
    K: 'static,
{
    kernel: &'static ConnectedKernel<K>,
}

impl<K: Kernel> KernelContext<K> {
    fn new(kernel: &'static ConnectedKernel<K>) -> Self {
        Self { kernel }
    }
}

impl<M, K: Kernel> Sink<M> for KernelContext<K>
where
    K: Handler<M>,
{
    fn send(&self, message: M) {
        unsafe { &mut *self.kernel.kernel.get() }.on_message(message)
    }
}

impl<M, K: Kernel> UpstreamContext<M> for KernelContext<K>
where
    K: Handler<M>,
{
    fn send(&self, message: M) {
        unsafe { &mut *self.kernel.kernel.get() }.on_message(message)
    }

    fn register_irq(&self, irq: u8, interrupt: &'static dyn Interruptable) {
        self.kernel
            .irq_registry
            .borrow_mut()
            .register(irq, interrupt);
    }
}

impl<K: Kernel> Handler<()> for K {
    fn on_message(&mut self, _message: ()) {
        // discard
    }
}

struct IrqRegistry {
    entries: Vec<IrqEntry, U16>,
}

impl IrqRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register(&mut self, irq: u8, interrupt: &'static dyn Interruptable) {
        self.entries.push(IrqEntry { irq, interrupt }).ok().unwrap();
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

struct IrqEntry {
    irq: u8,
    interrupt: &'static dyn Interruptable,
}
