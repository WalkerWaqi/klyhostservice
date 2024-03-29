use bytes::BytesMut;
use futures::SinkExt;
use log::{error, info};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::UnixStream;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::{Stream, StreamExt};
use tokio_util::codec::{BytesCodec, Decoder, Framed};

type Tx = mpsc::UnboundedSender<BytesMut>;

#[allow(dead_code)]
pub struct UnixSocket {
    guests: HashMap<String, Tx>,
}

#[allow(dead_code)]
impl UnixSocket {
    fn new() -> Self {
        Self {
            guests: HashMap::new(),
        }
    }

    pub fn get_instance() -> Arc<Mutex<UnixSocket>> {
        static mut POINT: Option<Arc<Mutex<UnixSocket>>> = None;

        unsafe {
            POINT
                .get_or_insert_with(|| Arc::new(Mutex::new(UnixSocket::new())))
                .clone()
        }
    }

    async fn send(&mut self, path: &str, message: &str) -> Result<(), Box<dyn Error>> {
        if let Some(guest) = self.guests.get(path) {
            guest.send(message.into())?;
        } else {
            error!("error path {}", path);
        }

        Ok(())
    }

    pub async fn run(&self, path: String) -> Result<(), Box<dyn Error>> {
        let stream = UnixStream::connect(Path::new(&path)).await?;
        tokio::spawn(async move {
            info!("connection to {}", path);
            if let Err(e) = Self::process(stream, path).await {
                error!("an error occurred; error = {:?}", e);
            }
        });

        Ok(())
    }

    async fn process(stream: UnixStream, path: String) -> Result<(), Box<dyn Error>> {
        let lines = BytesCodec::new().framed(stream);

        // Register our peer with state which internally sets up some channels.
        let mut guest = Guest::new(Self::get_instance(), lines).await?;

        // Process incoming messages until our stream is exhausted by a disconnect.
        while let Some(result) = guest.next().await {
            match result {
                // A message send to peer
                Ok(Message::Send(msg)) => {
                    guest.lines.send(msg.freeze()).await?;
                }
                // A message was received from a peer
                Ok(Message::Received(msg)) => {
                    info!("received from {}: {}", &path, msg.len());
                }
                Err(e) => {
                    error!("error = {:?}", e);
                }
            }
        }

        // If this section is reached it means that the client was disconnected!
        // Let's let everyone still connected know about it.
        {
            info!("disconnection from {}", &path);
            Self::get_instance().lock().await.guests.remove(&path);
        }

        Ok(())
    }
}

#[allow(dead_code)]
struct Guest {
    lines: Framed<UnixStream, BytesCodec>,
    rx: Pin<Box<dyn Stream<Item = BytesMut> + Send>>,
}

#[derive(Debug)]
enum Message {
    /// A message that should be send
    Send(BytesMut),

    /// A message that should be received by a client
    Received(BytesMut),
}

// Peer implements `Stream` in a way that polls both the `Rx`, and `Framed` types.
// A message is produced whenever an event is ready until the `Framed` stream returns `None`.
impl Stream for Guest {
    type Item = Result<Message, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // First poll the `UnboundedReceiver`.

        if let Poll::Ready(Some(v)) = Pin::new(&mut self.rx).poll_next(cx) {
            return Poll::Ready(Some(Ok(Message::Send(v))));
        }

        // Secondly poll the `Framed` stream.
        let result: Option<_> = futures::ready!(Pin::new(&mut self.lines).poll_next(cx));

        Poll::Ready(match result {
            // We've received a message we should broadcast to others.
            Some(Ok(message)) => Some(Ok(Message::Received(message))),

            // An error occurred.
            Some(Err(e)) => Some(Err(e)),

            // The stream has been exhausted.
            None => None,
        })
    }
}

impl Guest {
    /// Create a new instance of `Peer`.
    async fn new(
        state: Arc<Mutex<UnixSocket>>,
        lines: Framed<UnixStream, BytesCodec>,
    ) -> io::Result<Guest> {
        // Create a channel for this peer
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map.
        state.lock().await.guests.insert(
            lines
                .get_ref()
                .peer_addr()?
                .as_pathname()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            tx,
        );

        let rx = Box::pin(async_stream::stream! {
            while let Some(item) = rx.recv().await {
                yield item;
            }
        });

        Ok(Guest { lines, rx })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn nothing() {
        tokio::spawn(async {
            let _ = super::UnixSocket::get_instance()
                .lock()
                .await
                .run("/tmp/aaa.sock".to_string())
                .await;

            let _ = super::UnixSocket::get_instance()
                .lock()
                .await
                .send("/tmp/bbb.sock", "hello")
                .await;
        });
    }
}
