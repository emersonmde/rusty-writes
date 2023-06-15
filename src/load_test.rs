use std::ops::Range;

use crate::test_result::TestResult;

use async_trait::async_trait;

#[async_trait]
pub trait LoadTest {
    fn get_name(&self) -> &str;
    async fn run(&self, num_writes: usize, size_range: Range<usize>) -> TestResult;
}