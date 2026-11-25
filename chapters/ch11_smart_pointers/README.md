# Ch11. 스마트 포인터 · 내부 가변성

> `Box` · `Rc` · `Weak` · `RefCell` — C++ 의 `unique_ptr` / `shared_ptr` / `weak_ptr` 를 Rust 방식으로 다시 만나는 장  ·  예상 소요: 이론 40분 / 실습 60분  ·  선행: Ch08(트레이트), Ch09(라이프타임)

## 🎯 학습 목표

- `Box<T>` 가 언제 필요한지(재귀 타입, `dyn Trait`, 큰 값 이동) 설명하고 재귀 `enum` 을 만들 수 있다.
- `Rc<T>` / `Weak<T>` 의 참조 카운트(`strong_count`, `weak_count`)를 읽고, 순환 참조가 왜 **메모리 누수**인지 설명한다.
- `RefCell<T>` 가 "빌림 규칙을 컴파일 타임이 아닌 **런타임**에 검사한다"는 뜻을 이해하고, 위반하면 어떤 panic 이 나는지 안다.
- `Rc<RefCell<T>>` 를 **읽고** 이해하되, 언제 쓰지 말아야 하는지(arena/인덱스, 소유권 재설계) 판단한다. (AI 가 생성한 코드 리뷰의 핵심!)

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 힙에 단독 소유 | `std::unique_ptr<T>` | (모든 객체가 힙) | `Box<T>` |
| 공유 소유 (단일 스레드) | `std::shared_ptr<T>` (카운트가 항상 atomic) | 평범한 참조 | `Rc<T>` (atomic 아님, 더 빠름) |
| 공유 소유 (멀티 스레드) | `std::shared_ptr<T>` | 평범한 참조 | `Arc<T>` (Ch13) |
| 약한 참조 | `std::weak_ptr<T>` + `lock()` | `weakref.ref` + 호출 | `Weak<T>` + `upgrade()` → `Option<Rc<T>>` |
| 공유 객체 수정 | 그냥 가능 (`const` 만 지키면 됨) | 그냥 가능 | `Rc<RefCell<T>>` — 런타임 검사 |
| 규칙 위반 시 | UB (조용히 망가짐) | 해당 없음 | **panic** (`RefCell already borrowed`) |
| 순환 참조 | 누수 (`weak_ptr` 로 해결) | 순환 GC 가 회수 | 누수 (`Weak` 로 해결) — GC 없음 |
| 사용자 정의 포인터 | `operator*`, `operator->` | `__getattr__` 등 | `Deref` / `DerefMut` 트레이트 |

> 💡 **쉬운 설명 — 스마트 포인터란?** 그냥 "주소"가 아니라 *주소 + 추가 규칙(자동 해제, 카운트, 검사 …)* 을 묶어 둔 값입니다.
> 예전 이름은 "포인터처럼 쓰는 객체". C++ 에서 `unique_ptr` 을 써 봤다면 이미 절반은 아는 셈이에요.

> 이 README 의 코드 조각은 표준 라이브러리만 쓰므로 <https://play.rust-lang.org> 에 그대로 붙여 넣어 실행할 수 있습니다.

## 📘 이론

### 1. `Box<T>` — 힙에 놓는 단독 소유자 (`unique_ptr`)

```rust
let b = Box::new(5);          // 힙에 5 를 놓고 b 가 유일한 주인
println!("{}", *b + 1);       // 6  (역참조는 C++ 과 같은 `*`)
// 스코프를 벗어나면 자동으로 해제 (RAII) — delete 를 쓸 일이 없습니다.
```

`Box` 가 꼭 필요한 순간은 크게 세 가지입니다.

1. **재귀 타입** — 크기를 컴파일 타임에 알 수 없는 타입을 "고정 크기 포인터"로 감싸기
2. **트레이트 객체** `Box<dyn Trait>` (Ch08에서 이미 사용)
3. 아주 큰 값을 스택에서 힙으로 옮겨서 이동 비용을 줄이기

```rust
enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),   // Expr 안에 Expr 를 "직접" 넣을 수 없으니 Box 로 간접 참조
    Neg(Box<Expr>),
}
```

> 💡 **쉬운 설명 — 왜 직접 못 넣나요?** `Expr` 안에 `Expr` 가 있고, 그 안에 또 `Expr` 가 있고… 크기를 계산하면 무한대가 됩니다.
> 포인터(`Box`)는 항상 주소 한 칸 크기라서 "다음 칸은 저기 있어요"라고만 적어 두면 끝나요.

**`Box` 만의 특권:** 안의 값을 밖으로 *꺼내서(move)* 가질 수 있습니다. 유일한 주인이라 안전하기 때문이에요.

```rust
fn unwrap_neg(e: Expr) -> Expr {
    match e {
        Expr::Neg(inner) => *inner,   // Box 안의 값을 move 로 꺼냄
        other => other,
    }
}
```

⚠️ **C++ 개발자가 헷갈리는 점**
- `Box<T>` 는 `unique_ptr` 와 달리 **null 이 될 수 없습니다**. "없을 수도 있음"은 `Option<Box<T>>` 로 표현합니다. (`Option<Box<T>>` 는 null 포인터 최적화 덕분에 크기가 포인터와 같아요.)
- "moved-from 상태의 빈 `unique_ptr`" 같은 것이 없습니다. move 한 변수는 컴파일러가 아예 못 쓰게 막아요. (Ch03)

### 2. `Deref` — 스마트 포인터가 `&T` 처럼 행동하는 이유

```rust
fn shout(s: &str) { println!("{}", s.to_uppercase()); }
let boxed: Box<String> = Box::new(String::from("hi"));
shout(&boxed);   // &Box<String> -> &String -> &str 로 자동 변환 (Deref 강제 변환)
```

`Box`, `Rc`, `String`, `Vec` 모두 `Deref` 를 구현해서 `&Box<T>` 가 필요한 곳에 `&T` 를 넘길 수 있고, 메서드도 `.` 으로 바로 호출됩니다. C++ 의 `operator->` 와 비슷하지만 **컴파일러가 필요한 만큼 자동으로 여러 번 적용**해 줍니다.

### 3. `Rc<T>` — 단일 스레드 공유 소유 (`shared_ptr`)

```rust
use std::rc::Rc;
let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a);               // 데이터 복사 X, 카운트만 +1
println!("{}", Rc::strong_count(&a)); // 2
{ let _c = Rc::clone(&a); println!("{}", Rc::strong_count(&a)); } // 3
println!("{}", Rc::strong_count(&a)); // 다시 2
```

- `Rc::clone(&a)` 와 `a.clone()` 은 같은 일을 하지만, 관례적으로 **`Rc::clone(&a)`** 라고 씁니다. "깊은 복사가 아니라 카운트 증가"임이 눈에 띄기 때문이에요. 코드 리뷰에서 읽기 쉽습니다.
- `Rc` 안의 값은 **읽기 전용**입니다. 여러 주인이 동시에 쥐고 있으니 `&mut` 를 줄 수 없어요 (🔬 실험 3).
- `Rc` 는 **`Send` 가 아닙니다** — 다른 스레드로 못 보냅니다. 카운트가 atomic 이 아니기 때문. 스레드에서는 `Arc` (Ch13).

⚠️ **C++ 개발자가 헷갈리는 점**
- `shared_ptr` 는 `make_shared` 한 번이면 끝이지만, Rust 의 `Rc` 는 *읽기 전용 공유* 가 기본값입니다. 수정이 필요하면 `RefCell`/`Cell` 을 안에 넣어야 해요.
- C++ 에서는 `shared_ptr<T>` 를 습관처럼 쓰는 코드가 흔하지만, Rust 에서는 대부분 **그냥 소유권 이동이나 `&` 빌림**으로 해결됩니다. `Rc` 는 "정말로 소유자가 여럿"일 때만 쓰세요.

### 4. `Weak<T>` — 순환을 끊는 약한 참조 (`weak_ptr`)

```rust
use std::rc::{Rc, Weak};
let strong = Rc::new(5);
let weak: Weak<i32> = Rc::downgrade(&strong);
println!("{:?}", weak.upgrade());   // Some(5)  — 아직 살아 있음
drop(strong);
println!("{:?}", weak.upgrade());   // None     — 이미 해제됨
```

- `Rc::downgrade(&rc)` → `Weak`, `weak.upgrade()` → `Option<Rc<T>>` (C++ `weak_ptr::lock()` 와 같음)
- 카운트는 **두 개**: `strong_count` 가 0 이 되면 값이 해제되고, `weak_count` 는 "나를 지켜보는 약한 참조 수"일 뿐 수명에 영향이 없습니다.

> 💡 **쉬운 설명 — 강한 참조 vs 약한 참조** 강한 참조는 "이 물건이 필요해요(버리지 마세요)", 약한 참조는 "있으면 볼게요, 없어져도 괜찮아요".
> 부모 ↔ 자식처럼 서로를 가리킬 때는 **한쪽(보통 자식 → 부모)만 약하게** 만들어야 합니다.

### 5. 순환 참조는 누수다 (그리고 Rust 는 막아 주지 않는다)

```rust
// a.next -> b, b.next -> a   (둘 다 Rc)
// 변수 a, b 를 버려도: a 의 카운트는 2 -> 1, b 도 2 -> 1.  0 이 아니므로 영원히 남음!
```

| 단계 | a strong | b strong | 비고 |
|---|---|---|---|
| 순환 연결 직후 | 2 (변수 a + b.other) | 2 (변수 b + a.other) | |
| `drop(a); drop(b);` 후 | 1 (b.other) | 1 (a.other) | 서로 붙잡고 있어 0 이 되지 못함 → **누수** |
| `b -> a` 를 `Weak` 로 바꾸면 | 1 (변수 a 만) | 2 (변수 b + a.other) | drop 후 a → 0 → a 해제 → b 도 0 → 모두 해제 |

- Python 은 별도의 **순환 GC** 가 돌아서 회수해 주고, C++ `shared_ptr` 와 Rust `Rc` 는 **그렇지 않습니다.**
- 이게 가능한 이유: 메모리 누수는 "메모리 안전성" 위반이 아니라서 safe Rust 에서도 허용됩니다. Rust 의 약속은 *use-after-free 와 data race 없음* 이지 *누수 없음* 이 아니에요.

### 6. `RefCell<T>` — 빌림 규칙을 런타임에 검사하는 상자

Rust 의 기본 규칙은 "읽기 여러 개 **XOR** 쓰기 하나"(Ch04)이고 컴파일러가 확인합니다. 그런데 `Rc` 처럼 컴파일러가 증명하기 어려운 구조에서는 그 검사를 **실행 시점으로 미루는** 도구가 `RefCell` 입니다.

```rust
use std::cell::RefCell;
let c = RefCell::new(vec![1, 2, 3]);
c.borrow_mut().push(4);                    // `&c` (공유 참조) 인데도 수정 가능 = 내부 가변성
println!("{:?}", c.borrow());              // [1, 2, 3, 4]

let r1 = c.borrow();                       // 읽기 빌림 중...
let w = c.try_borrow_mut();                // 안전하게 시도 -> Err(BorrowMutError)
assert!(w.is_err());
// let w2 = c.borrow_mut();                // 이건 panic: "RefCell already borrowed"
```

| 호출 | 설명 | 규칙 위반 시 |
|---|---|---|
| `borrow()` | 읽기 빌림 (여러 개 가능) | 쓰기 빌림이 살아 있으면 panic (`already mutably borrowed`) |
| `borrow_mut()` | 쓰기 빌림 (하나만) | 읽기/쓰기 빌림이 살아 있으면 panic (`already borrowed`) |
| `try_borrow()` / `try_borrow_mut()` | 실패를 `Result` 로 돌려줌 | panic 대신 `Err` |

> 💡 **쉬운 설명 — 도서관 비유**: 책(데이터)을 여러 사람이 *읽는* 건 동시에 가능하지만, 누가 *고치는* 중이면 다른 사람은 못 읽습니다.
> 보통은 사서(컴파일러)가 입구에서 미리 막아 줍니다. `RefCell` 은 "입구 검사는 생략할 테니 책을 펼 때 그때그때 확인할게요" 라는 방식이에요. 그래서 걸리면 **실행 중에 panic** 입니다.

`borrow()` 가 돌려주는 `Ref<T>` / `RefMut<T>` 는 **가드(guard)** 입니다. 변수에 담아 두면 스코프 끝까지 빌림이 유지됩니다.

```rust
let first = cell.borrow_mut();   // 가드가 스코프 끝까지 살아 있음
let second = cell.borrow_mut();  // 💥 panic
```
임시값으로 쓰면 그 문장이 끝날 때 바로 풀립니다: `cell.borrow_mut().push(4);` 는 안전.

**`Cell<T>`**: `Copy` 타입(숫자, bool 등)을 위한 더 가벼운 내부 가변성. 빌림 없이 값을 통째로 `get()` / `set()` 합니다. 검사 비용도, panic 도 없어요.

```rust
use std::cell::Cell;
struct Counter { hits: Cell<u32> }
impl Counter { fn hit(&self) { self.hits.set(self.hits.get() + 1); } }   // &self 인데 수정!
```

⚠️ **C++ 개발자가 헷갈리는 점**
- C++ 의 `mutable` 멤버와 비슷한 용도지만, 훨씬 명시적이고 **위반을 잡아 줍니다**. (`const` 객체의 `mutable` 멤버를 두 곳에서 동시에 고치면 그냥 UB)
- `RefCell` 은 **Sync 가 아닙니다**. 멀티스레드용은 `Mutex`/`RwLock` (Ch13) — 똑같은 "런타임 검사" 아이디어지만 기다려 주는 버전이에요.

### 7. `Rc<RefCell<T>>` — 읽을 줄은 알되, 아껴 쓰기

두 가지를 합치면 "여럿이 공유하고, 아무나 수정 가능한" 객체가 됩니다. 연결 리스트, 그래프, 부모 포인터가 있는 트리에서 등장해요.

```rust
type NodeRef = Rc<RefCell<Node>>;
struct Node {
    text: String,
    parent: Weak<RefCell<Node>>,       // 부모는 약하게
    children: Vec<NodeRef>,            // 자식은 강하게
}
```

#### 언제 **쓰지 말아야** 하나요 — AI 가 만든 코드를 리뷰할 때

AI 코드 생성기는 빌림 검사기에 막히면 `Rc<RefCell<..>>` 로 "일단 컴파일되게" 만드는 경향이 있습니다. 컴파일은 되지만 **런타임 panic 과 설계 문제를 숨기는** 경우가 많아요. 아래 순서로 먼저 의심하세요.

| 상황 | `Rc<RefCell<T>>` 대신 이렇게 |
|---|---|
| 그냥 "소유자가 하나인데 여러 곳에서 읽고 싶다" | 소유자 하나 + `&T` 로 빌려 주기 |
| 부모/자식, 그래프 노드 | **arena + 인덱스**: `Vec<Node>` 에 두고 `children: Vec<usize>` / `parent: Option<usize>` |
| 함수 사이에서 수정 가능한 상태를 넘기고 싶다 | `&mut T` 를 인자로 넘기기 |
| 스레드 간 공유 | `Arc<Mutex<T>>` (Ch13) — `Rc` 는 컴파일 에러 |
| 여러 곳에서 "읽기만" 공유 | `Rc<T>` (RefCell 없이) 또는 `Arc<T>` |
| 이벤트 핸들러/콜백이 서로를 참조 | 채널(`mpsc`)로 메시지 전달, 소유권 구조 재설계 |

arena 로 바꾼 같은 트리는 이렇게 생겼습니다.

```rust
struct Tree { nodes: Vec<Node> }
struct Node { text: String, parent: Option<usize>, children: Vec<usize> }
impl Tree {
    fn add_child(&mut self, parent: usize, text: &str) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node { text: text.into(), parent: Some(parent), children: vec![] });
        self.nodes[parent].children.push(id);
        id
    }
}
```
- 카운트 없음, `RefCell` 없음, 런타임 panic 없음, 순환 누수 없음. 대신 인덱스를 직접 관리합니다. (C++ 의 `std::vector<Node>` + index 방식과 동일)
- 리뷰 체크: **`.borrow_mut()` 가 다른 `.borrow()` 가드가 살아 있는 구간(루프, 재귀, 콜백)에서 호출되지는 않는가?** 이게 바로 `already borrowed` panic 의 단골 원인입니다.

> 이 장의 실습은 `Rc<RefCell<>>` 를 **배우기 위해** 일부러 사용합니다. 실무에서는 위 표를 먼저 보세요.

## 🧯 자주 만나는 컴파일 에러

**1) E0072 — 재귀 타입에 Box 가 없음**
```text
error[E0072]: recursive type `Expr` has infinite size
 --> src/main.rs:3:1
  |
3 | enum Expr { Num(f64), Add(Expr, Expr) }
  | ^^^^^^^^^                 ---- recursive without indirection
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
```
의미: 크기가 무한. 고치는 법: 자식 자리에 `Box<Expr>`.

**2) E0596 — `Rc` 안의 값을 수정하려 함**
```text
error[E0596]: cannot borrow data in an `Rc` as mutable
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Vec<i32>>`
```
의미: `Rc` 는 읽기 전용 공유. 고치는 법: `Rc<RefCell<Vec<i32>>>` 로 감싸거나(내부 가변성), 정말 공유가 필요한지 재고하세요.

**3) E0507 — `Rc` 안의 값을 move 로 꺼내려 함**
```text
error[E0507]: cannot move out of an `Rc`
help: consider cloning the value if the performance cost is acceptable
```
의미: 주인이 여럿인 값은 한 명이 가져갈 수 없어요 (`Box` 는 가능). 고치는 법: `.clone()` 하거나 참조로 쓰기.

**4) E0277 — `Rc` 를 스레드로 보냄** (Ch13 에서 자세히)
```text
error[E0277]: `Rc<RefCell<Node>>` cannot be sent between threads safely
```

**5) 런타임 panic — `RefCell already borrowed`** (컴파일 에러가 아님!)
```text
thread 'main' panicked at src/main.rs:5:19:
RefCell already borrowed
```
의미: `RefCell` 규칙 위반. 고치는 법: 가드를 더 일찍 `drop` 하거나, 가드를 변수에 오래 담아 두지 말고 한 문장 안에서 쓰기. 위치가 어디인지 backtrace(`RUST_BACKTRACE=1`)로 확인.

## 🛠️ 실습: 수식 트리 + 대화 분기 트리

### 무엇을 만드나요

- **Part A**: `enum Expr { Num, Add, Mul, Neg }` 를 `Box` 로 재귀 정의하고 `eval` / `show` / `depth` / `simplify`(소유권을 받아 새 트리로 재조립).
- **Part B**: 고객센터 챗봇의 "대화 분기 트리". 노드는 `Rc<RefCell<Node>>`, 부모는 `Weak`. 루트→잎 경로 출력, `strong_count` 관찰, `visit` 는 `&NodeRef` 로 수정(내부 가변성).
- **순환 실험**: 강한 순환은 누수, `Weak` 로 끊으면 해제됨을 `probe`(약한 감시자)로 확인.

기대 출력(일부):
```text
== Part A: Expr ==
expr: ((2.0 + 3.0) * -(4.0 + 1.5))
value: -27.50
depth: 4
messy: ((1.0 * --7.0) + 0.0)
simplified: 7.0 (depth 1)
== Part B: dialogue tree ==
paths:
  Hello > Order status > Track package
  Hello > Order status > Cancel order
  Hello > Refund > Refund policy
  Hello > Chat
...
strong_count(root)=1 strong_count(order)=2
total visits: 4
== Cycle experiment ==
cycle: a strong=2 b strong=2
cycle leaked: yes
fixed: a strong=1 b strong=2
fixed leaked: no
```

### 파일 구성

| 파일 | 설명 |
|---|---|
| `python/main.py` | 참조 구현. `weakref`, `gc.disable()` 로 Rc 와 같은 조건을 만듦 |
| `cpp/main.cpp` | `unique_ptr` / `shared_ptr` / `weak_ptr` 버전 |
| `rust/src/main.rs` | **실습 파일** (TODO 11개) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법 (모두 이 챕터 폴더에서)

```bash
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch11_smart_pointers                      # 실습
cargo test -p ch11_smart_pointers                        # 실습 테스트
cargo run -q -p ch11_smart_pointers --example solution   # 정답
../../scripts/compare.sh ch11                            # 3개 언어 비교
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `eval` | `match`, `&Box<Expr>` Deref 강제 변환 |
| 2 | `show` | `format!`, 재귀 |
| 3 | `depth` | or-패턴 `A \| B` |
| 4 | `simplify` | `*box` 로 move out, 소유권 받는 재귀 |
| 5 | `add_child` | `Rc::clone`, `Rc::downgrade`, `borrow_mut` |
| 6 | `path_to_root` | `Weak::upgrade`, `while let` |
| 7 | `leaf_paths` | 재귀 + `borrow()` |
| 8 | `visit`, `total_visits` | 내부 가변성 |
| 9 | `try_borrow_while_reading` | `RefCell` 런타임 검사 |
| 10 | `demo_cycle_leak` | 강한 순환 = 누수 |
| 11 | `demo_weak_fixed` | `Weak` 로 순환 끊기 |

### 완료 기준
- `cargo test -p ch11_smart_pointers` 모두 통과
- `../../scripts/compare.sh ch11` 이 Python / C++ / Rust 출력 동일을 보고

### 🔬 실험 목록

| # | 위치 | 보게 될 것 |
|---|---|---|
| 1 | `enum Expr` 위 주석 | `E0072` 재귀 타입 무한 크기 → `Box` 의 필요성 |
| 2 | TODO(9) 근처 `lab_refcell_panic` | **런타임** panic `RefCell already borrowed` (컴파일은 통과!) |
| 3 | TODO(9) 아래 `lab_rc_mutation` | `E0596` `Rc` 는 읽기 전용 |
| 4 | TODO(10) 구현 후 | `cycle leaked: yes` — 변수를 버려도 `strong` 이 0 이 안 됨. 같은 코드에서 `Rc::downgrade` 로 바꾼 TODO(11) 와 비교 |

실험 4 를 더 파고들고 싶다면 `eprintln!` 로 `Rc::weak_count(&a)` 를 찍어 보세요. 약한 참조는 `strong_count` 를 올리지 않습니다.

## 🤖 바이브 코딩 포인트

- AI 코드에 `Rc<RefCell<T>>` / `Arc<Mutex<T>>` 가 **여기저기** 보이면 의심하세요. "빌림 검사기를 피하려고" 쓴 경우가 많습니다. 프롬프트: *"Rc<RefCell>을 쓰지 말고 소유권/인덱스(arena) 구조로 다시 설계해 줘."*
- `.borrow_mut()` 가 중첩되거나 루프 안/콜백 안에 있으면 `already borrowed` panic 후보입니다. 가드를 변수로 오래 들고 있는지 확인하세요. 테스트에서 잘 통과해도 특정 입력에서만 터질 수 있습니다.
- 부모 ↔ 자식 구조에서 **양쪽 모두 `Rc`** 이면 누수입니다. 반드시 한쪽은 `Weak` 여야 하고, `upgrade()` 의 `None` 처리(`if let Some(p)` / `?`)가 있는지 보세요.
- `.clone()` 이 `Rc` 에 대한 것인지(저렴한 카운트 증가) 데이터에 대한 것인지(깊은 복사) 변수 타입으로 확인하세요. 리뷰어는 `Rc::clone(&x)` 표기를 요구해도 좋습니다.
- `Box<dyn Trait>` 과 `Rc<dyn Trait>` 의 차이를 알고 요청하세요: 소유자가 하나면 `Box`, 여럿이면 `Rc`/`Arc`.
- `Rc` 가 있는 코드를 "스레드로 돌려 줘"라고 요청하면 `E0277` 이 납니다. `Arc` 로 바꾸면서 내부 `RefCell` → `Mutex` 도 같이 바꿔야 한다고 AI 에게 명시하세요. (Ch13)

## ✅ 셀프 체크

1. `Box<T>` 가 `Rc<T>` 와 달리 안의 값을 move 로 꺼낼 수 있는 이유는?
<details><summary>답</summary>`Box` 는 유일한 소유자라서, 꺼낸 뒤에 같은 값을 쓰는 다른 주인이 없기 때문입니다. `Rc` 는 주인이 여럿이라 한 명이 가져가면 나머지가 허공을 가리키게 돼요.</details>

2. 부모 ↔ 자식 둘 다 `Rc` 로 연결하면 어떤 일이 생기나요? 어떻게 고치나요?
<details><summary>답</summary>서로가 서로의 strong_count 를 1 씩 붙잡아서, 외부 변수를 다 버려도 0 이 되지 않아 영원히 해제되지 않습니다(누수). 자식 → 부모 방향을 `Weak` 로 바꾸세요.</details>

3. `RefCell` 은 컴파일 타임 검사를 없애 주나요?
<details><summary>답</summary>없애는 게 아니라 **런타임으로 미룹니다**. 규칙(읽기 여러 개 XOR 쓰기 하나)은 그대로이고, 위반하면 컴파일 에러 대신 `RefCell already borrowed` panic 이 납니다.</details>

4. `Rc<RefCell<Vec<i32>>>` 대신 쓸 수 있는 대안 두 가지를 말해 보세요.
<details><summary>답</summary>(1) 소유자를 하나로 두고 `&mut Vec<i32>` 를 인자로 전달 (2) 인덱스 기반 arena (`Vec<Node>` + `usize` id). 스레드 간 공유라면 `Arc<Mutex<..>>`.</details>

5. `Weak::upgrade()` 가 `Option` 을 돌려주는 이유는?
<details><summary>답</summary>약한 참조는 대상의 수명을 연장하지 않으므로, 호출 시점에 대상이 이미 해제되었을 수 있습니다. 그 경우 `None` 입니다.</details>

## 📚 더 알아보기

- The Rust Book Ch15 (스마트 포인터): <https://doc.rust-lang.org/book/ch15-00-smart-pointers.html> — `Box`, `Deref`, `Drop`, `Rc`, `RefCell`, 순환 참조
- Rust by Example — `Box`, `Rc`: <https://doc.rust-lang.org/rust-by-example/std/box.html>
- std 문서: [`std::rc`](https://doc.rust-lang.org/std/rc/index.html), [`std::cell`](https://doc.rust-lang.org/std/cell/index.html)
- 다음 장: Ch12(Rust다운 코드), Ch13(동시성 — `Arc`/`Mutex` 로 이어집니다)
