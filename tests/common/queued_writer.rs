use std::collections::VecDeque;
use std::task::Poll;

use tokio::io::AsyncWrite;

#[derive(Debug)]
pub(crate) struct QueuedWriter<'a> {
    queue: VecDeque<&'a [u8]>,
}

impl<'a> QueuedWriter<'a> {
    pub(crate) fn new() -> QueuedWriter<'a> {
        QueuedWriter {
            queue: VecDeque::new(),
        }
    }
}

impl<'a> AsyncWrite for QueuedWriter<'a> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        self.queue.push_back(buf);
        Poll::<_>::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::<_>::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::<_>::Ready(Ok(()))
    }
}
