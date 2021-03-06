use std::io;

use futures::Future;
use futures::Poll;
use futures::stream::Stream;

use tokio_core;
use tokio_core::LoopHandle;

use tokio_core::io::*;


#[allow(dead_code)]
pub fn oneshot<T : Send + 'static>(lh: LoopHandle) -> (Sender<T>, IoFuture<Receiver<T>>) {
    let (sender, receiver) = lh.channel();
    (
        Sender { sender: sender },
        receiver.map(|receiver| Receiver { receiver: Some(receiver) }).boxed()
    )
}

#[allow(dead_code)]
pub struct Sender<T> {
    sender: tokio_core::Sender<T>,
}

#[allow(dead_code)]
pub struct Receiver<T> {
    receiver: Option<tokio_core::Receiver<T>>,
}

#[allow(dead_code)]
impl<T> Sender<T> {
    pub fn send(self, t: T) -> io::Result<()> {
        self.sender.send(t)
    }
}

impl<T> Future for Receiver<T> {
    type Item = T;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let t = match &mut self.receiver {
            &mut None => panic!("cannot be polled twice"),
            &mut Some(ref mut receiver) => {
                match receiver.poll() {
                    Poll::NotReady => return Poll::NotReady,
                    Poll::Err(e) => return Poll::Err(e),
                    Poll::Ok(Some(t)) => t,
                    Poll::Ok(None) =>
                        return Poll::Err(io::Error::new(io::ErrorKind::Other,
                            "channel has been disconnected")),
                }
            }
        };

        self.receiver = None;

        Poll::Ok(t)
    }
}
