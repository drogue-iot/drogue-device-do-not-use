pub trait Kernel {
    fn start(&'static self);
}

pub struct KernelContext<K: Kernel> {
    kernel: K,
}

impl<K:Kernel> KernelContext<K> {
    pub fn new(kernel: K) -> Self {
        Self {
            kernel,
        }
    }

    pub fn start(&'static self) {
        self.kernel.start();
    }
}