# Ch03. 소유권과 이동

> 모든 값에는 소유자가 정확히 한 명. 값을 넘기면 소유권이 **이동**하고, 소유자가 사라지면 값도 **정확히 한 번** 정리된다.  ·  예상 소요: 이론 70분 / 실습 70분  ·  선행: Ch02

이 장은 Rust 전체에서 **가장 중요한 장**입니다. 컴파일 에러의 절반은 여기서 시작하고, "Rust 는 어렵다"는 평판의 출처이기도 해요.
하지만 규칙은 놀랄 만큼 적고, 한 번 이해하면 *C++ 에서 겪던 메모리 버그의 상당수가 컴파일 에러로 바뀌는* 이유가 보입니다.
천천히, 🔬 실험을 직접 해 보면서 읽으세요.

## 🎯 학습 목표

- 스택(stack)과 힙(heap)의 차이, `String` / `Vec` 가 메모리에서 어떻게 생겼는지 그림으로 설명할 수 있다.
- **소유권 3규칙**을 말하고, `let s2 = s1;` 이 왜 `s1` 을 쓸 수 없게 만드는지 설명한다.
- Rust 의 이동(move)과 C++ 의 복사 / `std::move` 의 차이 (특히 **moved-from 객체와 소멸자**) 를 정확히 비교할 수 있다.
- `Copy` 와 `Clone` 을 구분하고, 어떤 타입이 `Copy` 인지 판단한다.
- `impl Drop` 이 언제 호출되는지(스코프 끝 · 이동 · `drop()`) 와 drop 순서 규칙을 예측한다.
- `E0382` (이동 후 사용) 를 포함한 소유권 컴파일 에러를 읽고 고친다.

---

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| `b = a` (대입) | **복사**(깊은 복사가 기본) | 같은 객체에 이름표 하나 더 (alias) | **이동**(move) — `a` 는 더 못 씀. (`Copy` 타입이면 복사) |
| 소유자 수 | 개발자가 약속으로 관리 (`unique_ptr` / `shared_ptr` / 원시 포인터) | 참조 카운트 / GC 가 알아서 | **정확히 1명**, 컴파일러가 강제 |
| 이동 | `std::move(x)` = rvalue 로 **캐스트**만. 실제 이동은 move 생성자 | 해당 없음 | 언어 기본 동작: 비트 복사 + 원본 **정적 무효화** |
| 이동된 뒤의 원본 | 살아 있음 ("유효하지만 불특정"), **소멸자도 실행됨**, 쓰는 것도 컴파일됨 | 해당 없음 | **사용 불가**(E0382), 소멸자도 **실행되지 않음** |
| 깊은 복사 | `T b = a;` 한 줄 (조용히 비쌈) | `copy.deepcopy(a)` | `a.clone()` (**명시**, 비용이 코드에 보임) |
| 소멸자 | `~T()` (RAII) | `__del__` (시점 보장 약함) / `with` | `impl Drop for T` (RAII, 시점 결정적) |
| 소멸 순서 | 지역 변수는 선언의 역순 | 참조 카운트 0 이 될 때 (CPython) | 지역 변수 **선언의 역순**, 필드는 **선언 순서** |
| 메모리 해제 누락 / 이중 해제 | 가능 (UB) | 불가 (GC) | 불가 (안전한 Rust 에서는 컴파일 타임에 차단) |
| 해제 시점 비용 | 없음 (결정적) | GC 일시정지 가능 | 없음 (결정적) |

---

## 📘 이론

> 이 README 의 코드 조각은 표준 라이브러리만 쓰므로 <https://play.rust-lang.org> 에 그대로 붙여 넣어 실행할 수 있어요.

### 1. 먼저, 메모리 기초: 스택과 힙

프로그램은 데이터를 두 곳에 둡니다.

| | 스택 (stack) | 힙 (heap) |
|---|---|---|
| 비유 | 책상 위에 쌓은 접시 — 위에서만 넣고 뺀다 | 창고 — 필요할 때 자리를 빌리고 나중에 반납한다 |
| 크기 | 컴파일 때 정해진 고정 크기만 | 실행 중에 크기가 변해도 OK |
| 속도 | 매우 빠름 (포인터 하나 이동) | 느림 (빈 자리 찾기) |
| 정리 | 함수가 끝나면 자동 | **누가 언제 반납할지** 정해야 함 ← 이게 문제의 근원 |
| 예 | `i32`, `f64`, `bool`, 튜플, 고정 배열 | `String` 의 글자들, `Vec` 의 원소들 |

`String` 은 사실 **스택에 있는 작은 "손잡이" + 힙에 있는 실제 글자**로 이루어집니다.

```text
let s1 = String::from("hello");

   스택 (s1)                         힙
 ┌───────────────┐             ┌───┬───┬───┬───┬───┐
 │ ptr  ─────────┼───────────▶ │ h │ e │ l │ l │ o │
 │ len = 5       │             └───┴───┴───┴───┴───┘
 │ cap = 5       │              (64비트 환경에서 손잡이는 8+8+8 = 24 바이트)
 └───────────────┘
```

`Vec<T>` 도 똑같이 (ptr, len, cap) 입니다. C++ 의 `std::string` / `std::vector` 와 구조가 같아요.
**힙을 반납해야 하는 사람이 누구인가?** — C++ 은 "소멸자가 반납한다. 단, 복사/이동 규칙은 개발자가 맞게 짜야 한다", Python 은 "아무도 신경 안 쓴다(GC)",
Rust 는 **"소유자가 반납한다. 소유자는 항상 한 명이다"** 라고 답합니다.

> 💡 **쉬운 설명 — 소유권 = 열쇠 하나짜리 창고**
> 창고(힙 메모리)에는 열쇠가 정확히 하나만 있어요. 열쇠를 가진 사람(소유자)만 창고를 쓰고, 이사 갈 때(스코프가 끝날 때) 창고를 비우고 반납합니다.
> 열쇠를 다른 사람에게 넘기면(이동) 나는 더 이상 창고에 못 들어가요. 열쇠가 하나뿐이니 "두 사람이 동시에 비우려다 사고"(이중 해제)나
> "이미 비운 창고에 들어감"(해제 후 사용)이 *구조적으로* 불가능해집니다.

### 2. 소유권 3규칙

1. **모든 값은 소유자(owner)인 변수를 가진다.**
2. **한 번에 소유자는 한 명뿐이다.**
3. **소유자가 스코프를 벗어나면 값은 버려진다(drop).**

```rust
fn main() {
    {
        let s = String::from("hello");   // s 가 소유자. 힙에 "hello" 할당
        println!("{s}");
    }                                    // <- 스코프 끝: s 가 drop, 힙 반납 (C++ 의 소멸자 호출과 같은 시점)
    // println!("{s}");                  // ❌ s 는 이미 없음
}
```

### 3. 이동(move) — 규칙 2 의 결과

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;                  // 소유권이 s1 -> s2 로 "이동"
    // println!("{s1}");          // ❌ E0382: borrow of moved value: `s1`
    println!("{s2}");
}
```

일어나는 일은 이렇습니다.

```text
let s2 = s1;  직후

   s1 (무효!)           s2                           힙
 ┌─────────────┐   ┌───────────────┐          ┌───┬───┬───┬───┬───┐
 │ (사용 불가)  │   │ ptr  ─────────┼────────▶ │ h │ e │ l │ l │ o │
 └─────────────┘   │ len = 5       │          └───┴───┴───┴───┴───┘
                   │ cap = 5       │
                   └───────────────┘
```

- **스택의 손잡이 24 바이트만 복사**됩니다 (비트 복사). 힙의 글자들은 건드리지도 않아요. 그래서 **이동은 값의 크기와 상관없이 싸고**, 100 MB 짜리 `Vec` 를 옮겨도 24 바이트 복사입니다.
- 그리고 컴파일러가 **`s1` 을 "더 이상 쓸 수 없음"으로 표시**합니다. 이후 `s1` 을 쓰면 컴파일 에러, 스코프가 끝날 때 `s1` 의 drop 도 **호출하지 않아요**.
  그래서 같은 힙을 두 번 해제할 일이 없습니다.
- 정리: **Rust 의 이동 = 비트 복사 + 원본의 정적(컴파일 타임) 무효화.** 실행 중에 추가 비용이 거의 없습니다.

> 💡 **쉬운 설명 — "정적으로 무효화"란?**
> 프로그램을 *실행하기 전*, 컴파일러가 코드를 읽으며 "여기서 s1 의 열쇠를 s2 에게 넘겼네, 그럼 이 줄 이후에 s1 을 쓰는 코드가 있으면 에러!" 하고 추적한다는 뜻이에요.
> 실행 중에 "이 변수가 무효인가?" 를 검사하는 것이 아니라서 속도 손해가 없습니다.
> (조건에 따라 이동할 수도 안 할 수도 있는 드문 경우에만, 컴파일러가 숨은 작은 깃발(drop flag)을 하나 두고 실행 중에 확인해요.)

#### C++ 의 `std::move` 와 무엇이 다른가 — 정확한 비교

C++ 개발자가 가장 많이 오해하는 부분이라 꼼꼼히 비교합니다.

```cpp
std::string s1 = "hello";
std::string s2 = s1;              // (1) 복사: 힙을 새로 할당해서 "hello" 를 한 번 더 만든다
std::string s3 = std::move(s1);   // (2) std::move 는 "캐스트"일 뿐. 실제로 훔치는 건 string 의 move 생성자
std::cout << s1.size();           // (3) 컴파일됨! s1 은 "유효하지만 값은 불특정"인 상태로 살아 있다
}                                 // (4) s1, s2, s3 모두의 소멸자가 실행된다 (s1 은 비어 있어서 할 일이 없을 뿐)
```

| | C++ (`std::move`) | Rust (move) |
|---|---|---|
| `=` 의 기본 동작 | **복사** (깊은 복사) | **이동** |
| 이동을 하는 주체 | 타입의 move 생성자 / 대입 (**직접 작성**해야 할 수도) | 언어 자체 (모든 타입이 자동으로) |
| 이동 뒤 원본 | 계속 존재, 사용 가능 (값은 불특정) | **사용 불가** — 컴파일 에러 |
| 원본의 소멸자 | **실행됨** → 그래서 소멸자/move 생성자를 "빈 상태도 안전하게" 짜야 함 | **실행 안 됨** → "빈 상태" 라는 개념이 필요 없음 |
| 이동 후 사용 버그 | 컴파일 OK, 런타임에 터짐 (`clang-tidy` 의 `bugprone-use-after-move` 가 일부 탐지) | 컴파일 에러 |
| 실수로 비싼 복사 | `T b = a;` 한 글자로 발생 | `.clone()` 을 쓴 줄에서만 발생 |

이 챕터의 `cpp/main.cpp` 의 `~Task()` 를 보세요. **"이름이 비어 있으면 아무것도 출력하지 않는다"** 는 한 줄이 없으면, `Task` 를 `vector` 에 넣고 뺄 때마다
이동된 껍데기들의 소멸자가 `drop: ` 를 마구 찍습니다. Rust 에서는 이 한 줄이 아예 필요 없어요. 이동된 변수는 *drop 대상이 아니기* 때문입니다.

> ⚠️ **C++ 개발자가 헷갈리는 점**
> - Rust 의 `let s2 = s1;` 은 C++ 의 `auto s2 = std::move(s1);` 에 가깝습니다. 단, 그 뒤에 `s1` 을 쓰면 컴파일러가 막는다는 점이 결정적 차이예요.
> - 반대로 C++ 의 `auto s2 = s1;` (복사)을 Rust 로 옮기면 `let s2 = s1.clone();` 입니다.
> - C++ 의 RVO/복사 생략 덕분에 반환값은 이동조차 안 일어날 때가 많죠. Rust 도 같은 최적화를 하지만, **의미상**으로는 늘 이동입니다.

#### Python 과 비교

```python
a = [1, 2, 3]
b = a            # 이름표 하나 더. 같은 리스트
b.append(4)
print(a)         # [1, 2, 3, 4]  <- a 도 바뀜! (alias 버그의 단골 원인)
```

Python 의 변수는 객체에 붙은 이름표이고, 객체는 이름표가 0 개가 되면(참조 카운트) 치워집니다. 편하지만 **"누가 이 데이터를 바꿀 수 있지?"** 를 코드만 보고 알기 어려워요.
Rust 는 같은 데이터에 이름표(소유자)를 하나만 허용해서 이런 alias 를 원천 차단합니다. (읽기만 하는 공유는 "빌림"으로 허용 — 다음 장)

### 4. 함수와 소유권

```rust
fn takes(s: String) {                 // s 가 인자로 이동해 들어온다
    println!("got {s}");
}                                     // <- s 의 스코프 끝: drop (호출한 쪽은 이미 열쇠를 잃었다)

fn gives() -> String {
    String::from("new")               // 소유권을 호출자에게 "반환"
}

fn passes_through(s: String) -> String {
    s                                 // 받아서 그대로 돌려줌 (들어갔다 나오기)
}

fn main() {
    let a = String::from("x");
    takes(a);                         // a 이동
    // println!("{a}");               // ❌ E0382
    let b = gives();                  // b 가 소유자
    let b = passes_through(b);        // 소유권이 왕복 (shadowing 으로 같은 이름 재사용)
    println!("{b}");
}
```

- **인자를 넘기는 것 = 대입과 같은 규칙** (이동). **반환하는 것 = 소유권을 내보내는 것.**
- "받아서 쓰고 돌려주는" 패턴은 번거로워서, 다음 장의 **빌림(`&`)** 이 등장합니다. 이 챕터 실습의 `process(task) -> Task` 가 이 패턴이에요.
- `println!("{a}")` 는 `a` 를 **이동시키지 않습니다.** 매크로가 자동으로 `&a` (빌림)를 사용하기 때문이에요. 안 그러면 출력 한 번에 값이 사라지겠죠.

### 5. `Copy` 와 `Clone`

**모든 타입이 이동하는 것은 아닙니다.** 정수처럼 "스택에만 있는 작은 값" 은 비트 복사가 곧 완벽한 복제라서, 이동 대신 **복사**되고 원본도 계속 씁니다.

```rust
fn main() {
    let a = 5;
    let b = a;                // i32 는 Copy: a 도 계속 유효
    println!("{a} {b}");      // 5 5

    let s1 = String::from("hi");
    let s2 = s1.clone();      // String 은 Clone 만: 힙까지 새로 복사 (명시적, 비용 있음)
    println!("{s1} {s2}");    // hi hi
}
```

| | `Copy` | `Clone` |
|---|---|---|
| 발생 | **암묵적** (대입/인자 전달 때마다) | **명시적** (`.clone()` 호출) |
| 비용 | 항상 싸다 (단순 비트 복사) | 임의 (힙 할당, 깊은 복사) |
| 가능한 타입 | 정수·실수·`bool`·`char`·`&T`(공유 참조)·`()` 와, **전부 Copy 인 필드로만** 이루어진 튜플·배열·구조체 (직접 `#[derive(Clone, Copy)]` 를 붙여야 함) | 거의 모든 타입 (`#[derive(Clone)]`) |
| 불가능한 경우 | `String`/`Vec`/`Box` 처럼 힙을 소유하는 타입, **`Drop` 을 구현한 타입** | `Clone` 을 구현하지 않은 타입 (예: 락 가드) |

> 💡 **쉬운 설명 — Copy 는 "복사해도 아무 일 없는 타입"**
> 숫자 5 가 적힌 포스트잇은 복사해서 둘이 가져도 문제없죠. 하지만 "창고 열쇠"를 복사하면 두 사람이 같은 창고를 비우려 합니다.
> 그래서 열쇠(힙 소유) 를 가진 타입은 `Copy` 가 될 수 없고, 필요하면 `clone()` 으로 **창고째 새로 만들어야** 해요.

- `Copy` 는 `Clone` 의 일종입니다 (`#[derive(Clone, Copy)]` 두 개를 함께 씁니다).
- **`Drop` 을 구현한 타입은 `Copy` 불가** (`E0184`): 복사본마다 drop 이 호출되면 "정확히 한 번" 규칙이 깨지니까요.
- 판단 요령: "이 타입이 힙이나 외부 자원(파일, 소켓)을 *소유*하는가?" → 그렇다면 Copy 가 아닙니다.

### 6. Drop 과 RAII — 정리는 자동으로, 시점은 결정적으로

**RAII** (Resource Acquisition Is Initialization): "자원의 획득은 객체의 생성, 해제는 객체의 소멸". C++ 개발자에게는 익숙한 개념이죠. Rust 의 `Drop` 이 같은 역할입니다.

```rust
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {                      // 소유자가 사라질 때 컴파일러가 자동 호출
        println!("drop {}", self.0);
    }
}

fn main() {
    let _a = Noisy("a");
    let _b = Noisy("b");
    let _ = Noisy("temp");   // ← `_` 만 쓰면 바인딩이 없어서 "즉시" drop
    let _c = Noisy("c");
    println!("end of main");
}
```

실제 출력:

```text
drop temp
end of main
drop c
drop b
drop a
```

**drop 이 호출되는 시점 (모두 외워 둘 가치가 있어요)**

| 상황 | drop 시점 |
|---|---|
| 지역 변수 | 스코프(`}`)를 벗어날 때, **선언의 역순** (`c → b → a`) |
| 이동된 변수 | **호출 안 됨** (새 소유자가 책임) |
| 함수 인자 (소유권을 받은 경우) | 함수가 끝날 때 (넘기지 않았다면) |
| `drop(x)` (`std::mem::drop`) | 그 자리에서 즉시 (`x` 소유권을 받고 끝나는 함수일 뿐!) |
| `let _ = expr;` | 그 문장 끝에서 즉시 (`let _x = ...` 와 다름!) |
| 구조체 | 자신의 `drop()` 먼저 → 그 다음 필드들을 **선언 순서대로** |
| `Vec<T>` | 원소들을 **앞에서부터 순서대로** |
| 변수에 새 값 대입 (`x = ...`) | 덮어쓰이는 옛 값이 그 자리에서 즉시 |

C++ 와 비교하면 소멸 순서가 같아서 편합니다. (C++ 의 `std::vector` 원소 소멸 순서는 표준에 규정되어 있지 않은 반면 Rust 는 규정되어 있습니다.)

```rust
fn main() {
    let t = Noisy("x");
    // t.drop();         // ❌ E0040: explicit destructor calls not allowed
    drop(t);             // ✅ 일찍 정리하고 싶으면 std::mem::drop (소유권을 받아 버리는 평범한 함수)
}
```

`drop` 메서드를 직접 부를 수 없는 이유: 부른 뒤에도 스코프 끝에서 또 drop 되어 이중 해제가 되기 때문이에요. `drop(t)` 는 `t` 를 *이동*시키므로 안전합니다.

> ⚠️ **C++ / Python 개발자가 헷갈리는 점**
> - Python 의 `__del__` 은 "언젠가, 아마도" 호출됩니다 (CPython 은 참조 카운트 0 이면 즉시지만, 순환 참조·다른 구현체에서는 다름). 그래서 파일은 `with` 로 닫죠. Rust 의 `Drop` 은 **언제나 결정적**이어서 `with` 가 따로 필요 없어요 (`File`, `MutexGuard`, `TcpStream` 이 모두 이 방식).
> - `std::mem::forget(x)` 를 쓰면 drop 없이 잊을 수 있습니다 (메모리 누수는 "안전" 범주). 평소엔 쓸 일이 거의 없어요.

### 7. 값을 "빼내는" 법과 부분 이동

소유권 에러를 피하려고 `clone()` 만 남발하지 않도록, 흔히 쓰는 대안을 알아 둡시다.

```rust
use std::mem;

fn main() {
    // (1) Vec 에서 꺼내기: 인덱스로는 이동 불가(E0507), remove / pop / swap_remove / into_iter 는 가능
    let mut v = vec![String::from("a"), String::from("b")];
    let first = v.remove(0);             // 소유권이 Vec 밖으로 이동. 나머지는 앞으로 당겨짐

    // (2) Option 안의 값을 꺼내고 None 으로 만들기
    let mut slot = Some(String::from("job"));
    let taken = slot.take();             // taken = Some("job"), slot = None

    // (3) &mut 뒤에 있는 값을 통째로 가져오고 빈 값을 남기기 (빌림은 다음 장!)
    let mut s = String::from("hello");
    let old = mem::take(&mut s);         // old = "hello", s = ""
    let old2 = mem::replace(&mut s, String::from("new"));   // old2 = "", s = "new"

    println!("{first} {taken:?} {slot:?} {old} {old2:?} {s}");
}
```

**부분 이동(partial move)**: 구조체의 필드 하나만 옮기는 것도 되지만, 그 뒤 구조체 전체는 못 씁니다.

```rust
struct Pair { name: String, id: u32 }
fn main() {
    let p = Pair { name: String::from("a"), id: 7 };
    let n = p.name;            // name 만 이동 (부분 이동)
    println!("{}", p.id);      // ✅ 이동 안 된 필드(Copy)는 OK
    // let q = p;              // ❌ E0382: use of partially moved value: `p`
    println!("{n}");
}
```

단, **`Drop` 을 구현한 타입은 필드 이동이 아예 금지**됩니다 (`E0509`). 필드를 빼가면 `drop()` 이 "반쯤 비어 버린 값" 을 보게 되기 때문이에요.
이 챕터의 `Task` 가 바로 그 경우여서, `drain_names` 에서 `task.name.clone()` 을 써야 합니다.

### 8. 이 모든 것의 이유

C++ 에서의 `use-after-free`, `double-free`, `dangling pointer`, `iterator invalidation`, `use-after-move` 는 모두 **"그 메모리를 누가 책임지는지 코드가 말해 주지 않아서"** 생기는 버그입니다.
Rust 는 이 책임을 **타입 시스템에 새겨 넣어서**, 실행 전에 컴파일러가 검사합니다. 런타임 비용(GC, 참조 카운트)이 없는데도 메모리 안전하다는 것이 Rust 의 핵심 약속이에요.

처음에는 컴파일러와 씨름하는 느낌이 들 거예요. 팁: **에러 메시지의 세 줄(왜 Copy 가 아닌지 / 어디서 이동했는지 / 어디서 다시 썼는지)을 순서대로 읽으면** 거의 항상 답이 나옵니다. 그리고 컴파일러는 `help:` 로 고치는 방법도 보통 알려 줘요.

---

## 🧯 자주 만나는 컴파일 에러

모두 이 챕터의 🔬 실험실에서 재현할 수 있는 **실제 rustc 1.97 출력**입니다.

### 1) 이동 후 사용 — `E0382` (가장 흔한 에러)

```rust
let s1 = String::from("hello");
let s2 = s1;
println!("{s1}, {s2}");
```
```text
error[E0382]: borrow of moved value: `s1`
 --> o1.rs:4:16
  |
2 |     let s1 = String::from("hello");
  |         -- move occurs because `s1` has type `String`, which does not implement the `Copy` trait
3 |     let s2 = s1;
  |              -- value moved here
4 |     println!("{s1}, {s2}");
  |                ^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let s2 = s1.clone();
  |                ++++++++
```
**읽는 법**: ① 변수가 어떤 타입이고 왜 `Copy` 가 아닌지 → ② *어디서* 이동했는지(`value moved here`) → ③ *어디서* 다시 썼는지(`value borrowed here after move`).
**고치기**: (a) 정말 복사본이 둘 필요하면 `.clone()`, (b) 읽기만 하면 되면 **빌리기**(`&s1`, 다음 장), (c) 구조를 바꿔 소유자를 하나로.

### 2) 함수에 넘긴 뒤 사용 — `E0382`

```rust
fn consume(s: String) { println!("got {s}"); }
let name = String::from("build");
consume(name);
println!("{name}");
```
```text
error[E0382]: borrow of moved value: `name`
...
6 |     consume(name);
  |             ---- value moved here
7 |     println!("{name}");
  |                ^^^^ value borrowed here after move
  |
note: consider changing this parameter type in function `consume` to borrow instead if owning the value isn't necessary
 --> o2.rs:1:15
  |
1 | fn consume(s: String) {
  |    -------    ^^^^^^ this parameter takes ownership of the value
```
컴파일러가 "이 함수가 정말 소유권이 필요해? 빌리기만 하면 되는 거 아니야?" 하고 **설계를 되묻습니다.** 대부분 `&String` / `&str` 로 바꾸는 게 정답이에요 (4장).

### 3) 반복문 안의 이동 — `E0382` (double move)

```rust
for _ in 0..2 {
    consume(name);
}
```
```text
error[E0382]: use of moved value: `name`
 --> o3.rs:7:17
  |
6 |     for _ in 0..2 {
  |     ------------- inside of this loop
7 |         consume(name);
  |                 ^^^^ value moved here, in previous iteration of loop
```
첫 바퀴에서 이미 이동했으므로 두 번째 바퀴에서는 쓸 수 없습니다. 컴파일러는 루프를 "여러 번 실행될 수 있다"는 것까지 추적해요.

### 4) 조건부 이동 후 사용 — `E0382`

```rust
let s = String::from("x");
if std::env::args().count() > 5 {
    drop(s);
}
println!("{s}");
```
```text
error[E0382]: borrow of moved value: `s`
...
4 |         drop(s);
  |              - value moved here
5 |     }
6 |     println!("{s}");
  |                ^ value borrowed here after move
```
"실행 중에 안 이동할 수도 있다" 는 소용없습니다. **이동했을 가능성이 있으면 에러**예요 (보수적으로 안전한 쪽).

### 5) `for x in vec` 후 `vec` 재사용 — `E0382`

```text
error[E0382]: borrow of moved value: `names`
...
3 |     for n in names {
  |              ----- `names` moved due to this implicit call to `.into_iter()`
...
6 |     println!("{}", names.len());
  |                    ^^^^^ value borrowed here after move
  |
help: consider iterating over a slice of the `Vec<String>`'s content to avoid moving into the `for` loop
  |
3 |     for n in &names {
  |              +
```
`for x in v` 는 `v` 를 *소비*합니다 (`into_iter()`). 계속 쓰려면 `for x in &v` (읽기) 또는 `for x in &mut v` (수정).

### 6) `Drop` 타입의 필드 이동 — `E0509`

```text
error[E0509]: cannot move out of type `Task`, which implements the `Drop` trait
 --> o4.rs:7:13
  |
7 |     let n = t.name;
  |             ^^^^^^
  |             |
  |             cannot move out of here
  |             move occurs because `t.name` has type `String`, which does not implement the `Copy` trait
  |
help: consider borrowing here
7 |     let n = &t.name;
help: consider cloning the value if the performance cost is acceptable
7 |     let n = t.name.clone();
```

### 7) Vec 인덱스로 이동 — `E0507`

```text
error[E0507]: cannot move out of index of `Vec<String>`
 --> o5.rs:3:17
  |
3 |     let first = v[0];
  |                 ^^^^ move occurs because value has type `String`, which does not implement the `Copy` trait
help: consider borrowing here        ->  let first = &v[0];
help: consider cloning the value ...  ->  let first = v[0].clone();
```
`v[0]` 은 "Vec 에 *남아 있는* 원소" 를 가리키는 식이라, 그걸 이동시키면 Vec 안에 구멍이 생깁니다. `remove(0)` / `pop()` / 빌리기 / `clone()` 중에서 의도에 맞는 것을 고르세요.

### 8) `Copy` 를 붙일 수 없는 타입 — `E0204` / `E0184`

```text
error[E0204]: the trait `Copy` cannot be implemented for this type
 --> o9.rs:2:8
  |
1 | #[derive(Clone, Copy)]
  |                 ---- in this derive macro expansion
2 | struct Task { name: String }
  |        ^^^^   ------------ this field does not implement `Copy`

error[E0184]: the trait `Copy` cannot be implemented for this type; the type has a destructor
  = note: ... `Copy` not allowed on types with destructors
```

### 9) 소멸자를 직접 호출 — `E0040`

```text
error[E0040]: explicit use of destructor method
 --> e40.rs:5:7
  |
5 |     t.drop();
  |       ^^^^ explicit destructor calls not allowed
  |
help: consider using `drop` function
5 -     t.drop();
5 +     drop(t);
```

### 10) `move` 클로저로 넘긴 값 재사용 — `E0382`

```text
error[E0382]: borrow of moved value: `s1`
3 |     let t = std::thread::spawn(move || println!("{s1}"));
  |                                -------            -- variable moved due to use in closure
  |                                |
  |                                value moved into closure here
4 |     println!("{s1}");
  |                ^^ value borrowed here after move
```
스레드(13장)나 비동기(14장)에서 정말 자주 만납니다. 같은 규칙이니 놀라지 마세요.

---

## 🛠️ 실습: 작업 파이프라인

### 무엇을 만드나요

`Task { name: String, priority: u8, steps: u32 }` 가 **inbox → 처리 → done** 으로 흘러가며 소유권이 이동합니다.
`Task` 는 `impl Drop` 으로 `drop: <name>` 을 찍어서, **소유권이 끝나는 정확한 순간**이 출력에 나타납니다. 이 출력을 읽는 것이 곧 소유권 공부예요.

```text
== 2. run pipeline ==
start: write
finish: write (steps=1)
...
start: cleanup
cancel: cleanup
drop: cleanup           <- cancel 함수의 `}` 에서 task 가 drop (priority 0 은 취소)
...
== 3. drain names ==
drop: write             <- for 의 한 바퀴가 끝날 때마다 하나씩 drop
...
names: write,build,test,deploy
== 4. backup ==
copy=report original=report
drop: report            <- drop(copy) 로 일찍 버림
after explicit drop
== 5. scope ==
leaving scope
drop: inner-b           <- 선언의 역순
drop: inner-a
...
== 7. end of main ==
drop: report            <- main 이 끝나며 original 이 drop
```

### 파일 구성

| 파일 | 역할 |
|---|---|
| `python/main.py` | Python 참조 구현 (소멸자 자동 호출이 없어서 `release()` 를 직접 호출해 시점을 흉내) |
| `cpp/main.cpp` | C++20 참조 구현 (move 생성자, **moved-from 객체의 소멸자는 비어 있는지 검사** — 위 비교의 실제 코드) |
| `rust/src/main.rs` | ★ 여러분이 채울 파일 (TODO 10개 + 🔬 실험 10개) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법

```bash
cd chapters/ch03_ownership
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch03_ownership --example solution     # 정답 출력으로 "drop 위치" 를 먼저 구경하세요
cargo test -p ch03_ownership
../../scripts/compare.sh ch03                         # 세 언어의 출력(drop 순서 포함) 비교
../../scripts/compare.sh ch03 --solution
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| TODO(1) | `Task::new` | `&str` → `String` (소유권 만들기) |
| TODO(2) | `submit` | 인자로 소유권 받기, `Vec::push` |
| TODO(3) | `take_next` | `remove(0)` 으로 이동해 꺼내기, `Option` |
| TODO(4) | `process` | 받아서 고치고 **돌려주기** |
| TODO(5) | `finish` | 소유권을 컨테이너에 넘기기 |
| TODO(6) | `cancel` | 스코프 끝 = drop (RAII) |
| TODO(7) | `backup` | `clone` 의 위치/순서 |
| TODO(8) | `run_all` | `while let Some(..)`, 분기별 이동, `Copy` 필드 읽기 |
| TODO(9) | `drain_names` | `for x in vec` 소비, Drop 타입 필드 이동 금지 → `clone` |
| TODO(10) | `copy_vs_clone_demo` | `Copy` vs `Clone` |

### 완료 기준

- `cargo test -p ch03_ownership` 8개 모두 `ok`
- `../../scripts/compare.sh ch03` 에서 `✅ python vs rust : 출력이 똑같아요`

### 🔬 실험 목록 (`rust/src/main.rs` 의 `experiments()`)

| 실험 | 해볼 것 | 기대 결과 |
|---|---|---|
| 1 | `let s2 = s1;` 후 `s1` 출력 | `E0382` |
| 2 | 함수에 넘긴 뒤 같은 변수 사용 | `E0382` (+ "borrow instead" 제안) |
| 3 | `for` 안에서 이동 | `E0382` ("previous iteration of loop") |
| 4 | 이중 이동 / 조건부 이동 | `E0382` |
| 5 | `Drop` 타입의 필드 이동 | `E0509` |
| 6 | `v[0]` 로 String 꺼내기 | `E0507` |
| 7 | `String` 필드가 있는 구조체에 `Copy` | `E0204` |
| 8 | `for n in names` 후 `names` 재사용 | `E0382` (`into_iter`) |
| 9 | `t.drop()` 직접 호출 | `E0040` |
| 10 | **drop 순서 예측** (에러 아님) | 아래 정답 참고 |

**실험 10 의 정답** (`main` 첫 줄에 `experiments();` 를 넣고 실행):

```text
drop: temp              <- `let _ = ...` 는 바인딩이 없으니 그 문장이 끝나며 즉시
end of experiments
drop: c                 <- 나머지는 선언의 역순
drop: b
drop: a
```

**추가 도전**: (1) `process` 에서 일부러 `task.clone()` 을 반환해 보세요. 출력에 어떤 `drop:` 이 *추가로* 나타나나요? (옛 `task` 가 함수 끝에서 drop 되기 때문입니다.)
(2) C++ 파일의 `if (!name.empty())` 검사를 지우고 컴파일해서 실행해 보세요. `drop: ` (빈 이름) 줄이 몇 번 늘어나는지 세어 보면 moved-from 객체의 소멸자가 얼마나 자주 도는지 체감됩니다.

---

## 🤖 바이브 코딩 포인트

AI 가 생성한 Rust 코드에서 소유권 관점으로 **가장 먼저 볼 것들**:

- **`.clone()` 이 몇 번 나오나요?** AI 는 컴파일 에러(E0382, E0502…)를 "가장 쉽게" 없애려고 `.clone()` 을 남발합니다. 컴파일은 되지만 쓸데없는 힙 복사가 늘어요.
  각 `.clone()` 에 대해 "복사본이 *진짜로* 필요한가, 빌려도 되나?" 를 물어보세요. 프롬프트: *"불필요한 clone 을 빼고 참조(&)로 바꿔 줘. 꼭 필요한 clone 에는 이유를 주석으로 달아 줘."*
- **함수 매개변수가 `String` / `Vec<T>` (소유) 인지 `&str` / `&[T]` (빌림) 인지.** 읽기만 하는데 소유권을 받으면 호출하는 쪽이 불편해집니다 (대부분 `&str` / `&[T]` 가 정답. 4장).
- **`to_string()` / `to_owned()` / `String::from()` / `into()`** 가 반복문 안에 있으면 매번 힙 할당이 일어납니다. 필요한 곳인지 확인하세요.
- **AI 가 `Rc<RefCell<...>>` 나 `Arc<Mutex<...>>` 로 도망쳤나요?** 소유권 에러를 만나면 AI 는 공유 소유권으로 우회하곤 합니다 (11·13장). 단순한 소유권 이동으로 풀 수 있는지 먼저 물어보세요: *"공유 소유권 없이, 소유권 이동과 참조만으로 다시 설계해 줘."*
- **`impl Drop` 이 있는 타입**(파일, 락, 연결 등)은 drop 순서가 동작에 영향을 줄 수 있습니다. 필드 선언 순서와 지역 변수 선언 순서를 확인하세요.
- **`std::mem::take` / `replace` / `Option::take`** 가 보이면 "필드를 빼내는 정석 기법" 입니다. 정상이에요. 반대로 `unsafe` 나 `mem::forget` 가 보이면 이유를 반드시 물어보세요.
- 컴파일 에러가 나면 **에러 전문**을 AI 에게 주고 *"clone 으로 해결하지 말고, 왜 이동했는지 설명한 뒤 가장 적은 변경으로 고쳐 줘"* 라고 하면 설계에 대한 논의로 이어집니다.

---

## ✅ 셀프 체크

**Q1.** 아래에서 컴파일 에러가 나는 줄은? 이유는?

```rust
let a = 5;
let b = a;
let s = String::from("hi");
let t = s;
println!("{a} {b} {s} {t}");
```

<details><summary>답</summary>

`println!` 줄의 `{s}` — `s` 는 `String`(Copy 아님)이라 `let t = s;` 에서 이동되었기 때문에 `E0382`. `a` 는 `i32`(Copy)라서 `b` 로 복사된 뒤에도 유효합니다.
</details>

**Q2.** C++ 의 `std::string b = std::move(a);` 와 Rust 의 `let b = a;` 에서 `a` 의 소멸자(drop)는 각각 실행되나요? 이후 `a` 를 쓰면?

<details><summary>답</summary>

C++: `a` 는 moved-from 상태로 계속 존재하고 **소멸자가 실행**됩니다. 이후 `a.size()` 등은 컴파일되며 값은 불특정(보통 빈 문자열)입니다.
Rust: `a` 의 소유권이 이동했으므로 **drop 은 실행되지 않고**(새 소유자 `b` 가 한 번만 실행), 이후 `a` 를 쓰면 `E0382` 컴파일 에러입니다.
</details>

**Q3.** 다음 출력은 어떤 순서가 될까요?

```rust
let _x = Noisy("x");
let _y = Noisy("y");
let _ = Noisy("z");
println!("end");
```

<details><summary>답</summary>

`drop z` → `end` → `drop y` → `drop x`. `let _ = ...` 는 바인딩이 없어 즉시 drop, 나머지는 선언의 역순입니다.
</details>

**Q4.** `#[derive(Clone, Copy)] struct Task { name: String }` 은 왜 에러일까요? `struct P { x: i32 }` + `impl Drop for P` 에 `Copy` 를 붙이면?

<details><summary>답</summary>

`String` 은 힙을 소유해서 비트 복사하면 두 값이 같은 힙을 가리켜 이중 해제가 되므로 `Copy` 불가 (`E0204`). `Drop` 을 구현한 타입은 복사본마다 drop 이 호출되어 "정확히 한 번" 이 깨지므로 역시 불가 (`E0184`).
</details>

**Q5.** `Vec<String>` 에서 첫 원소를 `let first = v[0];` 으로 꺼내면 에러입니다. 의도별로 고치는 방법 세 가지를 말해 보세요.

<details><summary>답</summary>

① 읽기만: `let first = &v[0];` (빌림). ② 복사본 필요: `let first = v[0].clone();`. ③ Vec 에서 아예 빼서 소유: `let first = v.remove(0);` (또는 `swap_remove`, `pop`, `std::mem::take(&mut v[0])`).
</details>

**Q6.** `fn process(task: Task) -> Task` 처럼 소유권을 받아 돌려주는 설계의 불편한 점은? 다음 장에서 이를 어떻게 해결할까요?

<details><summary>답</summary>

호출자가 값을 계속 쓰려면 반환값을 다시 받아 써야 해서 번거롭습니다 (`let task = process(task);`). **빌림**(`&mut Task`)을 쓰면 소유권을 넘기지 않고 잠깐 빌려서 고치게 할 수 있어요 (4장).
</details>

---

## 📚 더 알아보기

- The Rust Book 4장 *Understanding Ownership*: <https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html>
  (4.1 *What is Ownership?* 가 이 장의 정식 버전입니다. 꼭 읽어 보세요)
- The Rust Book 15.3 *Running Code on Cleanup with the `Drop` Trait*: <https://doc.rust-lang.org/book/ch15-03-drop.html>
- Rust by Example — *Ownership and moves*: <https://doc.rust-lang.org/rust-by-example/scope/move.html>
- Rust Reference — *Destructors* (drop 순서의 정식 규칙): <https://doc.rust-lang.org/reference/destructors.html>
- std 문서: [`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html), [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html), [`std::mem::take`](https://doc.rust-lang.org/std/mem/fn.take.html), [`std::mem::replace`](https://doc.rust-lang.org/std/mem/fn.replace.html)
- Rust Book 4.1 의 "Variables and Data Interacting with Move" 절 (그림으로 보는 이동)
