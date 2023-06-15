use tokio::time::Instant;

use crate::{load_test::LoadTest, test_result::TestResult};

pub struct TestRunResult {
    pub num_writes: usize,
    pub run_duration_ns: f64,
    pub result: Vec<TestResult>,
}

pub struct TestRunner {
    pub test_runs: Vec<TestRunResult>,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            test_runs: Vec::new(),
        }
    }

    pub async fn run_tests(&self, tests: Vec<Box<dyn LoadTest>>) {
        for test in tests {
            println!("===== Running test: {} =====", test.get_name());
            let start = Instant::now();

            let run_start = Instant::now();
            let num_writes = 1_000_000;
            let result = test.run(num_writes, 300..2_000).await;
            let run_duration_ns = Instant::now().duration_since(run_start).as_nanos() as f64;
            Self::print_results(result, num_writes, run_duration_ns);

            let total_duration = Instant::now().duration_since(start).as_nanos() as f64;
            let total_duration_ms = total_duration / 1_000_000.0;
            let total_duration_s = total_duration_ms / 1000.0;

            println!("Completed test for {} in {:.2} s ({:.2} ms)", test.get_name(), total_duration_s, total_duration_ms);
            println!("");
        }
    }

    fn print_results(result: TestResult, num_writes: usize, run_duration_ns: f64) {
        let run_duration_ms = run_duration_ns / 1_000_000.0;
        let writes_per_s = num_writes as f64 / (run_duration_ms / 1000.0);
        println!("  Mean:         {:.2} ms ({:.2} ns)", result.mean / 1_000_000.0, result.mean);
        println!("  Median:       {:.2} ms ({:.2} ns)", result.median / 1_000_000.0, result.median);
        println!("  P90:          {:.2} ms ({:.2} ns)", result.p90 / 1_000_000.0, result.p90);
        println!("  Writes/s:     {:.2}", writes_per_s);
        println!("  Total Bytes:  {:.2} MB", result.total_bytes as f64 / 1_000_000.0);
        println!("  Total Writes: {:.2}", result.num_writes);
        println!("  Total Time:   {:.2} ms", run_duration_ms);
        println!("");
    }
}
