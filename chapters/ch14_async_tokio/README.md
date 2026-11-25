# Ch14. async/await 와 Tokio

> "기다리는 동안 다른 일을 하는" 코드를 쓰는 법. Python `asyncio` 와 거의 똑같지만, Future 는 게으르고 런타임은 라이브러리입니다.  ·  예상 소요: 이론 50분 / 실습 70분  ·  선행: Ch13 (동시성), Ch06 (에러 처리)

## 🎯 학습 목표

- `async fn` 이 "값"이 아니라 **Future(나중에 완성될 계산)** 를 돌려준다는 것, 그리고 Future 는 **`.await` 하거나 spawn 하기 전에는 아무것도 실행하지 않는다**(lazy)는 것을 설명할 수 있습니다.
- 런타임(Tokio)이 **표준 라이브러리가 아니라 라이브러리**라는 점, `#[tokio::main]` 이 하는 일을 이해합니다.
- 순차 실행 vs 동시 실행(`JoinSet`, `tokio::join!`), `timeout`, 재시도(백오프), `select!` 를 직접 구현합니다.
- `tokio::spawn` 이 요구하는 **`'static + Send`** 의 의미와, async 안에서 **블로킹하면 버그**인 이유(`spawn_blocking`)를 압니다.
- AI 에이전트가 왜 async 인지(LLM API 호출 = 기다림의 연속) 이해합니다. → Ch16, Ch17 의 토대

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust (Tokio) |
|---|---|---|---|
| 비동기 함수 | 코루틴 (`co_await`, 직접 타입 필요) | `async def` | `async fn` |
| 기다리기 | `co_await x` / `future.get()` | `await x` | `x.await` |
| "나중에 될 값" | `std::future<T>` / 코루틴 핸들 | 코루틴 객체 / `Task` / `Future` | `impl Future<Output = T>` |
| 만들면 바로 실행? | `std::async(launch::async)` 는 **바로**, `launch::deferred` 는 `get()` 때 | **아니요** (lazy) — `create_task` 하면 예약됨 | **아니요** (lazy) — `spawn` 하면 예약됨 |
| 실행기(런타임) | 표준에 **없음** (Asio, libunifex ...) | `asyncio.run()` (표준 라이브러리 안) | Tokio 등 **외부 크레이트** |
| 백그라운드 태스크 | `std::jthread` / `std::async` | `asyncio.create_task` | `tokio::spawn` |
| 여러 개 동시에 | 여러 `std::async` + `get()` | `asyncio.gather` / `TaskGroup` | `tokio::join!` / `JoinSet` |
| 제한 시간 | `wait_for` (취소는 안 됨!) | `asyncio.wait_for` | `tokio::time::timeout` |
| 먼저 끝난 것 | 직접 폴링 | `asyncio.wait(FIRST_COMPLETED)` | `tokio::select!` |
| 취소 | 표준에 없음 (`stop_token` 정도) | `task.cancel()` (예외로 전달) | Future 를 **drop** 하면 취소 |
| 비동기 sleep | (코루틴 라이브러리마다 다름) | `asyncio.sleep` | `tokio::time::sleep` |
| 블로킹 코드 격리 | 별도 스레드 | `asyncio.to_thread` | `tokio::task::spawn_blocking` |

> 💡 **쉬운 설명 — async 가 필요한 이유**
> 식당에 비유해 볼게요. 주방장(스레드)이 라면을 끓이는 3분 동안 가만히 서서 기다리면 손님 한 명밖에 못 받아요(순차/블로킹).
> 타이머를 맞춰 두고 그 사이에 다른 주문을 처리하면 한 명의 주방장이 여러 주문을 돌볼 수 있어요(async).
> LLM API 호출은 "네트워크 응답을 기다리는 시간"이 대부분이라서, 여러 호출을 한 스레드로 동시에 돌리기에 딱 좋습니다.

## 📘 이론

> 이 장의 코드 조각은 <https://play.rust-lang.org> 에서도 돌아갑니다. (Playground 는 `tokio` 를 지원합니다. 단, `Cargo.toml` 이 필요한 `start_paused` 같은 test-util 기능은 로컬에서 해 보세요.)

### 1. Future 는 "게으른 계산"입니다

`async fn` 을 **호출**하면 함수 본문이 실행되는 게 아니라 **Future 라는 값**이 만들어집니다. 본문은 그 Future 를 `.await` 할 때 비로소 실행됩니다.

```rust
async fn answer() -> u32 {
    println!("본문 실행!");
    42
}

#[tokio::main]
async fn main() {
    let fut = answer();          // 아무것도 출력되지 않습니다. fut 은 "할 일 목록"일 뿐!
    println!("만들었어요");
    let n = fut.await;           // 여기서 "본문 실행!" 이 출력됩니다.
    println!("{n}");
}
```

출력 순서: `만들었어요` → `본문 실행!` → `42`.

- **Python 과 같은 점**: 코루틴 객체(`answer()`)도 만들기만 해서는 실행되지 않습니다. (`await` 를 빼먹으면 `RuntimeWarning: coroutine ... was never awaited`)
- **C++ 과의 차이**: `std::async(std::launch::async, f)` 는 만드는 즉시 스레드를 띄워 실행합니다. Rust Future 와 가장 비슷한 건 `std::launch::deferred` (get 할 때 실행) 입니다. 이 장의 `cpp/main.cpp` 1번 섹션이 그 예입니다.

> ⚠️ **C++ 개발자가 헷갈리는 점**: "`.await` 를 빼먹어도 컴파일은 되고 아무 일도 안 일어난다"는 게 가장 흔한 실수입니다. 다행히 컴파일러가 경고해 줘요:
> `warning: unused implementor of `Future` that must be used` — 🔬 실험 B 에서 직접 확인합니다.

### 2. 런타임은 라이브러리입니다 (`#[tokio::main]`)

Future 는 스스로 움직이지 못합니다. 누군가 `poll` 을 반복 호출해서 "이제 진행할 수 있어?" 하고 물어봐야 하는데, 그 역할을 하는 것이 **런타임(실행기, executor)** 입니다.

- Python: `asyncio.run(main())` — 런타임이 표준 라이브러리에 들어 있습니다.
- Rust: 표준 라이브러리에는 `Future` trait 과 `async/await` **문법**만 있고 런타임은 없습니다. **Tokio** 가 사실상의 표준입니다.
- C++20: 코루틴 **문법**만 있고, 코루틴 반환 타입(Task)과 실행기가 표준에 없어서 직접 만들거나 Asio/cppcoro 같은 라이브러리가 필요합니다. 그래서 이 장의 C++ 예제는 코루틴 대신 `std::async` / `std::future` / 스레드로 "동시성"을 흉내 냅니다.

```rust
#[tokio::main]
async fn main() {
    /* ... */
}
// 위는 대략 아래와 같은 뜻입니다 (매크로가 펼쳐 줌):
fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        /* ... */
    })
}
```

> 💡 **쉬운 설명 — 왜 런타임을 분리했나요?**
> 서버(수천 개 연결), 임베디드(스레드 없음), 브라우저(wasm)는 "일을 돌리는 방식"이 전부 달라요. Rust 는 문법만 표준으로 두고 돌리는 엔진은 골라 쓰게 했습니다. 이 커리큘럼은 Tokio 하나만 씁니다.

### 3. 순차 vs 동시: `.await` 의 위치가 전부입니다

```rust
// 순차: 앞 호출이 끝나야 다음 호출이 시작 → 총 시간 = 합
for call in calls {
    results.push(fake_llm_call(call).await);
}

// 동시: 모두 spawn 해 두고 나중에 결과를 모음 → 총 시간 = 최댓값
let mut set = tokio::task::JoinSet::new();
for (i, call) in calls.iter().cloned().enumerate() {
    set.spawn(async move { (i, fake_llm_call(&call).await) });
}
while let Some(done) = set.join_next().await {
    // 먼저 끝난 순서로 나옵니다!
}
```

이 장의 실습에서 지연 시간 `100+200+300+150+150 = 900ms` 를 순차로 실행하면 약 900ms, 동시로 실행하면 약 300ms(가장 긴 것)입니다.

- `tokio::join!(a, b)`: 고정된 개수의 Future 를 **같은 태스크 안에서** 번갈아 진행합니다. 새 태스크를 만들지 않으니 `'static` 요구가 없고 지역 변수를 빌려 써도 됩니다.
- `JoinSet`: 개수가 정해지지 않은(동적인) 태스크들을 `spawn` 해서 모읍니다. 각각 별도의 태스크라서 다른 스레드에서 병렬로 실행될 수도 있습니다.
- Python 대응: `join!` ≈ `asyncio.gather(a, b)`, `JoinSet` ≈ `asyncio.TaskGroup` / `gather(*tasks)`.

> ⚠️ **결정적(deterministic) 출력 주의**: `join_next()` 는 "끝난 순서"로 결과를 줍니다. 순서가 중요하면 인덱스를 같이 들고 다니다가 정렬하세요. (이 장의 `run_concurrent`)

### 4. `tokio::spawn` 은 `'static + Send` 를 요구합니다

```rust
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
```

- **`'static`**: 스폰된 태스크는 "스폰한 함수가 끝난 뒤에도" 살아 있을 수 있습니다. 그래서 지역 변수의 **참조**를 들고 있으면 안 됩니다. → `async move` 로 **소유권을 넘기거나**, `clone()` / `Arc` 를 씁니다.
- **`Send`**: 태스크는 런타임의 워커 스레드들 사이를 옮겨 다닐 수 있습니다(work stealing). Ch13 에서 배운 `Send` 가 그대로 등장합니다. `Rc` 나 `MutexGuard`(std) 를 `.await` 너머로 들고 있으면 `Send` 가 아니라서 컴파일 에러가 납니다.

> 💡 **쉬운 설명 — `'static` 은 "영원히 산다"가 아니에요**
> "프로그램 끝까지 살 수 **있을 만큼** 아무에게도 빌린 게 없다"는 뜻입니다. 소유한 `String`, `Vec`, `Arc<T>` 는 `'static` 이에요.

Python 과 비교하면 이 부분이 가장 큰 차이입니다. Python 은 `create_task(coro)` 에 지역 변수를 마음대로 넘길 수 있고, 위험은 실행 중에야 드러납니다. Rust 는 컴파일 타임에 막아 줍니다.

### 5. 블로킹은 async 의 적입니다

```rust
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));       // ❌ 스레드를 통째로 재움
}
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await; // ✅ "1초 뒤에 깨워 줘" 하고 양보
}
```

async 런타임은 소수의 워커 스레드로 수많은 태스크를 돌립니다. 한 태스크가 `.await` 없이 오래 붙잡고 있으면(= **블로킹**) 그 스레드의 다른 태스크가 모두 멈춥니다. 컴파일러는 이걸 잡아 주지 않는 "조용한 버그"예요.

- 파일 읽기(`std::fs`), 무거운 계산, 동기 라이브러리 호출, `std::thread::sleep`, `std::sync::Mutex` 를 오래 잡기 등이 블로킹입니다.
- 해결: `tokio::task::spawn_blocking(|| 무거운 계산)` — 별도의 블로킹 전용 스레드 풀로 보냅니다. (Python: `asyncio.to_thread`, C++: 그냥 스레드)

### 6. timeout, 취소, select!

```rust
use tokio::time::{timeout, Duration};

match timeout(Duration::from_millis(150), fake_llm_call(&call)).await {
    Ok(answer) => println!("{answer}"),
    Err(_elapsed) => println!("너무 느려요"),
}
```

Rust 의 취소 모델은 단순합니다: **Future 를 drop 하면 그 자리에서 멈춥니다.** 별도의 `cancel()` 도, 예외 전파도 없어요.

- `timeout(d, fut)`: 시간이 지나면 `fut` 을 drop 하고 `Err(Elapsed)`.
- `select!`: 여러 Future 를 동시에 기다리다가 **먼저 끝난 것의 갈래만 실행**하고 나머지는 drop(취소).

```rust
let answer = tokio::select! {
    a = fast_model() => a,
    b = slow_model() => b,   // 진 쪽은 자동 취소
};
```

> ⚠️ **C++ 개발자가 헷갈리는 점**: `std::async` 가 돌려준 `std::future` 는 소멸자에서 작업이 끝날 때까지 **블록**합니다. 그래서 `wait_for` 로 timeout 을 감지해도 함수를 빠져나가는 시점에는 결국 작업이 끝나기를 기다립니다. (이 장의 `cpp/main.cpp` 주석 참고. 5번 섹션이 500ms 가 걸려요.) 취소가 언어/표준에 없다는 점이 Rust 와 가장 크게 다릅니다.

> ⚠️ **취소 안전성(cancel safety)**: `select!` 에서 진 Future 는 `.await` 지점에서 그냥 버려집니다. 그 사이에 "절반만 한 일"이 있으면 안 되겠죠. 이 장의 예제처럼 읽기 전용 호출은 안전하지만, 파일에 쓰다가 취소되는 경우 등은 조심해야 합니다. (고급 주제)

### 7. 재시도와 지수 백오프

네트워크 호출은 가끔 실패합니다. "바로 다시" 시도하면 서버가 더 힘들어지니, 실패할 때마다 기다리는 시간을 2배로 늘립니다 (10ms → 20ms → 40ms ...). 이를 **지수 백오프(exponential backoff)** 라고 해요.

```rust
pub async fn retry<F, Fut>(max_attempts: u32, base_ms: u64, mut op: F) -> Result<String, CallError>
where
    F: FnMut(u32) -> Fut,                                 // 시도할 때마다 "새 Future" 를 만드는 클로저
    Fut: Future<Output = Result<String, CallError>>,
```

Future 는 한 번 `.await` 하면 소모되므로 "다시 시도"하려면 **Future 를 만드는 클로저**를 받아야 합니다. (Ch07 의 `FnMut` 이 여기서 쓰입니다.) 실전에서는 여기에 무작위 흔들림(jitter)을 더하는데, 이 장은 출력을 결정적으로 유지하려고 뺐습니다.

### 8. async vs 스레드, 언제 무엇을?

| | 스레드 (Ch13) | async (이 장) |
|---|---|---|
| 잘 맞는 일 | CPU 를 많이 쓰는 계산 | 네트워크/파일 등 **기다림이 많은** 일 |
| 비용 | 스레드 하나당 수 MB 스택 | 태스크 하나당 수백 바이트~수 KB |
| 개수 | 수십~수백 | 수만~수십만 |
| 코드 모양 | 일반 함수 | `async fn` + `.await` (함수 색칠 문제: async 는 async 에서만 `.await` 가능) |

AI 에이전트(LLM API 호출, 도구 실행, 스트리밍)는 거의 전부 "기다림"이므로 async 가 기본 선택입니다.

## 🧯 자주 만나는 컴파일 에러

**1. `.await` 를 빼먹음 (경고)**
```
warning: unused implementor of `Future` that must be used
  = note: futures do nothing unless you `.await` or poll them
```
→ 의미: Future 를 만들기만 하고 실행하지 않았어요. 고치는 법: `.await` 를 붙이거나 `tokio::spawn` 하세요.

**2. 스폰한 태스크가 지역 변수를 빌림 (E0373)**
```
error[E0373]: async block may outlive the current function, but it borrows `calls`, which is owned by the current function
  = note: async blocks are not executed immediately and must either take a reference or ownership of outside variables they use
```
→ 의미: `tokio::spawn` 은 `'static` 이어야 하는데 지역 변수 `calls` 를 빌렸어요. 고치는 법: `async move { ... }` 로 소유권을 넘기거나, 먼저 `clone()` 하세요.

**3. `Send` 가 아닌 값을 await 너머로 들고 있음**
```
error: future cannot be sent between threads safely
help: within `{async block}`, the trait `Send` is not implemented for `Rc<i32>`
```
→ 의미: `Rc` 는 스레드 간 이동이 안 돼요. 고치는 법: `Arc` 로 바꾸거나, `.await` 전에 값을 drop 하는 블록으로 범위를 좁히세요. `std::sync::MutexGuard` 도 같은 에러의 단골입니다 → `tokio::sync::Mutex` 를 쓰거나 락을 `.await` 전에 놓으세요.

**4. async 가 아닌 곳에서 `.await` (E0728)**
```
error[E0728]: `await` is only allowed inside `async` functions and blocks
```
→ 의미: 일반 `fn` 안에서는 `.await` 를 쓸 수 없어요. 고치는 법: 함수를 `async fn` 으로 바꾸고 호출하는 쪽도 `.await` 하세요. (async 가 호출 체인을 타고 올라가는 "함수 색칠" 현상)

**5. `main` 이 async 인데 런타임이 없음 (E0752)**
```
error[E0752]: `main` function is not allowed to be `async`
```
→ 고치는 법: `#[tokio::main]` 을 붙이세요.

**6. 재귀 async fn (E0733)** — `async fn` 이 자기 자신을 `.await` 하면 Future 의 크기를 정할 수 없어요. `Box::pin(self_call()).await` 로 감쌉니다. (Ch11 의 재귀 enum 과 같은 이유예요.)

## 🛠️ 실습: 가짜 LLM API 동시 호출

### 무엇을 만드나요

LLM API 를 흉내 내는 `fake_llm_call` (지연 시간만큼 기다린 뒤 `answer to '<prompt>'` 반환)을 만들고, 5건의 호출을 **순차 vs 동시**로 돌려 보고, `join!`, timeout, retry(백오프), `select!`, `spawn_blocking` 까지 8가지를 구현합니다.

출력 예시 (일부):

```text
== 1. futures are lazy ==
future created, nothing has run yet
now awaiting the future
  [lazy] body is running
result: answer to 'lazy'
== 2. sequential ==
[0] answer to 'summarize'
...
== 3. concurrent ==
[0] answer to 'summarize'
...
concurrent faster: yes
== 5. timeout ==
slow (500ms, limit 150ms): error: timed out after 150ms
quick (50ms, limit 150ms): ok: answer to 'quick'
== 6. retry with backoff ==
  attempt 1 failed: service unavailable (attempt 1)
  attempt 2 failed: service unavailable (attempt 2)
retry (max 5): ok: answer to 'flaky' (attempt 3)
```

> 출력은 **결정적**이어야 합니다: 실제 걸린 시간(ms)은 출력하지 않고, 결과는 항상 입력 순서로 출력하며, "동시가 더 빠르다"는 900ms vs 300ms 라는 큰 차이(2배 이상)로만 판정합니다. (느린 컴퓨터에서도 안전)

### 파일 구성

| 경로 | 설명 |
|---|---|
| `python/main.py` | asyncio 로 만든 참조 구현 |
| `cpp/main.cpp` | `std::async` / `std::future` 로 만든 참조 구현 (C++20 코루틴은 표준에 실행기가 없어 사용하지 않음) |
| `rust/src/main.rs` | **실습 파일**: 가이드 주석 + `todo!()` |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법 (chapters/ch14_async_tokio 에서)

```bash
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch14_async_tokio                       # 실습 (TODO 를 풀기 전에는 todo! 로 패닉)
cargo test -p ch14_async_tokio                         # 실습 테스트
cargo run -q -p ch14_async_tokio --example solution    # 정답
../../scripts/compare.sh ch14                          # 세 언어 출력 비교 (--solution 이면 정답 기준)
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `fake_llm_call`: `tokio::time::sleep(..).await` 후 답변 | async fn, lazy Future, 논블로킹 sleep |
| 2 | `run_sequential`: for 루프 안에서 `.await` | 순차 실행 |
| 3 | `run_concurrent`: `JoinSet` + `spawn` + 인덱스 정렬 | `'static + Send`, 결정적 출력 |
| 4 | `join_pair`: `tokio::join!` | 같은 태스크 안의 동시성 |
| 5 | `call_with_timeout`: `tokio::time::timeout` | 취소 = drop |
| 6 | `retry`: 지수 백오프 | 제네릭 `FnMut -> Future` |
| 7 | `race`: `tokio::select!` | 먼저 끝난 것만 사용, 나머지는 취소 |
| 8 | `blocking_sum`: `spawn_blocking` | 블로킹 코드 격리 |

### 완료 기준

- `cargo test -p ch14_async_tokio` 전부 통과. (테스트는 `#[tokio::test(start_paused = true)]` 로 **가짜 시계**를 써서, 실제로 기다리지 않아도 `elapsed == 300ms` 같은 정확한 시간 단언이 가능합니다. → 느린 CI 에서도 흔들리지 않아요.)
- `../../scripts/compare.sh ch14` 가 Python / C++ / Rust 출력이 같다고 보고.
- 프로그램 전체 실행 시간은 약 2초 안팎입니다.

### 🔬 실험 목록

| 실험 | 어디서 | 확인할 것 |
|---|---|---|
| A | TODO(3) 주석 | `async { calls.len() }` 를 `tokio::spawn` → E0373. `async move` 로 고치면? |
| B | TODO(1) 주석 | `sleep(...)` 에 `.await` 를 빼면 `unused implementor of Future` 경고. 실제로 기다렸나요? |
| C | TODO(3) 주석 | `Rc` 를 spawn 안으로 → "future cannot be sent between threads safely" |
| D (선택) | `fake_llm_call` | `main` 을 `#[tokio::main(flavor = "current_thread")]` 로 바꾸고 `tokio::time::sleep` 을 `std::thread::sleep` 으로 바꿔 실행해 보세요. `concurrent faster: no` 가 나옵니다! (스레드 하나가 통째로 잠들어서 다른 태스크가 진행하지 못해요) |

## 🤖 바이브 코딩 포인트

AI 가 생성한 async Rust 코드를 볼 때 이렇게 확인하세요.

- **`.await` 누락**: `unused_must_use` 경고를 절대 무시하지 마세요. "아무 일도 안 하는 코드"가 됩니다. `cargo clippy -- -D warnings` 로 막으세요.
- **async 안의 블로킹 호출**: `std::thread::sleep`, `std::fs::read_to_string`, `reqwest::blocking`, 무거운 루프 — 전부 의심 대상. `tokio::time::sleep`, `tokio::fs`, `spawn_blocking` 으로 바꾸도록 요청하세요.
- **`std::sync::Mutex` 를 `.await` 너머로**: 락을 잡은 채 `.await` 하면 `Send` 에러 또는 교착(deadlock). "락은 `.await` 전에 놓고, 필요하면 `tokio::sync::Mutex`" 라고 지시하세요.
- **무한 대기 방지**: 외부 호출(LLM, HTTP)에는 항상 `timeout` 이 있는지 확인하세요. 없으면 에이전트가 영원히 멈춥니다.
- **순서 의존**: `JoinSet`/`select!` 결과는 완료 순서입니다. 순서가 중요한 출력/메시지 구성이면 인덱스로 정렬하는지 보세요 (Messages API 의 `tool_result` 순서 등).
- **프롬프트 팁**: "tokio 로 5개 호출을 동시에 실행하되 결과는 입력 순서로 반환하고, 호출마다 3초 timeout 과 최대 3회 지수 백오프 재시도를 적용해 줘. 블로킹 호출은 쓰지 마."

## ✅ 셀프 체크

1. `let f = fake_llm_call(&call);` 한 줄만 있고 `.await` 가 없을 때, 호출은 실행되나요?
   <details><summary>답</summary>아니요. Future 는 lazy 라서 만들기만 해서는 본문이 한 줄도 실행되지 않습니다. (컴파일러가 `unused_must_use` 경고를 줍니다.)</details>
2. `tokio::spawn` 에 넘기는 Future 가 `'static` 이어야 하는 이유는 무엇인가요?
   <details><summary>답</summary>스폰된 태스크는 스폰한 함수가 끝난 뒤에도 계속 실행될 수 있어서, 지역 변수에 대한 참조를 들고 있으면 dangling 이 됩니다. 그래서 소유권을 `move` 하거나 `Arc`/`clone` 을 씁니다.</details>
3. 한 `async fn` 안에서 `std::thread::sleep(1초)` 를 부르면 왜 문제인가요?
   <details><summary>답</summary>그 워커 스레드가 1초 동안 통째로 멈춰서, 같은 스레드에 배정된 다른 태스크들이 진행하지 못합니다. `tokio::time::sleep().await` 는 스레드를 풀어 주고 "나중에 깨워 달라"고만 등록합니다.</details>
4. Rust 에서 timeout 이 난 작업은 어떻게 "취소"되나요? C++ `std::async` 와 어떻게 다른가요?
   <details><summary>답</summary>`timeout` 이 안쪽 Future 를 drop 하면 다음 `.await` 지점에서부터 더 이상 진행되지 않습니다(취소). `std::async` 의 future 는 취소 수단이 없고 소멸자가 작업 종료를 기다립니다.</details>
5. 동시 실행 결과를 입력 순서로 돌려주려면 `JoinSet` 에서 무엇을 해야 하나요?
   <details><summary>답</summary>`join_next()` 는 끝난 순서로 나오므로, 태스크가 `(인덱스, 결과)` 를 돌려주게 하고 모은 뒤 인덱스로 정렬합니다.</details>

## 📚 더 알아보기

- The Rust Book: [Ch17 Async and Await](https://doc.rust-lang.org/book/ch17-00-async-await.html)
- [Tokio 튜토리얼](https://tokio.rs/tokio/tutorial) — `spawn`, `select!`, 채널
- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/) (async-book)
- `tokio::time` / `tokio::task::JoinSet` / `tokio::select!` 의 API 문서 (docs.rs/tokio)
- 비교: Python 공식 문서 [asyncio — Coroutines and Tasks](https://docs.python.org/3/library/asyncio-task.html)
