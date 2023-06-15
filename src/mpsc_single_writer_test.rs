use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::fs::File;
use rand::{Rng, SeedableRng};
use tempfile::tempdir;
use std::any::type_name;
use std::ops::Range;
use std::time::Instant;

use crate::load_test::LoadTest;
use crate::test_result::TestResult;

pub struct MpscSingleWriterTest;

#[async_trait]
impl LoadTest for MpscSingleWriterTest {
    fn get_name(&self) -> &str {
        type_name::<Self>()
    }

    async fn run(&self, num_writes: usize, size_range: Range<usize>) -> TestResult {
        let mut results = Vec::new();
        let mut total_bytes: u64 = 0;

        // Create a temporary directory for the test
        let dir = tempdir().expect("Failed to create temp directory");
        println!("Using TempDir: {:?}", dir);
        let file_path = dir.path().join("test_log");

        // Create a channel for sending messages to the file writer task
        let (sender, mut receiver) = mpsc::channel::<(Vec<u8>, Instant)>(num_writes);

        // Spawn the file writer task
        let writer_handle = tokio::spawn(async move {
            let mut file = File::create(&file_path).await.expect("Failed to create file");
            while let Some((message, duration_start)) = receiver.recv().await {
                file.write_all(&message).await.expect("Failed to write to file");
                let duration = Instant::now().duration_since(duration_start);

                // Add the result to the results vector
                results.push(duration.as_nanos() as f64);
            }
            results
        });

        // Spawn the producer tasks
        for _ in 0..num_writes {
            let mut rng = rand::rngs::StdRng::from_entropy();
            let msg_size = rng.gen_range(size_range.clone());
            total_bytes += msg_size as u64;
            let message = vec![0u8; msg_size];
            let sender_clone = sender.clone();

            tokio::spawn(async move {
                let start = Instant::now();
                sender_clone.send((message, start)).await.expect("Failed to send message");
            });
        }
        // Signal no more messages will be sent
        drop(sender);

        // Wait for the writer task to finish
        let mut results = writer_handle.await.expect("Writer task panicked");

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