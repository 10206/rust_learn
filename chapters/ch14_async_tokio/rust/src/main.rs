//! Ch14. async/await 와 Tokio — 가짜 LLM API 동시 호출
//!
//! 🎯 목표
//!   - Future 는 "게으르다"(await/spawn 전에는 실행되지 않음)는 것을 몸으로 느낍니다.
//!   - 순차 실행 vs 동시 실행(JoinSet / join!), timeout, retry(백오프), select! 를 직접 구현합니다.
//!   - tokio::spawn 의 'static + Send 요구와 spawn_blocking 을 이해합니다.
//!
//! 📂 대응 파일: ../python/main.py (asyncio),  ../cpp/main.cpp (std::async)
//!
//! 🚶 진행 방법
//!   1. TODO(1) 부터 번호 순서대로 `todo!()` 를 지우고 구현합니다.
//!   2. 중간중간 `cargo test -p ch14_async_tokio` 로 확인합니다. (테스트는 "가짜 시계"를 써서 실제로는 기다리지 않아요!)
//!   3. 🔬 실험 블록은 주석을 풀어서 컴파일러 에러/경고를 직접 확인해 보세요.
//!
//! ✅ 완료 기준
//!   `cargo test -p ch14_async_tokio` 가 모두 초록불이고,
//!   `../../scripts/compare.sh ch14` 가 Python / C++ / Rust 출력이 같다고 나와야 합니다.
//!   (막히면 정답: `cargo run -q -p ch14_async_tokio --example solution`)

// 다 풀고 나면 이 줄을 지워 보세요 (그러면 안 쓴 코드에 대한 경고가 보입니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::fmt;
use std::future::Future;
use std::time::{Duration, Instant};

use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};

/// 가짜 LLM 호출 한 건.
/// 💡 prompt 가 &str 이 아니라 String 인 이유: tokio::spawn 으로 보낼 데이터는 'static 이어야 해서
///    "빌린 데이터" 대신 "소유한 데이터"가 편합니다. (아래 🔬 실험 A 참고)
#[derive(Debug, Clone)]
pub struct Call {
    pub prompt: String,
    pub latency_ms: u64,
}

impl Call {
    pub fn new(prompt: &str, latency_ms: u64) -> Self {
        Call { prompt: prompt.to_string(), latency_ms }
    }
}

#[derive(Debug, PartialEq)]
pub enum CallError {
    Timeout(u64),
    Unavailable(u32),
}

impl fmt::Display for CallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallError::Timeout(ms) => write!(f, "timed out after {ms}ms"),
            CallError::Unavailable(n) => write!(f, "service unavailable (attempt {n})"),
        }
    }
}

impl std::error::Error for CallError {}

// ─────────────────────────────────────────────
// TODO(1): latency_ms 만큼 "비동기로" 기다린 뒤 "answer to '<prompt>'" 를 돌려주세요.
//   🐍 Python: await asyncio.sleep(call.latency_ms / 1000)      (python/main.py: fake_llm_call)
//   ⚙️ C++  : std::this_thread::sleep_for(...)  (스레드를 재웁니다)   (cpp/main.cpp: fake_llm_call)
//   🦀 Rust : tokio::time::sleep(Duration::from_millis(..)).await
//   💡 힌트: 이미 위에서 `use tokio::time::sleep` 을 해 두었어요. `std::thread::sleep` 을 쓰면 안 되는 이유는 README 참고!
//   ❓ 왜?: async fn 안에서 스레드를 재우면 그 스레드에서 돌던 다른 작업까지 같이 멈춥니다.
//
//   🔬 실험 B (Future 는 게으르다): 아래 한 줄을 main() 안에 넣고 컴파일해 보세요.
//        sleep(Duration::from_millis(1));     // .await 를 빼먹음
//      -> `warning: unused implementor of `Future` that must be used`
//      ❓ 이 sleep 은 실제로 1ms 를 기다렸을까요?
// ─────────────────────────────────────────────
pub async fn fake_llm_call(call: &Call) -> String {
    todo!("TODO(1): sleep 후 답변 문자열 반환")
}

// ─────────────────────────────────────────────
// TODO(2): calls 를 하나씩 차례로 await 해서 결과를 입력 순서대로 Vec 에 담아 돌려주세요.
//   🐍 Python: for c in calls: results.append(await fake_llm_call(c))   (run_sequential)
//   ⚙️ C++  : for 루프에서 fake_llm_call 직접 호출                        (run_sequential)
//   🦀 Rust : for call in calls { ... fake_llm_call(call).await ... }
//   💡 힌트: Vec::with_capacity(calls.len())
//   ❓ 왜?: 총 시간이 지연 시간의 "합"(900ms)이 되는 것이 기준선입니다. 다음 TODO 와 비교해 보세요.
// ─────────────────────────────────────────────
pub async fn run_sequential(calls: &[Call]) -> Vec<String> {
    todo!("TODO(2): 순차 실행")
}

// ─────────────────────────────────────────────
// TODO(3): 모든 호출을 tokio::spawn 계열로 "동시에" 실행하고, 결과를 입력 순서대로 돌려주세요.
//   🐍 Python: await asyncio.gather(*(fake_llm_call(c) for c in calls))   (run_concurrent)
//   ⚙️ C++  : std::async(std::launch::async, ...) 여러 개 -> get()         (run_concurrent)
//   🦀 Rust : JoinSet::new(), set.spawn(async move { (i, ...) }), set.join_next().await
//   💡 힌트: spawn 에 넘기는 async 블록은 'static 이어야 합니다. calls.iter().cloned().enumerate() 로 소유한 Call 을 만드세요.
//           join_next() 는 "먼저 끝난 순서"로 나오니, 인덱스 i 를 같이 들고 다니다가 sort_by_key 로 정렬하세요.
//   ❓ 왜?: 순서를 정렬하지 않으면 실행할 때마다 출력이 달라질 수 있어요 (비결정적).
//
//   🔬 실험 A ('static): 아래를 main() 안에 넣어 보세요.
//        let calls = vec![Call::new("x", 1)];
//        let h = tokio::spawn(async { println!("{}", calls.len()); });
//      -> error[E0373]: async block may outlive the current function, but it borrows `calls`
//      ❓ `async move` 로 바꾸면 왜 해결될까요? 그러면 이후에 calls 를 쓸 수 있을까요?
//   🔬 실험 C (Send): 아래를 넣어 보세요.
//        let rc = std::rc::Rc::new(5);
//        tokio::spawn(async move { sleep(Duration::from_millis(1)).await; println!("{rc}"); });
//      -> error: future cannot be sent between threads safely   (`Rc<i32>` 는 `Send` 가 아님)
//      ❓ .await 를 지우면 에러가 사라질까요? (힌트: 에러의 핵심은 "await 를 가로질러 살아 있는 값"입니다)
// ─────────────────────────────────────────────
pub async fn run_concurrent(calls: &[Call]) -> Vec<String> {
    todo!("TODO(3): JoinSet 으로 동시 실행, 입력 순서로 정렬")
}

// ─────────────────────────────────────────────
// TODO(4): 두 호출을 동시에 실행해서 (String, String) 으로 돌려주세요.
//   🐍 Python: await asyncio.gather(fake_llm_call(a), fake_llm_call(b))   (join_pair)
//   ⚙️ C++  : std::async 두 개 후 get() 두 번                              (join_pair)
//   🦀 Rust : tokio::join!(f1, f2)   -> 튜플을 돌려줍니다
//   💡 힌트: join! 은 spawn 과 달리 새 태스크를 만들지 않고 "같은 태스크 안에서" 두 Future 를 번갈아 poll 합니다.
//           그래서 'static 이 필요 없고, 참조(&Call)를 그대로 써도 됩니다.
// ─────────────────────────────────────────────
pub async fn join_pair(a: &Call, b: &Call) -> (String, String) {
    todo!("TODO(4): tokio::join!")
}

// ─────────────────────────────────────────────
// TODO(5): limit_ms 안에 끝나지 않으면 Err(CallError::Timeout(limit_ms)) 를 돌려주세요.
//   🐍 Python: asyncio.wait_for(coro, seconds) -> TimeoutError           (call_with_timeout)
//   ⚙️ C++  : future.wait_for(...) == std::future_status::timeout        (call_with_timeout)
//   🦀 Rust : tokio::time::timeout(Duration, future).await  -> Result<T, Elapsed>
//   💡 힌트: .map_err(|_| CallError::Timeout(limit_ms))
//   ❓ 왜?: 시간이 지나면 안쪽 Future 가 drop 되면서 "취소"됩니다. C++ 의 std::async 는 이게 안 돼요 (README 참고).
// ─────────────────────────────────────────────
pub async fn call_with_timeout(call: &Call, limit_ms: u64) -> Result<String, CallError> {
    todo!("TODO(5): timeout")
}

/// 처음 두 번은 실패하고 세 번째 시도부터 성공하는 불안정한 API (이미 구현되어 있어요).
pub async fn fake_flaky(prompt: &str, attempt: u32) -> Result<String, CallError> {
    sleep(Duration::from_millis(10)).await;
    if attempt < 3 {
        Err(CallError::Unavailable(attempt))
    } else {
        Ok(format!("answer to '{prompt}' (attempt {attempt})"))
    }
}

// ─────────────────────────────────────────────
// TODO(6): op(attempt) 를 호출해 성공하면 반환, 실패하면 아래 형식으로 출력한 뒤 재시도하세요.
//          출력:   println!("  attempt {attempt} failed: {e}");
//          백오프: 실패 후 base_ms * 2^(attempt-1) ms 만큼 sleep (10 -> 20 -> 40 ...)
//          max_attempts 번째도 실패하면 마지막 에러를 Err 로 돌려줍니다 (이때는 sleep 하지 않음).
//   🐍 Python: while True: try: return await op(attempt) except CallError ...   (retry)
//   ⚙️ C++  : while (true) { Outcome r = op(attempt); ... }                      (retry)
//   🦀 Rust : loop { match op(attempt).await { Ok(v) => return Ok(v), Err(e) => ... } }
//   💡 힌트: 2u64.pow(attempt - 1).  op 는 FnMut(u32) -> Fut 이므로 `mut op` 로 받았고, 시도마다 새 Future 를 만듭니다.
//   ❓ 왜?: Future 는 한 번 await 하면 소모됩니다. "다시 시도"하려면 새 Future 가 필요해서 클로저(op)를 받는 거예요.
// ─────────────────────────────────────────────
pub async fn retry<F, Fut>(max_attempts: u32, base_ms: u64, mut op: F) -> Result<String, CallError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<String, CallError>>,
{
    todo!("TODO(6): 지수 백오프 재시도")
}

// ─────────────────────────────────────────────
// TODO(7): 두 호출 중 먼저 끝나는 쪽의 답을 돌려주세요. 진 쪽은 취소됩니다.
//   🐍 Python: asyncio.wait(tasks, return_when=FIRST_COMPLETED) 후 나머지 cancel()   (race)
//   ⚙️ C++  : wait_for(1ms) 로 두 future 를 번갈아 확인 (진 쪽도 끝까지 기다림)           (race)
//   🦀 Rust : tokio::select! { x = fut_a => x, y = fut_b => y }
//   💡 힌트: select! 의 각 갈래는 `패턴 = future => 표현식` 모양입니다.
//   ❓ 왜?: 진 쪽 Future 는 drop 되는 것만으로 취소됩니다. 별도의 cancel() 호출이 필요 없어요.
// ─────────────────────────────────────────────
pub async fn race(a: &Call, b: &Call) -> String {
    todo!("TODO(7): tokio::select!")
}

/// 1² + 2² + ... + n² (CPU 를 쓰는 동기 함수, 이미 구현되어 있어요)
pub fn sum_of_squares(n: u64) -> u64 {
    (1..=n).map(|i| i * i).sum()
}

// ─────────────────────────────────────────────
// TODO(8): sum_of_squares(n) 을 tokio::task::spawn_blocking 으로 실행하고 결과를 돌려주세요.
//   🐍 Python: await asyncio.to_thread(sum_of_squares, n)         (blocking_sum)
//   ⚙️ C++  : std::async(std::launch::async, sum_of_squares, n).get()   (blocking_sum)
//   🦀 Rust : spawn_blocking(move || ...).await  -> Result<T, JoinError>
//   💡 힌트: 클로저는 'static 이라 n 을 `move` 로 가져가야 합니다. JoinError 는 .expect(..) 로 처리하세요.
//   ❓ 왜?: 오래 걸리는 동기 코드를 async 태스크 안에서 그냥 돌리면 그 워커 스레드의 다른 태스크가 모두 굶습니다.
// ─────────────────────────────────────────────
pub async fn blocking_sum(n: u64) -> u64 {
    todo!("TODO(8): spawn_blocking")
}

#[tokio::main]
async fn main() {
    println!("== 1. futures are lazy ==");
    let lazy_call = Call::new("lazy", 10);
    let fut = async {
        println!("  [lazy] body is running");
        fake_llm_call(&lazy_call).await
    }; // Future 값만 만들어졌을 뿐, 아직 아무것도 실행되지 않았습니다.
    println!("future created, nothing has run yet");
    println!("now awaiting the future");
    println!("result: {}", fut.await);

    let calls = vec![
        Call::new("summarize", 100),
        Call::new("translate", 200),
        Call::new("classify", 300),
        Call::new("extract", 150),
        Call::new("rewrite", 150),
    ];

    println!("== 2. sequential ==");
    let t0 = Instant::now();
    let seq = run_sequential(&calls).await;
    let seq_ms = t0.elapsed().as_millis();
    for (i, r) in seq.iter().enumerate() {
        println!("[{i}] {r}");
    }

    println!("== 3. concurrent ==");
    let t0 = Instant::now();
    let conc = run_concurrent(&calls).await;
    let conc_ms = t0.elapsed().as_millis();
    for (i, r) in conc.iter().enumerate() {
        println!("[{i}] {r}");
    }
    // 순차 합계 900ms vs 동시 최댓값 300ms: 실제 시간은 출력하지 않고 큰 마진으로만 비교합니다.
    println!("concurrent faster: {}", if conc_ms * 2 < seq_ms { "yes" } else { "no" });

    println!("== 4. join! ==");
    let (ra, rb) = join_pair(&Call::new("a", 100), &Call::new("b", 200)).await;
    println!("pair: {ra} | {rb}");

    println!("== 5. timeout ==");
    for (call, limit) in [(Call::new("slow", 500), 150), (Call::new("quick", 50), 150)] {
        let label = format!("{} ({}ms, limit {}ms)", call.prompt, call.latency_ms, limit);
        match call_with_timeout(&call, limit).await {
            Ok(answer) => println!("{label}: ok: {answer}"),
            Err(e) => println!("{label}: error: {e}"),
        }
    }

    println!("== 6. retry with backoff ==");
    for max_attempts in [5, 2] {
        match retry(max_attempts, 10, |n| fake_flaky("flaky", n)).await {
            Ok(answer) => println!("retry (max {max_attempts}): ok: {answer}"),
            Err(e) => println!("retry (max {max_attempts}): error: {e}"),
        }
    }

    println!("== 7. select (race) ==");
    let winner = race(&Call::new("fast_model", 50), &Call::new("slow_model", 300)).await;
    println!("winner: {winner}");

    println!("== 8. blocking work ==");
    let n = 1_000_000;
    println!("sum of squares 1..={n}: {}", blocking_sum(n).await);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    // start_paused = true: 가짜 시계. sleep 은 실제로 기다리지 않고 시계를 앞으로 돌립니다.
    // 그래서 아래 시간 단언은 느린 CI 에서도 정확히 같은 값이 나옵니다.
    fn sample_calls() -> Vec<Call> {
        vec![Call::new("a", 100), Call::new("b", 200), Call::new("c", 300)]
    }

    #[tokio::test(start_paused = true)]
    async fn fake_call_waits_and_answers() {
        let t0 = tokio::time::Instant::now();
        let answer = fake_llm_call(&Call::new("hi", 250)).await;
        assert_eq!(answer, "answer to 'hi'");
        assert_eq!(t0.elapsed(), Duration::from_millis(250));
    }

    #[tokio::test(start_paused = true)]
    async fn sequential_sums_latencies() {
        let t0 = tokio::time::Instant::now();
        let out = run_sequential(&sample_calls()).await;
        assert_eq!(out, ["answer to 'a'", "answer to 'b'", "answer to 'c'"]);
        assert_eq!(t0.elapsed(), Duration::from_millis(600));
    }

    #[tokio::test(start_paused = true)]
    async fn concurrent_takes_the_max_and_keeps_order() {
        let t0 = tokio::time::Instant::now();
        let out = run_concurrent(&sample_calls()).await;
        assert_eq!(out, ["answer to 'a'", "answer to 'b'", "answer to 'c'"]);
        assert_eq!(t0.elapsed(), Duration::from_millis(300));
    }

    #[tokio::test(start_paused = true)]
    async fn join_pair_runs_both_at_once() {
        let t0 = tokio::time::Instant::now();
        let (x, y) = join_pair(&Call::new("x", 100), &Call::new("y", 200)).await;
        assert_eq!((x.as_str(), y.as_str()), ("answer to 'x'", "answer to 'y'"));
        assert_eq!(t0.elapsed(), Duration::from_millis(200));
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_cancels_slow_calls() {
        let t0 = tokio::time::Instant::now();
        let slow = call_with_timeout(&Call::new("slow", 500), 150).await;
        assert_eq!(slow, Err(CallError::Timeout(150)));
        assert_eq!(t0.elapsed(), Duration::from_millis(150)); // 500ms 를 기다리지 않았습니다
        let quick = call_with_timeout(&Call::new("quick", 50), 150).await;
        assert_eq!(quick, Ok("answer to 'quick'".to_string()));
    }

    #[tokio::test(start_paused = true)]
    async fn retry_succeeds_on_third_attempt() {
        let tries = Arc::new(AtomicU32::new(0));
        let counter = Arc::clone(&tries);
        let result = retry(5, 10, move |n| {
            counter.fetch_add(1, Ordering::SeqCst);
            fake_flaky("f", n)
        })
        .await;
        assert_eq!(result, Ok("answer to 'f' (attempt 3)".to_string()));
        assert_eq!(tries.load(Ordering::SeqCst), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn retry_gives_up_and_backs_off() {
        let t0 = tokio::time::Instant::now();
        let result = retry(2, 10, |n| fake_flaky("f", n)).await;
        assert_eq!(result, Err(CallError::Unavailable(2)));
        // 시도1(10ms) + 백오프(10ms) + 시도2(10ms)
        assert_eq!(t0.elapsed(), Duration::from_millis(30));
    }

    #[tokio::test(start_paused = true)]
    async fn race_returns_the_faster_answer() {
        let t0 = tokio::time::Instant::now();
        let winner = race(&Call::new("slow", 300), &Call::new("fast", 50)).await;
        assert_eq!(winner, "answer to 'fast'");
        assert_eq!(t0.elapsed(), Duration::from_millis(50));
    }

    #[tokio::test]
    async fn blocking_sum_matches_formula() {
        assert_eq!(blocking_sum(10).await, 385);
        assert_eq!(blocking_sum(1000).await, 333_833_500);
    }

    #[tokio::test]
    async fn futures_do_nothing_until_awaited() {
        let flag = Arc::new(AtomicBool::new(false));
        let inner = Arc::clone(&flag);
        let fut = async move {
            inner.store(true, Ordering::SeqCst);
        };
        assert!(!flag.load(Ordering::SeqCst), "만들기만 했는데 실행되면 안 됩니다");
        fut.await;
        assert!(flag.load(Ordering::SeqCst));
    }
}
