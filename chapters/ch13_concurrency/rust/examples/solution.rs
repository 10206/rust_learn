//! Ch13 정답: 동시성 (std 만 사용)
//!
//! 실행: `cargo run -q -p ch13_concurrency --example solution`
//! 테스트: `cargo test -p ch13_concurrency --example solution`

use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;

pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    (2..).take_while(|i| i * i <= n).all(|i| !n.is_multiple_of(i))
}

/// `[lo, hi)` 구간의 소수 개수.
pub fn count_primes(lo: u64, hi: u64) -> usize {
    (lo..hi).filter(|&n| is_prime(n)).count()
}

/// 입력마다 스레드를 하나씩 띄워 제곱을 계산하고, **입력 순서대로** 결과를 돌려줍니다.
pub fn parallel_squares(inputs: &[u64]) -> Vec<u64> {
    let handles: Vec<thread::JoinHandle<u64>> = inputs
        .iter()
        .map(|&x| thread::spawn(move || x * x)) // `move`: x 를 스레드로 "복사해서 넘김" (u64 는 Copy)
        .collect();
    // join() 은 스레드의 반환값을 돌려줍니다. 스레드가 panic 했다면 Err.
    handles.into_iter().map(|h| h.join().expect("worker panicked")).collect()
}

/// 데이터를 `chunks` 조각으로 나눠 **빌려서**(복사 없이) 합을 구합니다. 결과는 조각 순서.
pub fn scoped_chunk_sums(data: &[u64], chunks: usize) -> Vec<u64> {
    if data.is_empty() || chunks == 0 {
        return Vec::new();
    }
    let size = data.len().div_ceil(chunks);
    // thread::scope: 스코프가 끝나기 전에 모든 스레드가 join 되므로 지역 변수를 안전하게 빌릴 수 있습니다.
    thread::scope(|s| {
        let handles: Vec<_> = data
            .chunks(size)
            .map(|chunk| s.spawn(move || chunk.iter().sum::<u64>()))
            .collect();
        handles.into_iter().map(|h| h.join().expect("worker panicked")).collect()
    })
}

/// 여러 스레드가 하나의 카운터를 `per_thread` 번씩 올립니다.
pub fn counter_with_mutex(threads: usize, per_thread: usize) -> usize {
    // Arc: 여러 스레드가 소유하는 포인터 (Rc 의 atomic 버전), Mutex: 한 번에 한 스레드만 접근
    let counter = Arc::new(Mutex::new(0usize));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let counter = Arc::clone(&counter); // 스레드마다 자기 Arc 를 하나씩
            thread::spawn(move || {
                for _ in 0..per_thread {
                    // lock() 이 돌려주는 가드가 스코프(여기선 이 문장)를 벗어나면 자동으로 unlock
                    *counter.lock().expect("mutex poisoned") += 1;
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("worker panicked");
    }
    *counter.lock().expect("mutex poisoned")
}

/// 생산자 스레드 `producers` 개가 채널로 메시지를 `per` 개씩 보내고, 받은 메시지를 정렬해 돌려줍니다.
pub fn collect_messages(producers: usize, per: usize) -> Vec<String> {
    let (tx, rx) = mpsc::channel();
    let handles: Vec<_> = (0..producers)
        .map(|p| {
            let tx = tx.clone(); // 송신자는 복제 가능 (multi-producer)
            thread::spawn(move || {
                for m in 0..per {
                    tx.send(format!("producer {p} / msg {m}")).expect("receiver alive");
                }
            })
        })
        .collect();
    // 원본 tx 도 drop 해야 "모든 송신자가 사라졌다" 가 되어 아래 rx 루프가 끝납니다.
    drop(tx);
    let mut out: Vec<String> = rx.iter().collect();
    for h in handles {
        h.join().expect("worker panicked");
    }
    out.sort(); // 도착 순서는 비결정적이므로 정렬
    out
}

/// 쓰기 스레드 4개가 `i * 10` 을 push 한 뒤, 읽기 스레드 3개가 동시에 합계를 구합니다.
/// (정렬된 데이터, 각 읽기 스레드가 본 합계들)
pub fn rwlock_demo() -> (Vec<u32>, Vec<u32>) {
    let data = Arc::new(RwLock::new(Vec::<u32>::new()));

    let writers: Vec<_> = (0..4u32)
        .map(|i| {
            let data = Arc::clone(&data);
            thread::spawn(move || data.write().expect("poisoned").push(i * 10)) // write(): 배타 락
        })
        .collect();
    for w in writers {
        w.join().expect("writer panicked");
    }

    let readers: Vec<_> = (0..3)
        .map(|_| {
            let data = Arc::clone(&data);
            // read(): 공유 락. 여러 스레드가 동시에 읽을 수 있습니다.
            thread::spawn(move || data.read().expect("poisoned").iter().sum::<u32>())
        })
        .collect();
    let sums: Vec<u32> = readers.into_iter().map(|r| r.join().expect("reader panicked")).collect();

    let mut written = data.read().expect("poisoned").clone();
    written.sort_unstable();
    (written, sums)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Job {
    pub id: usize,
    pub lo: u64,
    pub hi: u64,
}

/// 워커 `workers` 개가 작업 채널에서 Job 을 꺼내 소수를 세고, 결과 채널로 `(job.id, 개수)` 를 보냅니다.
/// 결과는 job id 순으로 정렬해 돌려줍니다.
pub fn worker_pool(jobs: Vec<Job>, workers: usize) -> Vec<(usize, usize)> {
    let (job_tx, job_rx) = mpsc::channel::<Job>();
    let (res_tx, res_rx) = mpsc::channel::<(usize, usize)>();
    // Receiver 는 복제할 수 없으므로 Arc<Mutex<..>> 로 워커들이 나눠 씁니다. (mpsc = multi-producer, single-consumer)
    let job_rx = Arc::new(Mutex::new(job_rx));

    let handles: Vec<_> = (0..workers)
        .map(|_| {
            let job_rx = Arc::clone(&job_rx);
            let res_tx = res_tx.clone();
            thread::spawn(move || {
                loop {
                    // 이 문장이 끝나면 락 가드가 drop 되어 풀립니다. (`while let Ok(j) = rx.lock()...recv()` 로 쓰면
                    // 일하는 동안에도 락을 쥐게 되어 사실상 직렬 실행이 됩니다!)
                    let next = job_rx.lock().expect("poisoned").recv();
                    match next {
                        Ok(job) => res_tx.send((job.id, count_primes(job.lo, job.hi))).expect("receiver alive"),
                        Err(_) => break, // 송신자가 모두 사라지고 큐가 비면 Err -> 종료
                    }
                }
            })
        })
        .collect();
    drop(res_tx); // 워커들이 가진 복제본만 남기고 원본은 닫기

    for job in jobs {
        job_tx.send(job).expect("workers alive");
    }
    drop(job_tx); // 더 보낼 작업이 없음을 워커에게 알림

    let mut results: Vec<(usize, usize)> = res_rx.iter().collect();
    for h in handles {
        h.join().expect("worker panicked");
    }
    results.sort_unstable();
    results
}

fn main() {
    println!("== 1. spawn + join ==");
    println!("squares: {:?}", parallel_squares(&[1, 2, 3, 4, 5, 6, 7, 8]));

    println!("== 2. scoped threads ==");
    let data: Vec<u64> = (1..=100).collect();
    let sums = scoped_chunk_sums(&data, 4);
    println!("chunk sums: {sums:?}");
    println!("total: {}", sums.iter().sum::<u64>());

    println!("== 3. Arc<Mutex> ==");
    println!("counter: {} (8 threads x 1000)", counter_with_mutex(8, 1000));

    println!("== 4. channel ==");
    for m in collect_messages(3, 2) {
        println!("  {m}");
    }

    println!("== 5. RwLock ==");
    let (written, seen) = rwlock_demo();
    println!("written: {written:?}");
    println!("readers saw: {seen:?}");

    println!("== 6. worker pool ==");
    let jobs: Vec<Job> = (0..10)
        .map(|i| Job { id: i, lo: i as u64 * 10_000, hi: (i as u64 + 1) * 10_000 })
        .collect();
    let results = worker_pool(jobs.clone(), 4);
    for &(id, n) in &results {
        let job = jobs[id];
        println!("  job {id}: [{}, {}) -> {n} primes", job.lo, job.hi);
    }
    println!("total primes below 100000: {}", results.iter().map(|&(_, n)| n).sum::<usize>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primes() {
        assert!(is_prime(2) && is_prime(97) && !is_prime(1) && !is_prime(91));
        assert_eq!(count_primes(0, 100), 25);
        assert_eq!(count_primes(10, 20), 4); // 11 13 17 19
    }

    #[test]
    fn squares_keep_input_order() {
        assert_eq!(parallel_squares(&[3, 1, 2]), vec![9, 1, 4]);
        assert!(parallel_squares(&[]).is_empty());
    }

    #[test]
    fn scoped_chunks() {
        let data: Vec<u64> = (1..=10).collect();
        assert_eq!(scoped_chunk_sums(&data, 2), vec![15, 40]);
        assert_eq!(scoped_chunk_sums(&data, 3).iter().sum::<u64>(), 55);
        assert_eq!(scoped_chunk_sums(&data, 3).len(), 3); // 4+4+2 개
        assert!(scoped_chunk_sums(&[], 4).is_empty());
    }

    #[test]
    fn mutex_counter_never_loses_updates() {
        assert_eq!(counter_with_mutex(8, 1000), 8000);
        assert_eq!(counter_with_mutex(1, 5), 5);
    }

    #[test]
    fn channel_collects_everything() {
        let msgs = collect_messages(3, 2);
        assert_eq!(msgs.len(), 6);
        assert_eq!(msgs[0], "producer 0 / msg 0");
        assert_eq!(msgs[5], "producer 2 / msg 1");
    }

    #[test]
    fn rwlock_readers_agree() {
        let (written, seen) = rwlock_demo();
        assert_eq!(written, vec![0, 10, 20, 30]);
        assert_eq!(seen, vec![60, 60, 60]);
    }

    #[test]
    fn pool_matches_sequential() {
        let jobs: Vec<Job> = (0..6).map(|i| Job { id: i, lo: i as u64 * 500, hi: (i as u64 + 1) * 500 }).collect();
        let expected: Vec<(usize, usize)> = jobs.iter().map(|j| (j.id, count_primes(j.lo, j.hi))).collect();
        assert_eq!(worker_pool(jobs.clone(), 3), expected);
        assert_eq!(worker_pool(jobs, 1), expected); // 워커 수와 무관
        assert!(worker_pool(Vec::new(), 2).is_empty());
    }

    #[test]
    fn arc_counts_owners() {
        let a = Arc::new(5);
        let b = Arc::clone(&a);
        let h = thread::spawn(move || *b + 1);
        assert_eq!(h.join().unwrap(), 6);
        assert_eq!(Arc::strong_count(&a), 1); // 스레드가 끝나면서 b 가 drop 됨
    }
}
