use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::fs::File;
use rand::{Rng, rngs::StdRng};
use rand::SeedableRng;
use tempfile::tempdir;
use std::any::type_name;
use std::ops::Range;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tokio::sync::Mutex as AsyncMutex;
use std::sync::Arc;

use crate::load_test::LoadTest;
use crate::test_result::TestResult;

pub struct ArcMutexTest;

#[async_trait]
impl LoadTest for ArcMutexTest {
    fn get_name(&self) -> &str {
        type_name::<Self>()
    }

    async fn run(&self, num_writes: usize, size_range: Range<usize>) -> TestResult {
        let mut results = Vec::new();
        let mut tasks = Vec::new();

        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_log");
        let file = File::create(&file_path).await.expect("Failed to create file");
        let file = Arc::new(AsyncMutex::new(file));
        let total_bytes = Arc::new(AtomicU64::new(0));

        for _ in 0..num_writes {
            // We create a new task for each write
            let file = Arc::clone(&file);
            let total_bytes = Arc::clone(&total_bytes);
            let size_range = size_range.clone();
            let task = tokio::spawn(async move {
                let mut rng = StdRng::from_entropy();

                // Generate a random message
                let msg_size = rng.gen_range(size_range);
                total_bytes.fetch_add(msg_size as u64, Ordering::SeqCst);
                let message = vec![0u8; msg_size];

                // Write the message to a file using async IO
                let start = Instant::now();
                let mut file = file.lock().await;

                file.write_all(&message).await.expect("Failed to write to file");
                let duration = Instant::now().duration_since(start);

                duration.as_nanos() as f64
            });

            tasks.push(task);
        }

        // We then await all the tasks and collect their results
        for task in tasks {
            let result = task.await.expect("Task panicked");
            results.push(result);
        }

        // Calculate the mean, median, and p90
        results.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let sum: f64 = results.iter().sum();
        let mean = sum / num_writes as f64;
        let median = results[num_writes / 2];
        let p90 = results[(9 * num_writes) / 10];

        // Close the temporary directory
        dir.close().expect("Failed to delete temp directory");

        TestResult {
            mean,
            median,
            p90,
            num_writes: num_writes as u64,
            total_bytes: total_bytes.load(Ordering::SeqCst),
        }
    }
}
