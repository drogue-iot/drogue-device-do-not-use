#![no_std]

/// Support for the root a component tree.
pub mod kernel;

/// Support for leaf or non-leaf components in a tree.
pub mod component;

/// Support for IRQ-based components.
pub mod interrupt;

#[doc(hidden)]
pub mod context;

#[doc(hidden)]
pub mod macros;

/// Support for handling messages outbound from child to parent.
pub mod handler;

mod fifo;

/// Quick imports of common traits and structs.
pub mod prelude {
    pub use crate::{
        kernel::{
            Kernel,
            KernelContext,
        },
        component::{
            Component,
            ConnectedComponent,
            ComponentContext,
            spawn,
        },
        interrupt::{
            Interrupt,
            ConnectedInterrupt,
            InterruptContext,
        },
        handler::Handler,
        device,
    };
}

#[cfg(test)]
mod tests {
    use crate::component::{Component, ComponentContext, ConnectedComponent};
    use crate::handler::Handler;
    use crate::interrupt::{ConnectedInterrupt, Interrupt, InterruptContext};
    use crate::kernel::{Kernel, KernelContext};
    use drogue_async::task::spawn;

    pub enum ButtonEvent {
        Pressed,
        Released,
    }

    pub struct Button {}

    impl Interrupt for Button {
        type OutboundMessage = ButtonEvent;

        fn on_interrupt(&mut self, context: &InterruptContext<Self>) {
            unimplemented!()
        }

        fn irq(&self) -> u8 {
            unimplemented!()
        }
    }

    pub enum LEDState {
        On,
        Off,
    }

    pub struct LED {}

    impl Component for LED {
        type InboundMessage = LEDState;
        type OutboundMessage = ();

        fn start(&mut self, ctx: &'static ComponentContext<Self>) {
            spawn("led", async move {
                loop {
                    let message = ctx.receive().await;
                    match message {
                        LEDState::On => {}
                        LEDState::Off => {}
                    }
                }
            });
        }
    }

    pub struct Flashlight {
        led: ConnectedComponent<LED>,
        button: ConnectedInterrupt<Button>,
    }

    pub enum FlashlightStatus {
        On,
        Off,
    }

    impl Component for Flashlight {
        type InboundMessage = ();
        type OutboundMessage = FlashlightStatus;

        fn start(&'static mut self, ctx: &'static ComponentContext<Self>) {
            //self.led.start(ctx);
            self.button.start(ctx);
        }
    }

    impl Handler<ButtonEvent> for Flashlight {
        fn on_message(&mut self, message: ButtonEvent) {
            match message {
                ButtonEvent::Pressed => {
                    self.led.send(LEDState::On);
                }
                ButtonEvent::Released => {
                    self.led.send(LEDState::Off);
                }
            }
        }
    }

    struct Device {
        flashlight: ConnectedComponent<Flashlight>,
    }

    impl Kernel for Device {
        fn start(&'static self, ctx: &'static KernelContext<Self>) {
            self.flashlight.start(ctx);
        }
    }

    impl Handler<FlashlightStatus> for Device {
        fn on_message(&mut self, message: FlashlightStatus) {
            unimplemented!()
        }
    }

    #[test]
    fn the_api() {
        use crate::device;

        let flashlight = Flashlight {
            led: ConnectedComponent::new(LED {}),
            button: ConnectedInterrupt::new(Button {}),
        };

        let kernel = Device {
            flashlight: ConnectedComponent::new(flashlight),
        };

        //device!( Device => kernel);
    }
}
