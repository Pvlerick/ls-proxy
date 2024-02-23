use std::collections::VecDeque;
use std::sync::Arc;
use std::task::Poll;

use tokio::io::AsyncRead;

#[derive(Debug)]
pub(crate) struct QueuedReader<'a> {
    queue: Arc<VecDeque<&'a [u8]>>,
}

impl<'a> QueuedReader<'a> {
    pub(crate) fn new() -> QueuedReader<'a> {
        QueuedReader {
            queue: Arc::new(VecDeque::new()),
        }
    }

    pub(crate) fn write(&mut self, payload: &'a [u8]) {
        self.queue.push_back(payload);
    }
}

impl<'a> Clone for QueuedReader<'a> {
    fn clone(&self) -> Self {
        QueuedReader {
            queue: self.queue.clone(),
        }
    }
}

impl<'a> AsyncRead for QueuedReader<'a> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.queue.pop_front() {
            Some(slice) => {
                buf.put_slice(slice);
                return Poll::<_>::Ready(Ok(()));
            }
            None => return Poll::<_>::Pending,
        }
    }
}
