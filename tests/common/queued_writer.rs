use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Poll;

use tokio::io::AsyncWrite;

#[derive(Debug)]
pub(crate) struct QueuedWriter<'a> {
    queue: Arc<Mutex<VecDeque<&'a [u8]>>>,
}

impl<'a> QueuedWriter<'a> {
    pub(crate) fn new() -> QueuedWriter<'a> {
        QueuedWriter {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub(crate) fn read(&mut self) -> Option<&[u8]> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }
}

impl<'a> Clone for QueuedWriter<'a> {
    fn clone(&self) -> Self {
        QueuedWriter {
            queue: self.queue.clone(),
        }
    }
}

impl<'a> AsyncWrite for QueuedWriter<'a> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let mut buf_copy = Vec::with_capacity(buf.len());
        buf_copy.clone_from_slice(buf);
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(&buf_copy);
        Poll::<_>::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::<_>::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::<_>::Ready(Ok(()))
    }
}
