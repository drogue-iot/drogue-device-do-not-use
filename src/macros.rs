
#[macro_export]
macro_rules! device {
    ($ty:ty => $kernel:expr) => {
        static mut KERNEL: Option<ConnectedKernel<$ty>> = None;

        let kernel = unsafe {
            KERNEL.replace(ConnectedKernel::new($kernel));
            KERNEL.as_ref().unwrap()
        };

        kernel.start();
    }
}