//! Ch13. 동시성 — 실습: 스레드 · Arc/Mutex · 채널 · 워커 풀
//!
//! 목표
//!   - `thread::spawn` + `move` 클로저 + `JoinHandle::join` 으로 값을 돌려받는다.
//!   - `thread::scope` 로 지역 데이터를 복사 없이 빌려서 병렬 처리한다.
//!   - `Arc<Mutex<T>>`, `RwLock`, `mpsc` 채널로 스레드 간 공유/통신을 한다.
//!   - 데이터 레이스가 "컴파일 에러"로 막히는 모습(Send/Sync)을 🔬 실험으로 직접 본다.
//!
//! 대응 파일: `../python/main.py`, `../cpp/main.cpp`
//!
//! 진행 방법
//!   1. TODO(1)부터 순서대로 `todo!()` 를 지우고 구현하세요. (표준 라이브러리만 사용)
//!   2. 🔬 실험 블록은 주석을 풀어서 컴파일러 에러를 읽어 보세요. (실험 후엔 다시 주석 처리!)
//!   3. 출력은 스레드 실행 순서와 무관하게 항상 같아야 합니다. (결과는 입력 순서/정렬로 모으기)
//!
//! 완료 기준
//!   - `cargo test -p ch13_concurrency` 가 모두 통과
//!   - `../../scripts/compare.sh ch13` 에서 Python / C++ / Rust 출력이 똑같음

// 다 풀고 나면 이 줄을 지워 보세요 (그러면 안 쓰는 코드에 대한 경고가 보입니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;

/// (이미 구현되어 있어요) 소수 판별.
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    (2..).take_while(|i| i * i <= n).all(|i| !n.is_multiple_of(i))
}

// ─────────────────────────────────────────────
// TODO(1): `[lo, hi)` 구간의 소수 개수를 세세요. (아직 스레드는 없어요. 워커 풀이 이 함수를 씁니다.)
//   🐍 Python: sum(1 for n in range(lo, hi) if is_prime(n))     (python/main.py: count_primes)
//   ⚙️ C++  : for 루프로 is_prime 카운트                          (cpp/main.cpp: count_primes)
//   🦀 Rust : `(lo..hi).filter(|&n| is_prime(n)).count()`
//   💡 힌트: 범위 `lo..hi` 는 그 자체로 이터레이터입니다.
// ─────────────────────────────────────────────
pub fn count_primes(lo: u64, hi: u64) -> usize {
    todo!("TODO(1): count_primes")
}

// ─────────────────────────────────────────────
// TODO(2): 입력마다 스레드를 하나씩 띄워 제곱을 계산하고, **입력 순서대로** 결과를 돌려주세요.
//   🐍 Python: threading.Thread(target=work, args=(i, x)).start() ... join()   (python/main.py: parallel_squares)
//   ⚙️ C++  : std::thread t([&]{ results[i] = x*x; }); ... t.join();            (cpp/main.cpp: parallel_squares)
//   🦀 Rust : `thread::spawn(move || x * x)` 는 `JoinHandle<u64>` 를 돌려주고, `handle.join()` 이 스레드의 반환값을 줍니다.
//   💡 힌트: 먼저 `Vec<JoinHandle<u64>>` 로 모두 spawn 한 뒤(그래야 동시에 돌아요), 그 다음 순서대로 join 하세요.
//           `join()` 은 `Result` 입니다. (스레드가 panic 했으면 Err) `.expect("worker panicked")`
//   ❓ 왜 `move`?: 스레드는 spawn 한 함수보다 오래 살 수 있어서, 클로저가 지역 변수를 빌려 가면 컴파일러가 거부합니다(E0373).
//           `move` 는 값을 클로저 안으로 "이동/복사" 시켜서 스레드가 자기 것을 갖게 합니다.
//
//   🔬 실험 1 (E0373): 아래 함수의 주석을 풀어서 `move` 가 없을 때의 에러를 읽어 보세요.
//   fn lab_forgot_move() {
//       let v = vec![1, 2, 3];
//       let h = thread::spawn(|| println!("{v:?}"));   // <- move 없음
//       h.join().unwrap();
//   }
//   기대 에러: error[E0373]: closure may outlive the current function, but it borrows `v`
//   ❓ 컴파일러의 `help:` 가 제안하는 수정은? (C++ 에서 `[&]` 로 캡처하고 detach 하면 컴파일되지만 UB 입니다.)
// ─────────────────────────────────────────────
pub fn parallel_squares(inputs: &[u64]) -> Vec<u64> {
    todo!("TODO(2): parallel_squares")
}

// ─────────────────────────────────────────────
// TODO(3): 데이터를 `chunks` 조각으로 나눠 조각마다 스레드로 합을 구하세요. (결과는 조각 순서)
//   조각 크기 = `data.len().div_ceil(chunks)`,  `data.chunks(size)` 로 나눕니다. (data 가 비었거나 chunks == 0 이면 빈 Vec)
//   🐍 Python: parts = [data[i:i+size] ...]; Thread(target=work) ...    (python/main.py: scoped_chunk_sums)
//   ⚙️ C++  : std::jthread + 참조 캡처 (join 책임은 프로그래머)           (cpp/main.cpp: scoped_chunk_sums)
//   🦀 Rust : `thread::scope(|s| { s.spawn(move || chunk.iter().sum::<u64>()) ... })`
//   💡 힌트: 이번엔 `Arc` 가 필요 없어요! 스코프 스레드는 스코프가 끝나기 전에 반드시 join 되므로 `data` 를 빌릴 수 있습니다.
//           `s.spawn(..)` 의 반환값도 `join()` 하면 값을 줍니다. 스코프 클로저의 마지막 표현식이 `thread::scope` 의 반환값입니다.
//   ❓ 왜?: 보통 `thread::spawn` 은 `'static` 만 받습니다(스레드가 얼마나 오래 살지 모르니까). scope 는 "여기서 끝난다" 를 보장해서 그 제약을 풉니다.
//
//   🔬 실험 2 (E0499): 두 스코프 스레드가 같은 변수를 동시에 고치려 하면?
//   fn lab_shared_mut_in_scope() {
//       let mut count = 0;
//       thread::scope(|s| {
//           s.spawn(|| count += 1);
//           s.spawn(|| count += 1);
//       });
//       println!("{count}");
//   }
//   기대 에러: error[E0499]: cannot borrow `count` as mutable more than once at a time
//   ❓ C++ 에서는 같은 코드가 컴파일되고 데이터 레이스(UB)가 됩니다. Rust 의 "쓰기 하나 XOR 읽기 여럿" 규칙(Ch04)이 스레드에도 그대로 적용돼요.
// ─────────────────────────────────────────────
pub fn scoped_chunk_sums(data: &[u64], chunks: usize) -> Vec<u64> {
    todo!("TODO(3): scoped_chunk_sums")
}

// ─────────────────────────────────────────────
// TODO(4): `threads` 개의 스레드가 공유 카운터를 `per_thread` 번씩 1 올리고, 최종값을 돌려주세요.
//   🐍 Python: with lock: counter += 1                              (python/main.py: counter_with_mutex)
//   ⚙️ C++  : std::lock_guard lk(m); ++counter;                      (cpp/main.cpp: counter_with_mutex)
//   🦀 Rust : `Arc<Mutex<usize>>` — 스레드마다 `Arc::clone(&counter)` 를 만들어 `move` 로 넘기고,
//           `*counter.lock().unwrap() += 1;`
//   💡 힌트: 모든 스레드를 spawn 한 뒤 전부 join 하고, 마지막에 `*counter.lock().unwrap()` 로 읽으세요.
//   ❓ 왜 Arc 와 Mutex 둘 다?: Arc = "여러 스레드가 함께 소유"(수명 문제 해결), Mutex = "한 번에 한 스레드만 접근"(데이터 레이스 해결).
//           C++ 의 `shared_ptr<mutex>` + 별도 변수와 달리, Rust 의 Mutex 는 **데이터를 안에 품고** 있어서 락 없이는 값에 접근할 방법이 없습니다.
//
//   🔬 실험 3 (E0277): Rc 를 스레드로 보내 보세요. (Ch11 의 Rc 는 카운트가 atomic 이 아니라서 위험합니다)
//   fn lab_rc_in_thread() {
//       use std::rc::Rc;
//       let rc = Rc::new(5);
//       let h = thread::spawn(move || println!("{rc}"));
//       h.join().unwrap();
//   }
//   기대 에러: error[E0277]: `Rc<i32>` cannot be sent between threads safely  (the trait `Send` is not implemented for `Rc<i32>`)
//   ❓ `Rc` 를 `Arc` 로 바꾸면 통과합니다. 왜 `Rc` 만 막힐까요? (README 의 Send/Sync 절)
//
//   🔬 실험 4 (E0382): Mutex 를 Arc 없이 두 스레드에 move 하면?
//   fn lab_mutex_moved_twice() {
//       let m = Mutex::new(0);
//       let a = thread::spawn(move || *m.lock().unwrap() += 1);
//       let b = thread::spawn(move || *m.lock().unwrap() += 1);
//       a.join().unwrap(); b.join().unwrap();
//   }
//   기대 에러: error[E0382]: use of moved value: `m`   → 해결책이 곧 `Arc<Mutex<_>>` 입니다.
//
//   🔬 실험 5 (컴파일은 되지만 틀린 결과 — `move` 의 함정): 숫자 카운터에 `move` 를 쓰면 복사본이 올라갑니다.
//   fn lab_move_copies() {
//       let mut count = 0;
//       let h = thread::spawn(move || { count += 1; });   // i32 는 Copy: 스레드는 복사본을 고침
//       h.join().unwrap();
//       println!("count = {count}");                      // 0 !  (경고 `unused_assignments` 가 힌트)
//   }
//   ❓ 공유하려면 `Arc<Mutex<i32>>` 또는 `AtomicUsize` 여야 합니다. 이런 "조용한 버그"는 AI 코드에서도 자주 보입니다.
//
//   🔬 실험 6 (E0277 Sync): `Arc<RefCell<_>>` 는 `Arc` 로 감싸도 막힙니다.
//   fn lab_refcell_in_arc() {
//       use std::cell::RefCell;
//       let c = Arc::new(RefCell::new(0));
//       let c2 = Arc::clone(&c);
//       thread::spawn(move || { *c2.borrow_mut() += 1; }).join().unwrap();
//   }
//   기대 에러: error[E0277]: `RefCell<i32>` cannot be shared between threads safely (the trait `Sync` is not implemented)
//   ❓ `RefCell` → `Mutex` 로 바꾸면 해결. Ch11 의 "런타임 빌림 검사" 가 스레드 안전한 버전이 Mutex/RwLock 입니다.
// ─────────────────────────────────────────────
pub fn counter_with_mutex(threads: usize, per_thread: usize) -> usize {
    todo!("TODO(4): counter_with_mutex")
}

// ─────────────────────────────────────────────
// TODO(5): 생산자 스레드 `producers` 개가 채널로 `format!("producer {p} / msg {m}")` 를 `per` 개씩 보내고,
//   받은 메시지를 **정렬해서** 돌려주세요.
//   🐍 Python: queue.Queue().put(...)                                (python/main.py: collect_messages)
//   ⚙️ C++  : 직접 만든 Channel<T> (mutex + condition_variable)       (cpp/main.cpp: Channel, collect_messages)
//   🦀 Rust : `let (tx, rx) = mpsc::channel();`  송신자는 `tx.clone()` 으로 스레드마다 하나씩 `move`
//   💡 힌트: 원본 `tx` 를 `drop(tx)` 하지 않으면 `rx.iter()` 가 영원히 끝나지 않습니다. (모든 Sender 가 사라져야 채널이 닫혀요)
//           `rx.iter().collect::<Vec<String>>()` 로 한 번에 모을 수 있고, 마지막에 `sort()`.
//   ❓ 왜 정렬?: 메시지 도착 순서는 스레드 스케줄링에 따라 매번 달라집니다. 출력은 결정적이어야 하니까요.
// ─────────────────────────────────────────────
pub fn collect_messages(producers: usize, per: usize) -> Vec<String> {
    todo!("TODO(5): collect_messages")
}

// ─────────────────────────────────────────────
// TODO(6): `Arc<RwLock<Vec<u32>>>` — 쓰기 스레드 4개(i = 0..4)가 `i * 10` 을 push 하고(모두 join),
//   그 다음 읽기 스레드 3개가 동시에 합계를 구해 각자 돌려줍니다. 반환: (정렬된 데이터, 읽기 스레드들이 본 합계들)
//   기대: ([0, 10, 20, 30], [60, 60, 60])
//   🐍 Python: (stdlib 에 RWLock 이 없어 Lock 으로 대체)             (python/main.py: rwlock_demo)
//   ⚙️ C++  : std::shared_mutex + shared_lock / unique_lock          (cpp/main.cpp: rwlock_demo)
//   🦀 Rust : `data.write().unwrap().push(..)` (배타) / `data.read().unwrap().iter().sum::<u32>()` (공유)
//   💡 힌트: 읽기 락은 여러 스레드가 동시에 잡을 수 있고, 쓰기 락은 혼자만 잡습니다. (Ch04 의 `&T` vs `&mut T` 와 같은 규칙의 런타임 버전)
//   ❓ 언제 Mutex 대신 RwLock?: 읽기가 압도적으로 많고 읽기 작업이 충분히 길 때. 짧은 작업이면 Mutex 가 더 빠를 수 있어요.
// ─────────────────────────────────────────────
pub fn rwlock_demo() -> (Vec<u32>, Vec<u32>) {
    todo!("TODO(6): rwlock_demo")
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Job {
    pub id: usize,
    pub lo: u64,
    pub hi: u64,
}

// ─────────────────────────────────────────────
// TODO(7): 워커 풀. 워커 `workers` 개가 **작업 채널**에서 Job 을 꺼내 `count_primes(job.lo, job.hi)` 를 계산하고,
//   **결과 채널**로 `(job.id, 개수)` 를 보냅니다. 결과는 job id 순으로 정렬해 돌려주세요.
//   🐍 Python: Queue 두 개 + None 종료 신호                           (python/main.py: worker_pool)
//   ⚙️ C++  : Channel<Job>, Channel<result>, close()                  (cpp/main.cpp: worker_pool)
//   🦀 Rust : 절차
//     1) `let (job_tx, job_rx) = mpsc::channel::<Job>();`  `let (res_tx, res_rx) = mpsc::channel::<(usize, usize)>();`
//     2) `Receiver` 는 복제가 안 되므로 `let job_rx = Arc::new(Mutex::new(job_rx));` 로 워커들이 나눠 씁니다.
//     3) 워커: `loop { let next = job_rx.lock().unwrap().recv(); match next { Ok(job) => ..., Err(_) => break } }`
//        (`while let Ok(job) = job_rx.lock().unwrap().recv() { 일 }` 로 쓰면 일하는 동안에도 락을 쥐고 있어서 사실상 직렬 실행! 왜 그럴까요?)
//     4) 각 워커에 `res_tx.clone()` 을 주고, 원본 `res_tx` 는 `drop`.
//     5) jobs 를 전부 `job_tx.send(..)` 한 뒤 `drop(job_tx)` — 이게 "더 이상 일 없음" 신호입니다.
//     6) `res_rx.iter().collect()` → 모든 워커 join → 정렬해서 반환
//   💡 힌트: 채널이 닫히는 조건(모든 Sender 가 drop 됨)을 정확히 지키는 것이 이 문제의 핵심입니다.
//           빠뜨리면 프로그램이 **영원히 멈춥니다.** (`cargo test` 가 멈추면 `Ctrl-C` 후 drop 위치를 점검)
//   ❓ 왜 워커 수와 결과가 무관해야 하나요?: 스케줄링에 따라 어느 워커가 어느 Job 을 가져갈지는 매번 다르지만, "각 Job 의 결과" 는 같기 때문이에요.
// ─────────────────────────────────────────────
pub fn worker_pool(jobs: Vec<Job>, workers: usize) -> Vec<(usize, usize)> {
    todo!("TODO(7): worker_pool")
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
