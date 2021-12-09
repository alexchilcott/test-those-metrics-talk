use actix_rt::time::{sleep, timeout};
use std::cmp::min;
use std::fmt::Debug;
use std::future::Future;
use std::time::{Duration, Instant};

#[derive(thiserror::Error, Debug)]
pub enum RetryTimeoutError<E> {
    #[error("Timed out waiting for a successful attempt")]
    NeverSucceeded(#[from] E),
    #[error("Timed out without ever completing an attempt")]
    NeverCompleted,
}

pub async fn retry_until_ok<F, Fut, T, E>(
    f: F,
    total_timeout: Duration,
    single_call_timeout: Duration,
    wait_time: Duration,
) -> Result<T, RetryTimeoutError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let timeout_time = Instant::now() + total_timeout;
    let mut last_attempt_error: Option<E> = None;
    loop {
        let remaining_time = timeout_time.saturating_duration_since(Instant::now());
        let timeout_for_next_call = min(remaining_time, single_call_timeout);
        let result = timeout(timeout_for_next_call, f()).await;

        match result {
            Ok(Ok(success)) => return Ok(success),
            Ok(Err(attempt_error)) => last_attempt_error = Some(attempt_error),
            Err(_) => (),
        }

        let remaining_time = timeout_time.saturating_duration_since(Instant::now());
        let was_last_attempt = wait_time > remaining_time;
        if was_last_attempt {
            match last_attempt_error {
                Some(e) => {
                    return Err(RetryTimeoutError::NeverSucceeded(e));
                }
                None => return Err(RetryTimeoutError::NeverCompleted),
            }
        }

        sleep(wait_time).await
    }
}
