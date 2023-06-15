use async_trait::async_trait;
use tokio::fs::{File, self};
use std::os::unix::prelude::FileExt;
use std::sync::Arc;
use flume;
use rand::{Rng, SeedableRng};
use tempfile::tempdir;
use std::any::type_name;
use std::ops::Range;
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::load_test::LoadTest;
use crate::test_result::TestResult;

pub struct FlumeMpmcTest;

#[async_trait]
impl LoadTest for FlumeMpmcTest {
    fn get_name(&self) -> &str {
        type_name::<Self>()
    }

    async fn run(&self, num_writes: usize, size_range: Range<usize>) -> TestResult {
        let mut total_bytes: u64 = 0;

        // Create a temporary directory for the test
        let dir = tempdir().expect("Failed to create temp directory");
        let file_path = dir.path().join("test_log");

        // Create a channel for sending messages to the file writer tasks
        let (sender, receiver) = flume::unbounded::<Vec<u8>>();

        // let file = std::fs::OpenOptions::new()
        //     .create(true)
        //     .write(true)
        //     .open(&file_path)
        //     .expect("Failed to create file");

        let offset = Arc::new(AtomicU64::new(0));

        // Spawn the file writer tasks
        let mut writer_handles = Vec::new();
        for _ in 0..10 {
            let receiver = receiver.clone();
            let file_path = file_path.clone();
            // let file_clone = file.try_clone().expect("Failed to clone file");
            let offset = offset.clone();
            writer_handles.push(tokio::spawn(async move {
                let mut results = Vec::new();
                while let Ok(message) = receiver.recv_async().await {
                    let duration_start = Instant::now();
                    let start_pos = offset.fetch_add(message.len() as u64, Ordering::SeqCst);
                    let file_path = file_path.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        let file = std::fs::OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(false)
                            .open(&file_path)
                            .expect("Failed to create file");
                        file.write_at(&message, start_pos)
                    }).await.expect("Failed to write to file");
                    match result {
                        Ok(_) => (),
                        Err(e) => panic!("Failed to write to file: {}", e),
                    }
                    let duration = Instant::now().duration_since(duration_start);

                    // Add the result to the results vector
                    results.push(duration.as_nanos() as f64);
                }
                results
            }));
        }

        // Spawn the producer tasks
        for _ in 0..num_writes {
            let mut rng = rand::rngs::StdRng::from_entropy();
            let msg_size = rng.gen_range(size_range.clone());
            total_bytes += msg_size as u64;
            let message = vec![0u8; msg_size];
            let sender_clone = sender.clone();

            tokio::spawn(async move {
                sender_clone.send_async(message).await.expect("Failed to send message");
            });
        }

        // Signal no more messages will be sent
        drop(sender);

        // Wait for all writer tasks to finish
        let mut all_results = Vec::new();
        for handle in writer_handles {
            let mut results = handle.await.expect("Writer task panicked");
            all_results.append(&mut results);
        }

        let metadata = fs::metadata(file_path).await.expect("Failed to get file metadata");
        let file_size = metadata.len();
        assert_eq!(file_size, offset.load(Ordering::SeqCst));

        dir.close().expect("Failed to delete temp directory");

        // Calculate the mean, median, and p90
        all_results.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let sum: f64 = all_results.iter().sum();
        let mean = sum / num_writes as f64;
        let median = all_results[num_writes / 2];
        let p90 = all_results[(9 * num_writes) / 10];

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
