## Async Write Testing

I couldn't find much information about what the fastest way to write to a file is using Rust so I tried testing a few methods I've seen or variations I could think of. The context I was considering when looking for the fastest write method was a server that accepts as many incomming network connections as possible, receives a short message, and writes it directly to disk in a sequential file. This means there would be many threads all trying to write to the same file at the same time. 

Tests:
- Normal for loop using tokio's `File`/`write`
- Spawning tasks using an `Arc<Mutex<File>>` to synchronizing writing
- Spawning tasks writing to a MPSC channel with a single separate thread writing
- Spawning tasks writing to an unbounded flume channel with a single separate write thread using BufWriter
- Spawning tasks writing to an unbounded flume channel with 10 write threads consuming
- Spawning tasks writing to an unbounded flume channel with a rendezvous channel

Each of these tests run some number of writes that contain a random payload between a min and max size. 

### Results
I wouldn't rely too much on these numbers other than to say any synchronization methods seem to have a similar impact, except for buffered writing in a single consumer thread.

```
===== Running test rusty_writes::flume_mpsc_buffered_writer_test::FlumeMpscBufferedWriterTest (1000000 writes of size 300..2000) =====
  Mean:         626.31 ms (626311420.51 ns)
  Median:       789.22 ms (789216459.00 ns)
  P90:          869.30 ms (869299584.00 ns)
  Writes/s:     397396.37
  Total Bytes:  1149.17 MB
  Total Writes: 1000000
  Total Time:   2516.38 ms

Completed test for rusty_writes::flume_mpsc_buffered_writer_test::FlumeMpscBufferedWriterTest in 2.52 s (2516.39 ms)

===== Running test rusty_writes::sync_io_test::SyncIoTest (1000000 writes of size 300..2000) =====
  Mean:         0.01 ms (6843.42 ns)
  Median:       0.01 ms (6208.00 ns)
  P90:          0.01 ms (8750.00 ns)
  Writes/s:     140792.95
  Total Bytes:  1149.33 MB
  Total Writes: 1000000
  Total Time:   7102.63 ms

Completed test for rusty_writes::sync_io_test::SyncIoTest in 7.10 s (7102.64 ms)

===== Running test rusty_writes::arc_mutex_test::ArcMutexTest (1000000 writes of size 300..2000) =====
  Mean:         5179.73 ms (5179727105.90 ns)
  Median:       5224.68 ms (5224681959.00 ns)
  P90:          8757.26 ms (8757262000.00 ns)
  Writes/s:     96453.55
  Total Bytes:  1149.67 MB
  Total Writes: 1000000
  Total Time:   10367.68 ms

Completed test for rusty_writes::arc_mutex_test::ArcMutexTest in 10.37 s (10367.69 ms)

===== Running test rusty_writes::mpsc_single_writer_test::MpscSingleWriterTest (1000000 writes of size 300..2000) =====
  Mean:         4500.68 ms (4500678987.50 ns)
  Median:       4560.64 ms (4560635625.00 ns)
  P90:          7552.71 ms (7552711500.00 ns)
  Writes/s:     94584.88
  Total Bytes:  1150.04 MB
  Total Writes: 1000000
  Total Time:   10572.51 ms

Completed test for rusty_writes::mpsc_single_writer_test::MpscSingleWriterTest in 10.57 s (10572.52 ms)

File size 1149162905 matches 1149162905 bytes written
===== Running test rusty_writes::flume_mpmc_test::FlumeMpmcTest (1000000 writes of size 300..2000) =====
  Mean:         0.12 ms (120934.44 ns)
  Median:       0.11 ms (111375.00 ns)
  P90:          0.19 ms (187750.00 ns)
  Writes/s:     81573.03
  Total Bytes:  1149.16 MB
  Total Writes: 1000000
  Total Time:   12258.95 ms

Completed test for rusty_writes::flume_mpmc_test::FlumeMpmcTest in 12.26 s (12258.96 ms)

```