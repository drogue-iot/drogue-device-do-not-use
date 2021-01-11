use core::cell::UnsafeCell;
use crate::sink::{Sink, Handler};
use crate::component::Component;
use heapless::{
    String,
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
}

impl<K: Kernel> ConnectedKernel<K> {
    pub fn new(kernel: K) -> Self {
        Self {
            kernel: UnsafeCell::new(kernel),
            context: UnsafeCell::new(None),
        }
    }

    pub fn start(&'static self) {
        let start_context = KernelContext::new(&self);
        unsafe {
            (&mut *self.context.get()).replace(start_context);
            (&mut *self.kernel.get()).start( (&*self.context.get()).as_ref().unwrap() );
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


