# Ch13. 동시성 (Concurrency)

> 스레드·`Arc<Mutex<T>>`·채널·`thread::scope` — 그리고 "데이터 레이스가 컴파일 에러"인 이유(`Send`/`Sync`)  ·  예상 소요: 이론 50분 / 실습 70분  ·  선행: Ch04(빌림), Ch07(클로저), Ch11(`Rc`/`RefCell`)

## 🎯 학습 목표

- `thread::spawn` / `JoinHandle::join` / `move` 클로저로 스레드를 만들고 결과를 돌려받는다.
- `thread::scope` 로 `Arc` 없이 지역 데이터를 병렬 처리한다.
- `Arc<Mutex<T>>`, `RwLock<T>`, `mpsc` 채널의 용도를 구분해 쓴다.
- `Send` / `Sync` 가 무엇이고, 왜 `Rc` 와 `RefCell` 이 스레드에서 막히는지 설명한다. (C++ 의 데이터 레이스 UB 와 대비)
- AI 가 만든 동시성 코드(`Arc<Mutex<..>>` 남발, 락을 쥔 채 오래 일하기, 채널 drop 누락)를 리뷰한다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust (std) |
|---|---|---|---|
| 스레드 만들기 | `std::thread` / `std::jthread` | `threading.Thread` | `thread::spawn(move \|\| ..)` |
| 결과 받기 | `std::future` / 공유 변수 | `Thread` 는 반환값 없음 (`concurrent.futures`) | `handle.join()` 이 반환값(`Result`) |
| 지역 변수 빌려 쓰기 | `[&]` 캡처 + join 약속 | 클로저가 그냥 참조 | `thread::scope` (컴파일러가 보장) |
| 뮤텍스 | `std::mutex` + `lock_guard` (**데이터와 분리**) | `threading.Lock` | `Mutex<T>` (**데이터를 품음**) |
| 읽기/쓰기 락 | `std::shared_mutex` | (stdlib 없음) | `RwLock<T>` |
| 공유 소유 | `std::shared_ptr` | 참조 | `Arc<T>` |
| 채널 | (표준 없음; 직접 구현) | `queue.Queue` | `mpsc::channel` |
| 원자 변수 | `std::atomic<T>` | (GIL 에 기대는 경우 많음) | `AtomicUsize` 등 |
| 데이터 레이스 | **컴파일됨, 실행 시 UB** | GIL 덕에 덜하지만 논리적 레이스는 존재 | **컴파일 에러** (`Send`/`Sync`) |
| 병렬 CPU 계산 | 스레드 | `multiprocessing` (GIL 때문) | 스레드 (진짜 병렬) |

> 이 README 의 코드 조각은 표준 라이브러리만 쓰므로 <https://play.rust-lang.org> 에 그대로 붙여 넣어 실행할 수 있습니다.

> 💡 **쉬운 설명 — 동시성과 병렬성**: *동시성(concurrency)* 은 "여러 일을 번갈아 진행할 수 있게 구조를 짜는 것", *병렬성(parallelism)* 은 "실제로 같은 순간에 여러 코어가 일하는 것"입니다. 스레드는 둘 다에 쓰입니다.
> *데이터 레이스* 는 "두 스레드가 같은 변수를 동시에 (하나 이상이 쓰기로) 건드리는 것"인데, 결과가 예측 불가능해지는 가장 고약한 버그예요.

## 📘 이론

### 1. 스레드 만들기: `spawn`, `join`, `move`

```rust
use std::thread;

let handle = thread::spawn(|| {
    println!("worker running");
    42                                  // 클로저의 반환값이 곧 스레드의 반환값
});
let answer = handle.join().unwrap();    // 스레드가 끝날 때까지 기다리고 값을 받음
println!("{answer}");                   // 42
```

- `join()` 은 `Result<T, Box<dyn Any + Send>>` 입니다. 스레드가 panic 하면 `Err`. (다른 스레드의 panic 이 main 을 죽이지 않아요.)
- **`JoinHandle` 을 버리면 스레드는 분리(detach)되고**, main 이 끝나면 프로세스와 함께 죽습니다. 기다려야 할 스레드는 꼭 `join`.
- C++ `std::jthread`(소멸자에서 자동 join)와 달리 Rust 의 `JoinHandle` 은 drop 시 join 하지 않습니다.

#### `move` 클로저가 필요한 이유

```rust
let v = vec![1, 2, 3];
let h = thread::spawn(|| println!("{v:?}"));    // ❌ E0373: closure may outlive the current function
let h = thread::spawn(move || println!("{v:?}")); // ✅ v 의 소유권이 스레드로 이동
```

스레드는 만든 함수보다 오래 살 수 있습니다. 클로저가 `v` 를 *빌려* 가면 `v` 가 먼저 사라질 수 있으니 컴파일러가 막습니다. `move` 는 클로저가 값을 **소유**하게 만들어서 이 문제를 없앱니다.

⚠️ **C++ 개발자가 헷갈리는 점**: C++ 에서 `std::thread t([&]{ use(local); }); t.detach();` 는 컴파일되고, `local` 이 먼저 사라지면 *dangling reference UB* 입니다. Rust 는 컴파일 타임에 막아요.
⚠️ `Copy` 타입(`i32` 등)에 `move` 를 쓰면 **복사본**이 스레드로 갑니다. 원본을 고치려는 의도라면 조용히 틀립니다. (🔬 실험 5)

### 2. `thread::scope` — 빌려 쓰는 스레드

```rust
let data = vec![1, 2, 3, 4, 5, 6];
let (left, right) = data.split_at(3);
let (a, b) = thread::scope(|s| {
    let h1 = s.spawn(|| left.iter().sum::<i32>());   // data 를 그냥 빌림. Arc 도, move 도 필요 없음
    let h2 = s.spawn(|| right.iter().sum::<i32>());
    (h1.join().unwrap(), h2.join().unwrap())
});
println!("{a} {b}");   // 6 15
```

`scope` 가 끝나기 전에 안에서 만든 스레드가 **모두 join 됨을 컴파일러가 알기** 때문에 `'static` 이 아닌 참조를 써도 됩니다. 입력을 조각내서 병렬 처리하는 "fork-join" 형태엔 이게 가장 간단해요. 쓰기가 겹치면 여전히 에러입니다 (🔬 실험 2, `E0499`).

> 💡 **쉬운 설명**: `spawn` 은 "친구에게 일을 맡기고 나는 갈 길 간다"(친구에게 물건을 *줘야* 함 = move/Arc). `scope` 는 "여기서 같이 일하고 문 닫기 전에 모두 모인다"(물건을 *빌려줘도* 됨).

### 3. 가장 중요한 두 트레이트: `Send` 와 `Sync`

| 트레이트 | 의미 | 예 |
|---|---|---|
| `Send` | 이 타입의 **값을 다른 스레드로 보내도(이동해도)** 안전 | `i32`, `String`, `Vec<T>`, `Arc<T>`(T: Send+Sync), `Mutex<T>` |
| `Sync` | 이 타입의 **`&T` 를 여러 스레드가 동시에 가져도** 안전 | `i32`, `Mutex<T>`, `Arc<T>` |
| `Send` 아님 (`Sync` 도 아님) | | `Rc<T>` (카운트가 atomic 이 아님), 원시 포인터 |
| `Send` 지만 `Sync` 아님 | 값을 옮기는 것은 OK, 동시에 `&` 공유는 위험 | `RefCell<T>`, `Cell<T>` |

- 컴파일러가 **자동으로** 판정합니다. (모든 필드가 `Send` 면 구조체도 `Send`.) 직접 구현할 일은 거의 없고, 하는 순간 `unsafe` 입니다.
- `thread::spawn` 의 시그니처가 핵심: `F: FnOnce() -> T + Send + 'static`, `T: Send + 'static`. 클로저가 캡처한 값이 전부 `Send` 여야만 컴파일됩니다.
- 그래서 `Rc` 를 스레드로 보내면 *아주 이해하기 쉬운 에러*가 납니다 (🔬 실험 3):

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
  = help: within `{closure}`, the trait `Send` is not implemented for `Rc<i32>`
```

**C++ 와의 대비.** C++ 에서는 `std::shared_ptr` 의 카운트가 항상 atomic 이고(느려도 안전), 일반 변수를 락 없이 두 스레드가 고치면 **컴파일도 되고 실행도 되지만 결과는 UB** 입니다. 증상은 "가끔만 틀림, 디버거 붙이면 사라짐". 도구(ThreadSanitizer)로 *실행 중에* 잡아야 해요. Rust 는 같은 실수를 **컴파일 에러**로 바꿉니다. 그 대가는 "스레드 간 공유 설계를 타입으로 드러내야 한다" 는 것이죠.

**Python 의 GIL.** CPython 은 한 번에 한 스레드만 바이트코드를 실행하는 GIL 이 있어서 `threading` 은 I/O 대기에 유효하지만 CPU 작업을 코어에 나누지는 못합니다(그래서 `multiprocessing`). 반대로 GIL 덕분에 일부 연산이 우연히 안전해 보여서 `counter += 1` 레이스를 놓치기 쉽습니다(실제로는 read→add→write 3단계라 안전하지 않음). Rust 스레드는 OS 스레드 그대로 **진짜 병렬**이고, 안전은 GIL 이 아니라 타입 시스템이 보장합니다.

> 💡 **쉬운 설명 — Send / Sync 를 공 비유로**: `Send` = "이 공을 친구에게 던져 줘도 된다(이제 내 공이 아님)". `Sync` = "이 공을 여럿이 동시에 *쳐다봐도* 된다".
> `Rc` 는 "공 옆의 개수판을 연필로 고치는 공" 이라 던지면 두 사람이 동시에 지워 쓸 수 있어 위험하고, `Arc` 는 "개수판이 자물쇠 달린(atomic) 공" 이라 안전해요.

### 4. `Arc<Mutex<T>>` — 공유하고 수정하기

```rust
use std::sync::{Arc, Mutex};
let counter = Arc::new(Mutex::new(0));
let handles: Vec<_> = (0..4).map(|_| {
    let counter = Arc::clone(&counter);        // 스레드마다 Arc 하나 (카운트 +1)
    thread::spawn(move || {
        for _ in 0..1000 { *counter.lock().unwrap() += 1; }
    })
}).collect();
for h in handles { h.join().unwrap(); }
println!("{}", *counter.lock().unwrap());      // 4000
```

- `Arc<T>` = **A**tomic **R**eference **C**ounted. `Rc` 와 API 가 같고 카운트가 atomic 이라 `Send`. (Ch11 의 `Rc` ↔ `Arc`)
- `Mutex<T>` 는 **데이터를 안에 품습니다.** `lock()` 으로 얻은 `MutexGuard` 를 통해서만 접근 가능 → "락을 안 잡고 접근" 이 *불가능*. (C++ 은 mutex 와 데이터가 따로라 실수 가능)
- `MutexGuard` 가 drop 될 때 자동 unlock (RAII, C++ `lock_guard` 와 같음). 보통 임시값 한 문장(`*m.lock().unwrap() += 1;`) 으로 씁니다.
- **poisoning**: 락을 쥔 스레드가 panic 하면 Mutex 는 "오염(poisoned)" 상태가 되고 이후 `lock()` 은 `Err` 를 돌려줍니다. `unwrap()` 은 "오염되면 나도 panic" 이라는 선택이에요.

⚠️ **자주 하는 실수들**
- **락을 쥔 채 오래 일하기**: 락은 다른 스레드를 막습니다. 가드를 필요한 만큼만 잡고, 계산은 락 밖에서 하세요.
  ```rust
  let item = queue.lock().unwrap().pop();      // 한 문장으로 끝 → 락 해제
  do_heavy(item);                              // 락 밖에서 일하기
  // ❌ while let Some(item) = queue.lock().unwrap().pop() { do_heavy(item); }   // 일하는 동안 락 유지!
  ```
- **데드락**: 락 A→B 순서와 B→A 순서로 잡는 두 스레드는 서로를 영원히 기다립니다. Rust 도 못 막아요(안전하지만 멈춤). 락 순서를 고정하세요.
- **단순 카운터/플래그**는 `Mutex<usize>` 보다 `AtomicUsize` / `AtomicBool` 이 가볍습니다 (`fetch_add(1, Ordering::Relaxed)`).
- 락을 쥔 채 `.await` 하면 문제가 됩니다 (Ch14).

### 5. `RwLock<T>` — 읽기는 여럿, 쓰기는 하나

```rust
use std::sync::RwLock;
let cfg = RwLock::new(String::from("v1"));
{ let r1 = cfg.read().unwrap(); let r2 = cfg.read().unwrap(); println!("{r1} {r2}"); } // 읽기는 동시에 OK
*cfg.write().unwrap() = "v2".to_string();                                              // 쓰기는 혼자
```
Ch04 의 "`&T` 여러 개 XOR `&mut T` 하나" 규칙의 **런타임 버전**입니다. 읽기가 많은 설정/캐시에 적합해요. (C++ `shared_mutex`)

### 6. 채널 `mpsc` — 공유하지 말고 보내라

```rust
use std::sync::mpsc;
let (tx, rx) = mpsc::channel();
let tx2 = tx.clone();                         // mpsc = multiple producer, single consumer
thread::spawn(move || tx.send("hello").unwrap());
thread::spawn(move || tx2.send("world").unwrap());
for msg in rx { println!("{msg}"); }          // 모든 Sender 가 drop 되면 루프 종료
```

- `send` 는 값의 **소유권을 이동**합니다. 보낸 뒤 그 값을 쓰면 컴파일 에러(Ch03) → 공유 상태 없이 통신 → 락이 필요 없어요.
- 받는 쪽(`Receiver`)은 하나뿐입니다(복제 불가). 여러 워커가 하나의 작업 큐를 나눠야 하면 `Arc<Mutex<Receiver<T>>>` 로 감싸는 것이 표준 패턴 (이 장의 워커 풀).
- 채널 종료 규칙: **모든 `Sender` 가 drop 되면** `recv()` 는 `Err`, `for msg in rx` 는 끝납니다. 원본 `tx` 를 `drop` 하는 걸 잊으면 영원히 멈춥니다.
- 철학: *"메모리를 공유해서 통신하지 말고, 통신으로 메모리를 공유하라."* (Go 의 슬로건이지만 Rust 도 같은 선택지를 줍니다.)

| 상황 | 고르기 |
|---|---|
| 결과를 한 번 돌려받기 | `JoinHandle::join()` |
| 입력을 조각내서 병렬 처리 후 합치기 | `thread::scope` |
| 작업 큐/이벤트 스트림 | `mpsc::channel` |
| 여러 스레드가 같은 상태를 수정 | `Arc<Mutex<T>>` |
| 읽기 위주의 공유 설정 | `Arc<RwLock<T>>` 또는 불변 `Arc<T>` |
| 카운터, 종료 플래그 | `Atomic*` |

### 7. 이 장 바깥의 이야기 (맛보기)

- 데이터 병렬(`par_iter().map(..)`)은 `rayon` 크레이트가 표준입니다. 이 과정에서는 std 만 써요.
- 네트워크 I/O 처럼 "기다리는 시간"이 대부분인 일은 OS 스레드 대신 **async/await** (Ch14, Tokio) 가 훨씬 효율적입니다. 에이전트가 LLM API 수십 개를 동시에 부르는 경우가 바로 그렇죠.
- `Arc<Mutex<T>>` 가 코드 전체에 퍼져 있다면 설계를 의심해 보세요. 한 소유자 스레드가 상태를 갖고 나머지는 채널로 요청을 보내는 "액터" 형태가 더 단순한 경우가 많습니다.

## 🧯 자주 만나는 컴파일 에러

**1) E0373 — `move` 를 빠뜨림**
```text
error[E0373]: closure may outlive the current function, but it borrows `v`, which is owned by the current function
help: to force the closure to take ownership of `v` (and any other referenced variables), use the `move` keyword
```
의미: 스레드가 `v` 보다 오래 살 수 있음. 고치기: `move ||`, 또는 정말 빌리고 싶으면 `thread::scope`.

**2) E0277 — `Rc` 를 스레드로 보냄**
```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
  = help: within `{closure}`, the trait `Send` is not implemented for `Rc<i32>`
```
의미: `Rc` 는 `Send` 가 아님. 고치기: `Arc`.

**3) E0277 — `Arc<RefCell<T>>` (Sync 아님)**
```text
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
  = help: the trait `Sync` is not implemented for `RefCell<i32>`
  = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
```
의미: `Arc` 로 감싸도 안의 타입이 `Sync` 여야 함. 고치기: `Arc<Mutex<T>>` / `Arc<RwLock<T>>`.

**4) E0382 — `Mutex` 를 두 스레드에 move**
```text
error[E0382]: use of moved value: `m`
  move occurs because `m` has type `std::sync::Mutex<i32>`, which does not implement the `Copy` trait
```
의미: 소유자가 하나뿐. 고치기: `Arc::new(Mutex::new(..))` 후 스레드마다 `Arc::clone`.

**5) E0499 — 스코프 스레드 두 개가 같은 변수를 가변으로 빌림**
```text
error[E0499]: cannot borrow `count` as mutable more than once at a time
```
의미: 쓰기 접근이 겹침. 고치기: `Mutex` / `AtomicUsize`, 또는 스레드별 결과를 따로 받아 합치기.

**6) E0594 — `Arc` 안의 값을 직접 수정**
```text
error[E0594]: cannot assign to data in an `Arc`
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Arc<i32>`
```
의미: `Arc<T>` 는 읽기 전용 공유. 고치기: `Arc<Mutex<T>>` / atomic.

## 🛠️ 실습: 워커 풀 + 병렬 도구들

### 무엇을 만드나요

여섯 가지 동시성 도구를 차례로 연습하고, 마지막에 **워커 풀**을 만듭니다. 워커 4개가 작업 채널에서 "구간 소수 개수 세기" 일을 꺼내 처리하고 결과 채널로 보냅니다. 스레드 스케줄링이 매번 달라도 출력은 **항상 같도록** 결과를 입력 순서/정렬로 모읍니다.

기대 출력(일부):
```text
== 1. spawn + join ==
squares: [1, 4, 9, 16, 25, 36, 49, 64]
== 2. scoped threads ==
chunk sums: [325, 950, 1575, 2200]
total: 5050
== 3. Arc<Mutex> ==
counter: 8000 (8 threads x 1000)
== 4. channel ==
  producer 0 / msg 0
  ...
== 5. RwLock ==
written: [0, 10, 20, 30]
readers saw: [60, 60, 60]
== 6. worker pool ==
  job 0: [0, 10000) -> 1229 primes
  ...
total primes below 100000: 9592
```

### 파일 구성

| 파일 | 설명 |
|---|---|
| `python/main.py` | `threading` + `queue` 버전 (GIL 설명 주석) |
| `cpp/main.cpp` | `std::thread`/`jthread`/`mutex`/`shared_mutex` + 직접 만든 `Channel<T>`. `cpp/BUILD_FLAGS` 에 `-pthread` |
| `rust/src/main.rs` | **실습 파일** (TODO 7개, 🔬 실험 6개) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법 (모두 이 챕터 폴더에서)

```bash
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x -pthread && /tmp/x
cargo run -q -p ch13_concurrency                      # 실습
cargo test -p ch13_concurrency                        # 실습 테스트 (워커 풀의 drop 을 빠뜨리면 여기서 멈춰요!)
cargo run -q -p ch13_concurrency --example solution   # 정답
../../scripts/compare.sh ch13                         # 3개 언어 비교
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `count_primes` | 범위 이터레이터 (워커 풀의 재료) |
| 2 | `parallel_squares` | `thread::spawn`, `move`, `JoinHandle`, `join` |
| 3 | `scoped_chunk_sums` | `thread::scope`, `chunks`, 빌려 쓰기 |
| 4 | `counter_with_mutex` | `Arc<Mutex<T>>`, `lock()`, 가드 |
| 5 | `collect_messages` | `mpsc::channel`, `Sender::clone`, `drop(tx)` |
| 6 | `rwlock_demo` | `Arc<RwLock<T>>`, `read()` / `write()` |
| 7 | `worker_pool` | 작업/결과 채널 2개, `Arc<Mutex<Receiver>>`, 종료 신호 |

### 완료 기준
- `cargo test -p ch13_concurrency` 모두 통과 (몇 번 반복해도 항상 통과: `for i in 1 2 3 4 5; do cargo test -q -p ch13_concurrency; done`)
- `../../scripts/compare.sh ch13` 에서 Python / C++ / Rust 출력이 똑같음

### 🔬 실험 목록

| # | 위치 | 보게 될 것 |
|---|---|---|
| 1 | TODO(2) 주석 `lab_forgot_move` | `E0373` — `move` 없는 클로저 |
| 2 | TODO(3) 주석 `lab_shared_mut_in_scope` | `E0499` — 두 스레드가 같은 변수 가변 접근 (C++ 이면 UB) |
| 3 | TODO(4) 주석 `lab_rc_in_thread` | `E0277` `Rc<i32>` cannot be sent between threads safely |
| 4 | `lab_mutex_moved_twice` | `E0382` — 해결책이 곧 `Arc` |
| 5 | `lab_move_copies` | 컴파일은 되지만 `count = 0` (복사본을 고침) |
| 6 | `lab_refcell_in_arc` | `E0277` `Sync` 가 아님 → `Mutex` 로 교체 |

실험이 끝나면 꼭 다시 주석 처리하세요. 에러가 나는 코드가 남아 있으면 나머지 TODO 를 컴파일할 수 없어요.

## 🤖 바이브 코딩 포인트

- AI 가 `Arc<Mutex<..>>` 를 **모든 곳에** 두르면 의심하세요. 정말 공유 상태가 필요한지, 채널/`scope`/소유권 이동으로 풀 수 없는지 물어보세요. 프롬프트: *"공유 상태를 최소화하고, 가능하면 채널이나 thread::scope 로 다시 설계해 줘."*
- **락을 쥔 채 긴 작업**(네트워크 호출, 파일 I/O, `sleep`)이 있는지 확인하세요. `let x = m.lock().unwrap().clone();` 처럼 값을 꺼내고 락을 놓은 뒤 일하는 것이 기본입니다.
- 락이 여러 개면 **획득 순서**가 일관된지 보세요 (데드락). AI 는 이를 자주 놓칩니다.
- 채널 코드에서는 **원본 `Sender` 를 drop 했는지**, 워커가 `recv()` 에러로 종료하는지 확인하세요. 안 그러면 프로그램이 종료되지 않습니다.
- `.lock().unwrap()` 이 흩어져 있으면 poisoning 정책(panic 전파 vs 복구)을 한 군데에서 정하도록 요청하세요.
- `thread::spawn` 을 루프에서 수천 번 부르는 코드는 스레드 폭증입니다. 풀(워커 N개)로 바꾸거나, I/O 위주라면 async(Ch14)로 바꾸도록 요청하세요.
- 에러 `E0277 ... cannot be sent between threads safely` 를 만나면 AI 에게 "**Rc→Arc, RefCell→Mutex**" 처럼 *짝으로* 바꾸라고 알려 주세요. 한쪽만 바꾸면 다음 에러를 만납니다.

## ✅ 셀프 체크

1. `thread::spawn(|| println!("{v:?}"))` 가 컴파일 에러인 이유와 두 가지 해결책은?
<details><summary>답</summary>스레드가 `v` 를 빌렸는데 스레드가 `v` 의 주인보다 오래 살 수 있어서 `E0373`. 해결: `move` 로 소유권을 넘기거나, 스레드가 반드시 먼저 끝나는 `thread::scope` 를 쓰기.</details>

2. `Rc` 는 왜 `Send` 가 아니고, `Arc` 는 왜 되나요?
<details><summary>답</summary>`Rc` 의 참조 카운트는 atomic 이 아니라서 두 스레드가 동시에 증감하면 카운트가 틀어져 use-after-free 나 누수가 생깁니다. `Arc` 는 atomic 연산으로 카운트를 관리하므로 안전하지만 그만큼 약간 느립니다.</details>

3. C++ 의 `std::mutex` 와 Rust 의 `Mutex<T>` 의 가장 큰 차이는?
<details><summary>답</summary>Rust 의 Mutex 는 보호할 데이터를 안에 품고 있어서 락을 잡지 않고는 데이터에 접근할 방법이 없습니다. C++ 은 뮤텍스와 데이터가 분리되어 있어 락을 잊어도 컴파일됩니다.</details>

4. `mpsc` 워커 풀에서 `drop(job_tx)` 를 빼먹으면 어떻게 되나요?
<details><summary>답</summary>워커들이 "더 올 작업이 있을지도 모른다" 며 `recv()` 에서 영원히 기다리므로 `join` 이 끝나지 않아 프로그램이 멈춥니다. 모든 `Sender` 가 drop 되어야 채널이 닫힙니다.</details>

5. 출력이 스레드 실행 순서와 무관하게 항상 같으려면 어떻게 모아야 하나요?
<details><summary>답</summary>`join` 을 spawn 순서대로 호출해 결과를 입력 순서로 모으거나, 채널로 받은 결과에 id 를 붙여 마지막에 정렬합니다. 도착 순서 자체에 의존하지 않는 것이 핵심이에요.</details>

## 📚 더 알아보기

- The Rust Book Ch16 (겁 없는 동시성): <https://doc.rust-lang.org/book/ch16-00-concurrency.html> — 스레드, 메시지 전달, `Mutex`/`Arc`, `Send`/`Sync`
- Rust by Example — Threads, Channels: <https://doc.rust-lang.org/rust-by-example/std_misc/threads.html>
- std 문서: [`std::thread`](https://doc.rust-lang.org/std/thread/), [`std::sync`](https://doc.rust-lang.org/std/sync/), [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/), [`std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/)
- Rustonomicon — Send and Sync: <https://doc.rust-lang.org/nomicon/send-and-sync.html>
- 다음 장: Ch14 async/await 와 Tokio
