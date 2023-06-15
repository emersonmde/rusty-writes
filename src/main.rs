
use arc_mutex_test::ArcMutexTest;
use flume_mpmc_test::FlumeMpmcTest;
use flume_mpsc_buffered_writer_test::FlumeMpscBufferedWriterTest;
use flume_mpsc_callback_test::FlumeMpscCallbackTest;
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
mod flume_mpsc_buffered_writer_test;
mod flume_mpsc_callback_test;


#[tokio::main]
async fn main() {
    let tests: Vec<Box<dyn LoadTest>> = vec![
        Box::new(SyncIoTest),
        Box::new(ArcMutexTest),
        Box::new(MpscSingleWriterTest),
        Box::new(FlumeMpmcTest),
        Box::new(FlumeMpscBufferedWriterTest),
        Box::new(FlumeMpscCallbackTest),
    ];

    let runner = TestRunner::new();
    runner.run_tests(tests).await;
}
