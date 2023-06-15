
use arc_mutex_test::ArcMutexTest;
use flume_mpmc_test::FlumeMpmcTest;
use mpsc_single_writer_test::MpscSingleWriterTest;
use sync_io_test::SyncIoTest;
use load_test::LoadTest;
use test_runner::TestRunner;

mod load_test;
mod test_result;
mod test_runner;

// tests
mod sync_io_test;
mod arc_mutex_test;
mod mpsc_single_writer_test;
mod flume_mpmc_test;


#[tokio::main]
async fn main() {
    let tests: Vec<Box<dyn LoadTest>> = vec![
        Box::new(SyncIoTest),
        Box::new(ArcMutexTest),
        Box::new(MpscSingleWriterTest),
        Box::new(FlumeMpmcTest),
    ];

    let runner = TestRunner::new();
    runner.run_tests(tests).await;
}
