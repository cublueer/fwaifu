/// Run a function repeatedly with a fixed interval.
/// Clears the terminal before each iteration.
pub async fn watch_loop<F, Fut>(interval_secs: u64, mut func: F)
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    loop {
        // Clear terminal: ANSI escape to clear screen and move cursor to home
        print!("\x1B[2J\x1B[H");

        // Run the function
        func().await;

        // Sleep for the configured interval
        tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
    }
}
