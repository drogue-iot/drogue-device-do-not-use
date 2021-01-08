mod fifo;
mod sink;
mod kernel;
mod container;
mod component;
mod interrupt;
mod macros;

#[cfg(test)]
mod tests {
    use crate::sink::Handler;
    use crate::kernel::{Kernel, KernelContext, KernelStartContext};
    use crate::container::{Container, ContainerStartContext, ContainerContext};
    use crate::component::{Component, ComponentStartContext, ComponentContext};
    use crate::interrupt::{Interrupt, InterruptContext};

    pub enum ButtonEvent {
        Pressed,
        Released,
    }

    pub struct Button {}

    impl Interrupt for Button {
        type OutboundMessage = ButtonEvent;
    }

    pub enum LEDState {
        On,
        Off,
    }

    pub struct LED {}

    impl Component for LED {
        type InboundMessage = LEDState;
        type OutboundMessage = ();

        fn start(&mut self, ctx: &'static ComponentStartContext<Self>) {
            println!("starting LED");
        }
    }

    pub struct Flashlight {
        led: ComponentContext<LED>,
        button: InterruptContext<Button>,
    }

    impl Flashlight {}

    impl Container for Flashlight {
        fn start<K:Kernel>(&'static self, context: &'static ContainerStartContext<Self>) {
            self.led.start(context);
            self.button.start(context);
        }
    }

    pub enum FlashlightStatus {
        On,
        Off,
    }

    impl Component for Flashlight {
        type InboundMessage = ();
        type OutboundMessage = FlashlightStatus;
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
        flashlight: ContainerContext<Flashlight>,
    }

    impl Kernel for Device {
        fn start(&'static self, ctx: &'static KernelStartContext<Self>) {
            self.flashlight.start(ctx);
        }
    }

    #[test]
    fn the_api() {
        use crate::device;

        let flashlight = Flashlight {
            led: ComponentContext::new(LED {}),
            button: InterruptContext::new(Button {}),
        };

        let kernel = Device {
            flashlight: ContainerContext::new(flashlight),
        };

        device!( Device => kernel);
    }
}
