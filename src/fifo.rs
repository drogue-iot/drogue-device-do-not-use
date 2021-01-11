use core::task::Context as FutureContext;
use core::cell::RefCell;
use core::task::{Waker, Poll};
use crate::component::Component;
use heapless::ArrayLength;
use heapless::spsc::{Queue, Producer, Consumer};
use core::cell::UnsafeCell;
use core::future::Future;
use core::pin::Pin;

pub struct Signaller {
    waker: RefCell<Option<Waker>>,
}

impl Signaller {
    pub fn new() -> Self {
        Self {
            waker: RefCell::new(None)
        }
    }

    pub fn set_waker(&self, waker: Waker) {
        self.waker.borrow_mut().replace(waker);
    }

    pub fn wake(&self) {
        let mut waker = self.waker.borrow_mut().take();
        if let Some(waker) = waker {
            waker.wake()
        }
    }
}

pub struct AsyncFifo<C: Component, N: ArrayLength<C::InboundMessage>> {
    queue: Queue<C::InboundMessage, N>,
    signaller: Signaller,
}

impl<C: Component, N: ArrayLength<C::InboundMessage>> AsyncFifo<C, N> {
    pub fn new() -> Self {
        Self {
            queue: Queue::new(),
            signaller: Signaller::new(),
        }
    }

    pub fn split(&mut self) -> (AsyncProducer<C::InboundMessage, N>, AsyncConsumer<C::InboundMessage, N>) {
        let (producer, consumer) = self.queue.split();

        (
            AsyncProducer::new(producer, &self.signaller),
            AsyncConsumer::new(consumer, &self.signaller),
        )
    }
}


pub struct AsyncProducer<'q, T, N: ArrayLength<T>> {
    inner: Producer<'q, T, N>,
    signaller: &'q Signaller,
}

impl<'q, T, N: ArrayLength<T>> AsyncProducer<'q, T, N> {
    pub fn new(producer: Producer<'q, T,N>, signaller: &'q Signaller) -> Self {
        Self {
            inner: producer,
            signaller,
        }
    }
    pub fn enqueue(&mut self, item: T) {
        self.inner.enqueue(item);
        self.signaller.wake();
    }
}


pub struct AsyncConsumer<'q, T, N: ArrayLength<T>> {
    inner: Consumer<'q, T, N>,
    signaller: &'q Signaller,
}



impl<'q, T, N: ArrayLength<T>> AsyncConsumer<'q, T, N> {
    pub fn new(consumer: Consumer<'q, T, N>, signaller: &'q Signaller) -> Self {
        Self {
            inner: consumer,
            signaller,
        }
    }

    pub async fn dequeue(&'static mut self) -> T {
        struct Dequeue<T: 'static, N: ArrayLength<T> + 'static> {
            //_marker: PhantomData<T>,
            consumer: UnsafeCell<&'static mut Consumer<'static, T, N>>,
            signaller: &'static Signaller,
        }

        impl<T: 'static, N: ArrayLength<T> + 'static> Future for Dequeue<T, N> {
            type Output = T;

            fn poll(self: Pin<&mut Self>, cx: &mut FutureContext<'_>) -> Poll<Self::Output> {
                unsafe {
                    let consumer = &mut *self.consumer.get();
                    if let Some(item) = consumer.dequeue() {
                        Poll::Ready(item)
                    } else {
                        self.signaller.set_waker( cx.waker().clone() );
                        Poll::Pending
                    }
                }
            }
        }

        Dequeue {
            //_marker: PhantomData::default(),
            consumer: UnsafeCell::new(&mut self.inner),
            signaller: self.signaller,
        }.await
    }
}


