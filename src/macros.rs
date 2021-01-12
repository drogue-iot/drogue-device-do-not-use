/// Configure and start a device `Kernel`.
///
/// Additionally, allocate some number of bytes for the async executor.
///
/// For example:
///
/// ```
/// use drogue_device::kernel::{Kernel, KernelContext};
/// use drogue_device::component::ConnectedComponent;
/// struct MyDevice {
///    led: ConnectedComponent<LED>,
/// }
///
/// impl Kernel for MyDevice {
///     fn start(&'static self,ctx: &'static KernelContext<Self>) {
///         self.led.start( ctx );
///     }
/// }
///
/// device!( MyDevice => Kernel; 1024 );
/// ```
#[macro_export]
macro_rules! device {
    ($ty:ty => $kernel:expr; $memory:literal) => {
        $crate::kernel::init_executor!(memory: $memory);
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

        $crate::kernel::run_forever()
    };
}
