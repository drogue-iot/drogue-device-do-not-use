#![no_std]

mod fifo;
pub mod sink;
pub mod kernel;
pub mod component;
pub mod interrupt;
pub mod macros;
pub mod context;

pub extern crate drogue_async;

#[cfg(test)]
mod tests {
    use crate::sink::{Handler};
    use crate::kernel::{Kernel, KernelContext, IrqRegistry};
    use crate::component::{Component, ComponentContext, ConnectedComponent};
    use crate::interrupt::{Interrupt, ConnectedInterrupt, InterruptContext};
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
