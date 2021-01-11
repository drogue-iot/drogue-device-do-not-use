
pub trait Sink<M> {
    fn send(&self, message: M);
}

pub trait Handler<M> {
    fn on_message(&mut self, message: M);
}
