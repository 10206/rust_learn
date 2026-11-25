//! Ch14. async/await 와 Tokio — 정답 (solution)
//!
//! 대응 파일: ../python/main.py (asyncio), ../cpp/main.cpp (std::async)
//! 실행: cargo run -q -p ch14_async_tokio --example solution
//! 테스트: cargo test -p ch14_async_tokio --example solution

use std::fmt;
use std::future::Future;
use std::time::{Duration, Instant};

use tokio::task::JoinSet;
use tokio::time::{sleep, timeout};

/// 가짜 LLM 호출 한 건. 소유한 String 을 쓰는 이유: tokio::spawn 은 'static 을 요구하기 때문입니다.
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

/// TODO(1): latency 만큼 "비동기로" 기다린 뒤 답을 돌려줍니다.
pub async fn fake_llm_call(call: &Call) -> String {
    // tokio::time::sleep 은 스레드를 막지 않고 "나중에 깨워 달라"고 등록만 합니다.
    sleep(Duration::from_millis(call.latency_ms)).await;
    format!("answer to '{}'", call.prompt)
}

/// TODO(2): 순차 실행. 총 시간 = 지연 시간의 합.
pub async fn run_sequential(calls: &[Call]) -> Vec<String> {
    let mut results = Vec::with_capacity(calls.len());
    for call in calls {
        results.push(fake_llm_call(call).await);
    }
    results
}

/// TODO(3): JoinSet 으로 모두 spawn → 동시 실행. 결과는 입력 순서로 돌려줍니다.
pub async fn run_concurrent(calls: &[Call]) -> Vec<String> {
    let mut set = JoinSet::new();
    for (i, call) in calls.iter().cloned().enumerate() {
        // spawn 은 'static + Send 를 요구하므로 call 을 clone 해서 async move 로 넘깁니다.
        set.spawn(async move { (i, fake_llm_call(&call).await) });
    }
    let mut results = Vec::with_capacity(calls.len());
    // join_next 는 "먼저 끝난 순서"로 나옵니다 → 인덱스로 다시 정렬해야 결정적입니다.
    while let Some(joined) = set.join_next().await {
        results.push(joined.expect("task panicked"));
    }
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, answer)| answer).collect()
}

/// TODO(4): 두 호출을 동시에 실행 (tokio::join!).
pub async fn join_pair(a: &Call, b: &Call) -> (String, String) {
    tokio::join!(fake_llm_call(a), fake_llm_call(b))
}

/// TODO(5): 제한 시간 안에 못 끝나면 Err(CallError::Timeout).
pub async fn call_with_timeout(call: &Call, limit_ms: u64) -> Result<String, CallError> {
    // timeout 이 Elapsed 를 돌려주면 안쪽 Future 는 drop 되어 그 자리에서 취소됩니다.
    timeout(Duration::from_millis(limit_ms), fake_llm_call(call))
        .await
        .map_err(|_elapsed| CallError::Timeout(limit_ms))
}

/// 처음 두 번은 실패하고 세 번째 시도부터 성공하는 불안정한 API.
pub async fn fake_flaky(prompt: &str, attempt: u32) -> Result<String, CallError> {
    sleep(Duration::from_millis(10)).await;
    if attempt < 3 {
        Err(CallError::Unavailable(attempt))
    } else {
        Ok(format!("answer to '{prompt}' (attempt {attempt})"))
    }
}

/// TODO(6): 지수 백오프 재시도. op(attempt) 를 부르고, 실패하면 base, 2*base, 4*base ... 만큼 쉬었다가 재시도.
pub async fn retry<F, Fut>(max_attempts: u32, base_ms: u64, mut op: F) -> Result<String, CallError>
where
    F: FnMut(u32) -> Fut,
    Fut: Future<Output = Result<String, CallError>>,
{
    let mut attempt = 1;
    loop {
        match op(attempt).await {
            Ok(answer) => return Ok(answer),
            Err(e) => {
                println!("  attempt {attempt} failed: {e}");
                if attempt >= max_attempts {
                    return Err(e);
                }
                sleep(Duration::from_millis(base_ms * 2u64.pow(attempt - 1))).await;
                attempt += 1;
            }
        }
    }
}

/// TODO(7): 먼저 끝나는 쪽의 답을 돌려줍니다. 진 쪽 Future 는 select! 가 drop(취소) 합니다.
pub async fn race(a: &Call, b: &Call) -> String {
    tokio::select! {
        answer = fake_llm_call(a) => answer,
        answer = fake_llm_call(b) => answer,
    }
}

/// 1² + 2² + ... + n² (CPU 를 쓰는 동기 함수)
pub fn sum_of_squares(n: u64) -> u64 {
    (1..=n).map(|i| i * i).sum()
}

/// TODO(8): CPU 작업은 async 스레드를 막지 않도록 spawn_blocking 으로 보냅니다.
pub async fn blocking_sum(n: u64) -> u64 {
    tokio::task::spawn_blocking(move || sum_of_squares(n))
        .await
        .expect("blocking task panicked")
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
