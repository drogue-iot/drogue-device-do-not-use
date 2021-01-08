
#[macro_export]
macro_rules! device {
    ($ty:ty => $kernel:expr) => {
        static mut KERNEL: Option<KernelContext<$ty>> = None;

        let kernel = unsafe {
            KERNEL.replace(KernelContext::new($kernel));
            KERNEL.as_ref().unwrap()
        };

        kernel.start();
    }
}