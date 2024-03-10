use std::future::Future;
use tokio::sync::Mutex;
use log::{error, info};
use serde_json::json; // For structured logging
use thiserror::Error;
use anyhow::Result; // Use anyhow for more flexible error handling
use std::time::Instant;

#[derive(Error, Debug)]
pub enum GeneralError {
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

async fn execute_async_operation<F, Fut, T>(
    future: F,
    operation_desc: &str,
) -> Result<T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let start_time = Instant::now();
    match future().await {
        Ok(result) => {
            let duration = start_time.elapsed();
            // Structured logging with operation description and duration
            info!("{}", json!({
                "message": "Successfully completed operation",
                "operation": operation_desc,
                "duration_ms": duration.as_millis(),
            }));
            Ok(result)
        }
        Err(e) => {
            let duration = start_time.elapsed();
            // Structured error logging with operation description, error, and duration
            error!("{}", json!({
                "message": "Error during operation",
                "operation": operation_desc,
                "error": e.to_string(),
                "duration_ms": duration.as_millis(),
            }));
            Err(e)
        }
    }
}

// Example usage demonstrating the adaptability to various async operations
async fn example_usage() -> Result<()> {
    let operation_result = execute_async_operation(
        || async {
            // Your async operation here. This is just a placeholder.
            // For real use, insert asynchronous logic such as disk I/O, API calls, etc.
            Ok("Operation Result")
        },
        "example operation",
    )
    .await;

    match &operation_result {
        Ok(result) => info!("Operation succeeded with result: {:?}", result),
        Err(e) => error!("Operation failed: {}", e),
    }

    operation_result.map(|_| ()) // Convert result to match function signature if necessary
}
