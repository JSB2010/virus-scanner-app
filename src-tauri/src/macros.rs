#[macro_export]
macro_rules! retry_with_backoff {
    ($error_handler:expr, $operation:expr, $body:expr) => {{
        let mut attempts = 0;
        let max_attempts = 3;
        let mut last_error = None;

        loop {
            match $body.await {
                Ok(result) => {
                    // Report success if we had to retry
                    if attempts > 0 {
                        $error_handler.lock().await.report_success($operation).await;
                    }
                    break Ok(result);
                }
                Err(e) => {
                    attempts += 1;
                    last_error = Some(e.to_string());
                    
                    if attempts >= max_attempts {
                        break Err(last_error.unwrap());
                    }
                    
                    // Exponential backoff
                    let delay = std::time::Duration::from_secs(2u64.pow(attempts));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }};
}
