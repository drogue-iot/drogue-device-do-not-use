use std::cell::UnsafeCell;
use crate::sink::{Sink, Handler};
use crate::component::Component;
use crate::container::Container;

pub trait Kernel : Sized {
    fn start(&'static self, ctx: &'static KernelStartContext<Self>);
}

pub struct KernelContext<K: Kernel>
where K: 'static
{
    kernel: UnsafeCell<K>,
    context: UnsafeCell<Option<KernelStartContext<K>>>,
}

impl<K: Kernel> KernelContext<K> {
    pub fn new(kernel: K) -> Self {
        Self {
            kernel: UnsafeCell::new(kernel),
            context: UnsafeCell::new(None),
        }
    }

    pub fn start(&'static self) {
        let start_context = KernelStartContext::new(&self);
        unsafe {
            (&mut *self.context.get()).replace(start_context);
            (&mut *self.kernel.get()).start( (&*self.context.get()).as_ref().unwrap() );
        }
    }
}

pub struct KernelStartContext<K: Kernel>
    where K: 'static
{
    kernel: &'static KernelContext<K>,
}

impl<K: Kernel> KernelStartContext<K> {
    pub fn new(kernel: &'static KernelContext<K>) -> Self {
        Self {
            kernel
        }
    }
}

impl<M, K: Kernel> Sink<M> for KernelStartContext<K>
    where K: Handler<M>
{
    fn send(&self, message: M) {
        unsafe { &mut *self.kernel.kernel.get() }.on_message(message)
    }
}

