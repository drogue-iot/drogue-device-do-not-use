
pub trait Sink<M> {
    fn send(&self, message: M);
}

pub trait Handler<M> {
    fn on_message(&mut self, message: M);
}

//pub trait HandlerSink<M> {
    //fn send(&self, message: M);
//}

//impl<M> HandlerSink<M> for Handler<M> {
    //fn send(&self, message: M) {
        //self.on_message(message)
    //}
//}
