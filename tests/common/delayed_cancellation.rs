use std::time::Duration;

use tokio_util::sync::CancellationToken;

pub trait DelayedCancellation {
    fn that_cancels_after(duration: Duration) -> CancellationToken;
}

impl DelayedCancellation for CancellationToken {
    fn that_cancels_after(duration: Duration) -> CancellationToken {
        let token = CancellationToken::new();
        let token_clone = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            token_clone.cancel();
        });
        token
    }
}
