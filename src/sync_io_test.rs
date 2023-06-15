use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::fs::File;
use rand::{Rng, SeedableRng};
use tempfile::tempdir;
use std::any::type_name;
use std::ops::Range;
use std::time::Instant;

use crate::load_test::LoadTest;
use crate::test_result::TestResult;

pub struct SyncIoTest;

#[async_trait]
impl LoadTest for SyncIoTest {
    fn get_name(&self) -> &str {
        type_name::<Self>()
    }

    async fn run(&self, num_writes: usize, size_range: Range<usize>) -> TestResult {
        let mut results = Vec::new();
        let mut rng = rand::rngs::StdRng::from_entropy();
        let mut total_bytes: u64 = 0;

        // Create a temporary directory for the test
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_log");
        for _ in 0..num_writes {

            // Generate a random message
            let msg_size = rng.gen_range(size_range.clone());
            total_bytes += msg_size as u64;
            let message = vec![0u8; msg_size];

            // Write the message to a file using async IO
            let start = Instant::now();
            let mut file = File::create(&file_path).await.expect("Failed to create file");

            file.write_all(&message).await.expect("Failed to write to file");
            let duration = Instant::now().duration_since(start);

            // Add the result to the results vector
            results.push(duration.as_nanos() as f64);
        }
        dir.close().expect("Failed to delete temp directory");

        // Calculate the mean, median, and p90
        results.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let sum: f64 = results.iter().sum();
        let mean = sum / num_writes as f64;
        let median = results[num_writes / 2];
        let p90 = results[(9 * num_writes) / 10];

        // Cast the result of the `run` method to a `TestResult` object
        TestResult {
            mean,
            median,
            p90,
            num_writes: num_writes as u64,
            total_bytes,
        }
    }
}
