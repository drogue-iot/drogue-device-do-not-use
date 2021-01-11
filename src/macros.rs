
#[macro_export]
macro_rules! device {
    ($ty:ty => $kernel:expr; $memory:literal) => {

        $crate::drogue_async::init_executor!( memory: $memory );
        static mut KERNEL: Option<$crate::kernel::ConnectedKernel<$ty>> = None;

        let kernel = unsafe {
            KERNEL.replace($crate::kernel::ConnectedKernel::new($kernel));
            KERNEL.as_ref().unwrap()
        };

        kernel.start();

        #[exception]
        fn DefaultHandler(irqn: i16) {
            unsafe {
                KERNEL.as_ref().unwrap().interrupt(irqn);
            }
        }

        $crate::drogue_async::executor::run_forever()
    }
}