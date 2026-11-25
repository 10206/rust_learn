# 03. rustc 컴파일러 에러 읽는 법

> Rust를 배울 때 가장 많이 보게 되는 것이 컴파일러 에러입니다. 그런데 이 에러 메시지는 "혼내는 글"이 아니라 **무엇이 문제이고, 어디를 고치면 되는지 알려 주는 안내문**에 가깝습니다. 이 문서는 그 안내문을 읽는 요령과, 입문자가 가장 자주 만나는 에러 20가지를 정리했어요.
>
> 이 문서의 에러 출력은 모두 실제로 `rustc 1.97.0`(edition 2024)에 코드를 넣어 얻은 결과입니다. 버전이 달라지면 문구가 조금 다를 수 있어요.

## 목차

1. [에러 메시지 해부하기](#1-에러-메시지-해부하기)
2. [에러를 만났을 때 읽는 순서](#2-에러를-만났을-때-읽는-순서)
3. [`rustc --explain`과 에러 코드 검색](#3-rustc---explain과-에러-코드-검색)
4. [자주 만나는 에러 모음](#4-자주-만나는-에러-모음)
   - 소유권/빌림: E0382, E0384, E0596, E0499, E0502, E0505, E0506, E0507
   - 라이프타임: E0597, E0716, E0106
   - 스레드/트레이트: E0373, E0277
   - 타입/이름/패턴: E0308, E0425, E0433, E0599, E0004, E0282
5. [에러가 한 번에 수십 개 나올 때](#5-에러가-한-번에-수십-개-나올-때)
6. [AI에게 에러를 물어볼 때](#6-ai에게-에러를-물어볼-때)

---

## 1. 에러 메시지 해부하기

가장 대표적인 에러(이동된 값을 다시 쓰기)를 분해해 봅시다.

```rust,compile_fail
fn main() {
    let s = String::from("hello");
    let t = s;
    println!("{s} {t}");
}
```

```text
error[E0382]: borrow of moved value: `s`
 --> src/main.rs:4:16
  |
2 |     let s = String::from("hello");
  |         - move occurs because `s` has type `String`, which does not implement the `Copy` trait
3 |     let t = s;
  |             - value moved here
4 |     println!("{s} {t}");
  |                ^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let t = s.clone();
  |              ++++++++
```

| 부분 | 예시 | 읽는 법 |
|---|---|---|
| **심각도 + 에러 코드** | `error[E0382]` | `error`는 빌드 실패, `warning`은 빌드는 되지만 주의. 대괄호 안의 `E0382`는 에러 번호로, 검색/`--explain`의 열쇠입니다. |
| **한 줄 제목** | `borrow of moved value: `s`` | 무슨 일이 일어났는지 한 문장 요약. 먼저 이것부터 읽으세요. |
| **위치** | `--> src/main.rs:4:16` | `파일:줄:열`. VS Code에서는 클릭하면 그 자리로 갑니다. |
| **코드 발췌 + 화살표** | `^`, `-` | 해당 줄을 보여 주고 밑줄로 가리켜요. |
| **주(primary) 라벨** | `^ value borrowed here after move` | `^^^`(캐럿)로 표시된 곳이 **실제로 규칙을 어긴 자리**입니다. |
| **보조(secondary) 라벨** | `- value moved here` | `---`(대시)로 표시된 곳은 **원인을 설명하는 이전 사건**입니다. 위에서는 "2줄에서 만들었고 → 3줄에서 이동했고 → 4줄에서 또 썼다"는 이야기가 줄 번호 순서로 읽혀요. |
| **`note:`** | `note: `State` defined here` | 배경 정보. 에러의 이유나 관련 정의 위치를 알려 줍니다. |
| **`help:`** | `help: consider cloning ...` | **컴파일러가 제안하는 수정안**. `+++`는 추가, `~`는 바뀌는 줄, `-`는 삭제를 뜻해요. 그대로 적용하면 되는 경우가 많지만, 의도에 맞는지는 꼭 생각해 보세요(예: 위의 `clone`은 "복제 비용을 감수해도 되면" 이라는 조건부 제안이에요). |

> 💡 **쉬운 설명**: Rust의 borrow checker(빌림 검사기)는 "이 값은 누가 갖고 있고, 누가 빌려 갔는지"를 장부처럼 추적합니다. 에러 메시지의 보조 라벨은 그 장부의 기록(만들었다 → 이동했다 → 또 쓰려 했다)을 시간 순서대로 보여 주는 것이라고 생각하면 편해요.

> ⚠️ **C++ 개발자 메모**: g++/clang++의 템플릿 에러처럼 수백 줄이 나오는 일은 드뭅니다. 대부분 10~20줄 안에서 원인과 해결책이 모두 나와요.

## 2. 에러를 만났을 때 읽는 순서

1. **첫 번째 에러부터** 고치세요. 뒤의 에러들은 첫 에러의 연쇄 반응인 경우가 많습니다. (터미널에서는 스크롤이 올라가니 `cargo check 2>&1 | head -40`로 앞부분만 보는 것도 방법이에요.)
2. 제목 한 줄을 읽습니다. (`cannot borrow`, `mismatched types`, `does not live long enough` ...)
3. `^^^`(주 라벨)가 있는 줄을 봅니다. 여기가 "걸린 곳"입니다.
4. `---`(보조 라벨)를 **줄 번호 순서대로** 읽어 이야기를 파악합니다.
5. `help:`가 있으면 읽어 보고, 그 제안이 내 의도와 맞는지 판단합니다.
6. 모르겠으면 에러 코드로 `rustc --explain`을 읽습니다.
7. 한 가지를 고치고 다시 `cargo check`. 한 번에 여러 개를 고치지 마세요.

> 팁: `cargo check`는 실행 파일을 만들지 않고 검사만 해서 `cargo build`보다 훨씬 빠릅니다. rust-analyzer는 저장할 때마다 이것을 자동으로 돌려 줘서(설정에서 `clippy`로 바꿨다면 clippy 검사까지) 에디터에 빨간 줄로 보여 줘요.

## 3. `rustc --explain`과 에러 코드 검색

에러 코드를 알면 컴파일러에 내장된 **상세 해설**을 읽을 수 있습니다.

```bash
rustc --explain E0382
```

이 명령은 에러가 나는 이유, 틀린 예제, 올바른 예제를 차례로 보여 줍니다. 인터넷이 없어도 되고, 설치된 컴파일러 버전에 정확히 맞는 설명이에요. 웹에서는 `https://doc.rust-lang.org/error_codes/E0382.html`에서 같은 내용을 볼 수 있습니다. (VS Code에서는 에러 코드를 클릭하면 이 페이지로 연결되기도 해요.)

`cargo clippy`의 경고(`warning: clippy::...`)는 `rustc --explain`이 아니라 `https://rust-lang.github.io/rust-clippy/master/`에서 lint 이름으로 검색합니다.

---

## 4. 자주 만나는 에러 모음

각 항목은 **한 줄 의미 → 최소 재현 코드 → 실제 출력 → 고치는 법 → 관련 챕터** 순서입니다. 출력의 `src/main.rs`는 재현 코드를 그 파일에 넣었다고 가정한 경로예요. 긴 출력은 `... (이하 생략)`으로 줄였습니다.

### E0382 — 값이 이미 이동(move)됐어요: borrow of moved value

- **한 줄 의미**: 소유권이 다른 변수로 넘어간 뒤에 원래 변수를 또 쓰려고 했어요. C++이라면 복사(또는 `std::move` 후 빈 객체)였겠지만 Rust는 기본이 이동이고, 이동한 뒤의 변수는 사용 금지입니다.
- **관련 챕터**: ch03 소유권

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let s = String::from("hello");
    let t = s;
    println!("{s} {t}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0382]: borrow of moved value: `s`
 --> src/main.rs:4:16
  |
2 |     let s = String::from("hello");
  |         - move occurs because `s` has type `String`, which does not implement the `Copy` trait
3 |     let t = s;
  |             - value moved here
4 |     println!("{s} {t}");
  |                ^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
3 |     let t = s.clone();
  |              ++++++++
```

**고치는 법**: 정말 둘 다 필요하면 `.clone()`으로 복제하고, 읽기만 하면 되면 `&s`로 빌려 주세요. 함수에 넘길 때도 `&String`/`&str`로 받게 바꾸는 것이 보통 정답입니다.

```rust
fn main() {
    let s = String::from("hello");
    let t = s.clone(); // 복제 (또는 let t = &s; 로 빌리기)
    println!("{s} {t}");
}
```

### E0384 — 불변 변수에 두 번 대입했어요

- **한 줄 의미**: `let`으로 만든 변수는 기본이 불변(immutable)입니다. 값을 바꾸려면 선언할 때 `mut`를 붙여야 해요. (Python의 변수나 C++의 non-const 변수와 기본값이 반대예요.)
- **관련 챕터**: ch02 문법 기초

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let x = 5;
    x = 6;
    println!("{x}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0384]: cannot assign twice to immutable variable `x`
 --> src/main.rs:3:5
  |
2 |     let x = 5;
  |         - first assignment to `x`
3 |     x = 6;
  |     ^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut x = 5;
  |         +++
```

**고치는 법**: `let mut x = 5;` 로 고칩니다. 바꿀 일이 없다면 새 `let`으로 가리기(shadowing)도 됩니다.

```rust
fn main() {
    let mut x = 5;
    x = 6;
    println!("{x}");
}
```

### E0596 — 불변 변수를 가변으로 빌리려 했어요

- **한 줄 의미**: `push` 같은 메서드는 `&mut self`가 필요한데, 변수가 `mut`로 선언되지 않았어요. 같은 이유로 함수 매개변수 `&T`를 `&mut T`처럼 쓸 때도 이 에러가 납니다.
- **관련 챕터**: ch02 문법 기초 / ch04 빌림

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let v = Vec::new();
    v.push(1);
    println!("{v:?}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0596]: cannot borrow `v` as mutable, as it is not declared as mutable
 --> src/main.rs:3:5
  |
3 |     v.push(1);
  |     ^ cannot borrow as mutable
  |
help: consider changing this to be mutable
  |
2 |     let mut v = Vec::new();
  |         +++
```

**고치는 법**: `let mut v = ...` 로 바꾸거나, 매개변수라면 `&mut Vec<i32>`로 받도록 시그니처를 고칩니다.

```rust
fn main() {
    let mut v = Vec::new();
    v.push(1);
    println!("{v:?}");
}
```

### E0499 — 가변 참조를 동시에 둘 만들었어요

- **한 줄 의미**: 같은 값에 대해 `&mut`는 한 번에 하나만 살아 있을 수 있어요. 첫 번째 참조가 아직 쓰이는 동안 두 번째를 만들면 에러입니다.
- **관련 챕터**: ch04 참조와 빌림

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let mut s = String::from("a");
    let r1 = &mut s;
    let r2 = &mut s;
    r1.push('b');
    r2.push('c');
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0499]: cannot borrow `s` as mutable more than once at a time
 --> src/main.rs:4:14
  |
3 |     let r1 = &mut s;
  |              ------ first mutable borrow occurs here
4 |     let r2 = &mut s;
  |              ^^^^^^ second mutable borrow occurs here
5 |     r1.push('b');
  |     -- first borrow later used here
```

**고치는 법**: 첫 번째 참조를 다 쓴 뒤에 두 번째를 만들도록 순서를 바꾸세요. (참조의 수명은 `}`가 아니라 마지막으로 쓰인 줄까지예요.)

```rust
fn main() {
    let mut s = String::from("a");
    let r1 = &mut s;
    r1.push('b'); // r1은 여기까지만 사용
    let r2 = &mut s;
    r2.push('c');
    println!("{s}");
}
```

### E0502 — 읽기 참조가 살아 있는데 값을 바꾸려 했어요

- **한 줄 의미**: `&v[0]`처럼 빌린 참조가 나중에 또 쓰이는데, 그 사이에 `v.push(4)`로 벡터를 바꿨어요. push가 메모리를 재할당하면 참조가 허공을 가리키게 되므로(C++의 iterator 무효화와 같은 문제) 컴파일러가 막아 줍니다.
- **관련 챕터**: ch04 참조와 빌림

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    v.push(4);
    println!("{first}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:5
  |
3 |     let first = &v[0];
  |                  - immutable borrow occurs here
4 |     v.push(4);
  |     ^^^^^^^^^ mutable borrow occurs here
5 |     println!("{first}");
  |                ----- immutable borrow later used here
```

**고치는 법**: 값을 복사해서 들고 있거나(`let first = v[0];` — `i32`는 Copy), 참조 사용을 `push` 앞에서 끝내세요.

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    let first = v[0]; // i32 는 Copy 라서 값 복사
    v.push(4);
    println!("{first}");
}
```

### E0505 — 빌려 준 값을 다른 곳으로 이동시키려 했어요

- **한 줄 의미**: `r = &v`로 빌려 준 상태에서 `consume(v)`로 소유권을 넘겨 버렸어요. 빌린 쪽(`r`)이 나중에 쓰이므로 안 됩니다.
- **관련 챕터**: ch03 소유권 / ch04 빌림

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn consume(v: Vec<i32>) {
    println!("{}", v.len());
}
fn main() {
    let v = vec![1, 2, 3];
    let r = &v;
    consume(v);
    println!("{r:?}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0505]: cannot move out of `v` because it is borrowed
 --> src/main.rs:7:13
  |
5 |     let v = vec![1, 2, 3];
  |         - binding `v` declared here
6 |     let r = &v;
  |             -- borrow of `v` occurs here
7 |     consume(v);
  |             ^ move out of `v` occurs here
8 |     println!("{r:?}");
  |                - borrow later used here
  |
help: consider cloning the value if the performance cost is acceptable
  |
6 |     let r = &v.clone();
  |               ++++++++
```

**고치는 법**: `consume(v.clone())`처럼 복제해서 넘기거나, `consume`이 `&[i32]`를 받게 바꾸거나, `r`의 사용을 먼저 끝내세요.

```rust
fn consume(v: &[i32]) {
    println!("{}", v.len());
}
fn main() {
    let v = vec![1, 2, 3];
    let r = &v;
    consume(&v);
    println!("{r:?}");
}
```

### E0506 — 빌려 준 값에 대입했어요

- **한 줄 의미**: `r = &x`가 살아 있는 동안 `x = 20`으로 값을 덮어썼어요. 참조가 가리키는 값이 몰래 바뀌는 것을 막는 규칙입니다.
- **관련 챕터**: ch04 참조와 빌림

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let mut x = 10;
    let r = &x;
    x = 20;
    println!("{r}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0506]: cannot assign to `x` because it is borrowed
 --> src/main.rs:4:5
  |
3 |     let r = &x;
  |             -- `x` is borrowed here
4 |     x = 20;
  |     ^^^^^^ `x` is assigned to here but it was already borrowed
5 |     println!("{r}");
  |                - borrow later used here
```

**고치는 법**: 참조를 다 쓴 뒤에 대입하거나, 처음부터 값을 복사해서 읽으세요.

```rust
fn main() {
    let mut x = 10;
    let r = &x;
    println!("{r}");
    x = 20; // r 을 다 쓴 뒤에 대입
    println!("{x}");
}
```

### E0507 — 참조 뒤에 있는 값을 꺼내서 이동하려 했어요

- **한 줄 의미**: `r`은 `&Task`(빌린 것)인데 `r.name`(String)을 통째로 꺼내 가려 했어요. 빌린 것의 일부를 뽑아 가면 원본이 구멍 난 상태가 되니까 금지입니다. 반복문에서 `for t in &tasks { let n = t.name; }` 형태로 자주 만납니다.
- **관련 챕터**: ch03 소유권 / ch04 빌림

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
struct Task { name: String }
fn main() {
    let t = Task { name: "a".to_string() };
    let r = &t;
    let n = r.name;
    println!("{n}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0507]: cannot move out of `r.name` which is behind a shared reference
 --> src/main.rs:5:13
  |
5 |     let n = r.name;
  |             ^^^^^^ move occurs because `r.name` has type `String`, which does not implement the `Copy` trait
  |
help: consider borrowing here
  |
5 |     let n = &r.name;
  |             +
help: consider cloning the value if the performance cost is acceptable
  |
5 |     let n = r.name.clone();
  |                   ++++++++
```

**고치는 법**: 빌려서 쓰거나(`&r.name`), 복제하거나(`r.name.clone()`), 정말 소유권이 필요하면 `Option::take()`/`std::mem::take`를 쓰세요.

```rust
struct Task {
    name: String,
}
fn main() {
    let t = Task { name: "a".to_string() };
    let r = &t;
    let n = &r.name; // 빌리기
    println!("{n}");
}
```

### E0597 — 빌린 값이 충분히 오래 살지 못해요 (dangling 방지)

- **한 줄 의미**: `x`는 안쪽 블록이 끝날 때 사라지는데, 바깥의 `r`이 그 뒤에도 `x`를 가리키려 해요. C++이라면 조용히 댕글링 포인터가 되는 코드입니다.
- **관련 챕터**: ch04 참조와 빌림 / ch09 라이프타임

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let r;
    {
        let x = 5;
        r = &x;
    }
    println!("{r}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0597]: `x` does not live long enough
 --> src/main.rs:5:13
  |
4 |         let x = 5;
  |             - binding `x` declared here
5 |         r = &x;
  |             ^^ borrowed value does not live long enough
6 |     }
  |     - `x` dropped here while still borrowed
7 |     println!("{r}");
  |                - borrow later used here
```

**고치는 법**: 값의 주인(`x`)이 참조보다 더 오래 살도록 선언 위치를 바깥으로 옮기거나, 참조 대신 값을 복사/소유하세요.

```rust
fn main() {
    let x = 5; // 바깥 범위로 이동
    let r;
    {
        r = &x;
    }
    println!("{r}");
}
```

### E0716 — 임시 값이 빌려진 채로 사라졌어요

- **한 줄 의미**: `String::from("temp")`는 이 줄이 끝나면 버려지는 임시 값인데, 그 `as_str()` 결과를 변수에 저장해 두려 했어요. 위의 E0597과 같은 종류인데 원인이 '이름 없는 임시 값'인 경우입니다.
- **관련 챕터**: ch04 참조와 빌림 / ch09 라이프타임

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let s: &str = String::from("temp").as_str();
    println!("{s}");
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0716]: temporary value dropped while borrowed
 --> src/main.rs:2:19
  |
2 |     let s: &str = String::from("temp").as_str();
  |                   ^^^^^^^^^^^^^^^^^^^^         - temporary value is freed at the end of this statement
  |                   |
  |                   creates a temporary value which is freed while still in use
3 |     println!("{s}");
  |                - borrow later used here
  |
help: consider using a `let` binding to create a longer lived value
  |
2 ~     let binding = String::from("temp");
3 ~     let s: &str = binding.as_str();
  |
```

**고치는 법**: 임시 값을 `let`으로 이름 붙여 주인을 만들어 주세요(컴파일러의 `help`가 그대로 알려 줘요).

```rust
fn main() {
    let owner = String::from("temp");
    let s: &str = owner.as_str();
    println!("{s}");
}
```

### E0106 — 라이프타임 표기가 빠졌어요

- **한 줄 의미**: 참조 두 개를 받아 참조 하나를 돌려주는데, 결과가 `a`에서 왔는지 `b`에서 왔는지 컴파일러가 알 수 없어요. 라이프타임은 '이 참조들의 유효 기간이 어떤 관계인지' 서술하는 이름표입니다.
- **관련 챕터**: ch09 라이프타임

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn longest(a: &str, b: &str) -> &str {
    if a.len() > b.len() { a } else { b }
}
fn main() {
    println!("{}", longest("ab", "c"));
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:33
  |
1 | fn longest(a: &str, b: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `a` or `b`
help: consider introducing a named lifetime parameter
  |
1 | fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
  |           ++++     ++          ++          ++
```

**고치는 법**: `help`가 제안한 대로 `<'a>`를 붙여 "반환값은 두 입력 중 짧은 쪽만큼 산다"고 적어 주세요.

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() > b.len() { a } else { b }
}
fn main() {
    println!("{}", longest("ab", "c"));
}
```

### E0373 — 클로저가 지역 변수를 빌린 채 스레드로 나가려 해요

- **한 줄 의미**: `thread::spawn`에 넘긴 클로저는 현재 함수보다 오래 살 수 있어서(`'static`), 지역 변수 `v`를 빌려 갈 수 없어요.
- **관련 챕터**: ch13 동시성 (클로저는 ch07)

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
use std::thread;
fn main() {
    let v = vec![1, 2, 3];
    let h = thread::spawn(|| {
        println!("{v:?}");
    });
    h.join().unwrap();
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0373]: closure may outlive the current function, but it borrows `v`, which is owned by the current function
 --> src/main.rs:4:27
  |
4 |     let h = thread::spawn(|| {
  |                           ^^ may outlive borrowed value `v`
5 |         println!("{v:?}");
  |                    - `v` is borrowed here
  |
note: function requires argument type to outlive `'static`
 --> src/main.rs:4:13
  |
4 |       let h = thread::spawn(|| {
  |  _____________^
5 | |         println!("{v:?}");
6 | |     });
  | |______^
help: to force the closure to take ownership of `v` (and any other referenced variables), use the `move` keyword
  |
4 |     let h = thread::spawn(move || {
  |                           ++++
```

**고치는 법**: `move` 키워드로 소유권을 클로저 안으로 옮기세요. 여러 스레드가 함께 쓰려면 `Arc`로 감쌉니다.

```rust
use std::thread;
fn main() {
    let v = vec![1, 2, 3];
    let h = thread::spawn(move || {
        println!("{v:?}");
    });
    h.join().unwrap();
}
```

### E0277 — 트레이트가 구현돼 있지 않아요 (1) 타입이 다른 연산

- **한 줄 의미**: `i32 + f64`처럼 Rust는 숫자 타입을 자동 변환하지 않아요. E0277은 "이 타입에 그 트레이트(기능)가 없다"는 가장 흔한 에러로, 제네릭 경계(`T: Display` 등)를 못 지킬 때도 나옵니다.
- **관련 챕터**: ch02 문법 기초 / ch08 트레이트

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let a: i32 = 1;
    let b: f64 = 2.0;
    println!("{}", a + b);
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0277]: cannot add `f64` to `i32`
 --> src/main.rs:4:22
  |
4 |     println!("{}", a + b);
  |                      ^ no implementation for `i32 + f64`
  |
  = help: the trait `Add<f64>` is not implemented for `i32`
  ... (이하 생략)
```

**고치는 법**: `as`로 명시적으로 변환하세요. 정수 → 실수는 `f64::from(a)`가 무손실이라 더 좋습니다.

```rust
fn main() {
    let a: i32 = 1;
    let b: f64 = 2.0;
    println!("{}", f64::from(a) + b);
}
```

### E0277 — 트레이트가 구현돼 있지 않아요 (2) `Send`가 아닌 타입을 스레드로

- **한 줄 의미**: `Rc`는 참조 횟수를 원자적으로 세지 않아서 스레드 사이에 보낼 수 없어요(`Send` 아님). 컴파일러가 데이터 레이스를 컴파일 시점에 막는 대표 사례입니다.
- **관련 챕터**: ch11 스마트 포인터 / ch13 동시성

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let a: i32 = 1;
    let b: f64 = 2.0;
    println!("{}", a + b);
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0277]: cannot add `f64` to `i32`
 --> src/main.rs:4:22
  |
4 |     println!("{}", a + b);
  |                      ^ no implementation for `i32 + f64`
  |
  = help: the trait `Add<f64>` is not implemented for `i32`
help: the following other types implement trait `Add<Rhs>`
 --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/arith.rs:98:8
  |
  = note: `i32` implements `Add`
 ::: /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/ops/arith.rs:113:0
  |
  = note: in this macro invocation
 --> /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/internal_macros.rs:22:8
  |
  = note: `&i32` implements `Add<i32>`
 ::: /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/internal_macros.rs:33:8
  |
  = note: `i32` implements `Add<&i32>`
 ::: /rustc/2d8144b7880597b6e6d3dfd63a9a9efae3f533d3/library/core/src/internal_macros.rs:44:8
  |
  = note: `&i32` implements `Add`
  = note: this error originates in the macro `add_impl` (in Nightly builds, run with -Z macro-backtrace for more info)
```

**고치는 법**: 스레드 간 공유는 `Arc`(원자적 참조 카운트)로 바꿉니다. 값을 바꿔야 하면 `Arc<Mutex<T>>`.

```rust
use std::sync::Arc;
use std::thread;
fn main() {
    let r = Arc::new(5);
    let h = thread::spawn(move || println!("{r}"));
    h.join().unwrap();
}
```

### E0308 — 타입이 맞지 않아요 (mismatched types)

- **한 줄 의미**: 가장 흔한 에러예요. 위 예에서는 (1) 함수 끝줄에 `;`를 붙여서 값 대신 `()`를 반환했고, (2) 정수 자리에 문자열을 넣었어요. Rust에서 `;`는 '이 식의 값을 버린다'는 뜻이라, 마지막 줄의 `;` 하나가 반환 타입을 바꿉니다.
- **관련 챕터**: ch02 문법 기초

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn add(a: i32, b: i32) -> i32 {
    a + b;
}
fn main() {
    let x: i32 = "5";
    println!("{}", add(1, x));
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0308]: mismatched types
 --> src/main.rs:1:27
  |
1 | fn add(a: i32, b: i32) -> i32 {
  |    ---                    ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     a + b;
  |          - help: remove this semicolon to return this value

error[E0308]: mismatched types
 --> src/main.rs:5:18
  |
5 |     let x: i32 = "5";
  |            ---   ^^^ expected `i32`, found `&str`
  |            |
  |            expected due to this
```

**고치는 법**: (1) 마지막 줄의 `;`를 지우세요. (2) 타입에 맞는 값을 쓰거나 `"5".parse::<i32>()`로 변환하세요.

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
fn main() {
    let x: i32 = "5".parse().unwrap();
    println!("{}", add(1, x));
}
```

### E0425 — 이름을 찾을 수 없어요

- **한 줄 의미**: 오타이거나, 스코프 밖의 변수이거나, 선언하지 않은 함수를 불렀어요.
- **관련 챕터**: ch02 문법 기초 / ch10 모듈

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let count = 3;
    println!("{}", cnt);
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0425]: cannot find value `cnt` in this scope
 --> src/main.rs:3:20
  |
3 |     println!("{}", cnt);
  |                    ^^^ not found in this scope
```

**고치는 법**: 철자와 스코프를 확인하세요. rust-analyzer가 빨간 줄과 함께 비슷한 이름을 제안해 줍니다.

```rust
fn main() {
    let count = 3;
    println!("{}", count);
}
```

### E0433 — 경로/타입을 찾을 수 없어요 (보통 `use` 누락)

- **한 줄 의미**: `HashMap`은 prelude에 없어서 `use`로 가져와야 해요. C++의 `#include <unordered_map>`이나 Python의 `from collections import ...`에 해당합니다.
- **관련 챕터**: ch07 컬렉션 / ch10 모듈

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let m = HashMap::<String, i32>::new();
    println!("{}", m.len());
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0433]: cannot find type `HashMap` in this scope
 --> src/main.rs:2:13
  |
2 |     let m = HashMap::<String, i32>::new();
  |             ^^^^^^^ use of undeclared type `HashMap`
  |
help: consider importing this struct
  |
1 + use std::collections::HashMap;
  |
```

**고치는 법**: `help`가 알려 준 `use std::collections::HashMap;` 한 줄을 파일 맨 위에 추가하세요.

```rust
use std::collections::HashMap;
fn main() {
    let m = HashMap::<String, i32>::new();
    println!("{}", m.len());
}
```

### E0599 — 그런 메서드가 없어요

- **한 줄 의미**: 타입에 없는 메서드를 불렀어요. 트레이트 메서드라면 (a) 그 트레이트를 구현하지 않았거나 (b) 트레이트를 `use`로 가져오지 않았거나 (c) 제네릭 경계(`T: Trait`)가 없는 경우입니다.
- **관련 챕터**: ch05 구조체와 열거형 / ch08 트레이트

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
struct Dog;
fn main() {
    let d = Dog;
    d.bark();
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0599]: no method named `bark` found for struct `Dog` in the current scope
 --> src/main.rs:4:7
  |
1 | struct Dog;
  | ---------- method `bark` not found for this struct
...
4 |     d.bark();
  |       ^^^^ method not found in `Dog`
```

**고치는 법**: 메서드를 `impl` 블록에 추가하거나 필요한 트레이트를 구현/`use`하세요.

```rust
struct Dog;
impl Dog {
    fn bark(&self) {
        println!("woof");
    }
}
fn main() {
    let d = Dog;
    d.bark();
}
```

### E0004 — match가 모든 경우를 다루지 않아요

- **한 줄 의미**: `match`는 모든 가능성을 빠짐없이 적어야 해요(exhaustive). 열거형에 변종을 추가하면 관련 `match`가 전부 에러가 되므로, 수정할 곳을 컴파일러가 찾아 줍니다. 이것이 Rust 리팩터링이 안전한 이유 중 하나예요.
- **관련 챕터**: ch05 구조체와 열거형

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
enum State { Idle, Busy, Done }
fn main() {
    let s = State::Idle;
    match s {
        State::Idle => println!("idle"),
        State::Busy => println!("busy"),
    }
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0004]: non-exhaustive patterns: `State::Done` not covered
 --> src/main.rs:4:11
  |
4 |     match s {
  |           ^ pattern `State::Done` not covered
  |
note: `State` defined here
 --> src/main.rs:1:6
  |
1 | enum State { Idle, Busy, Done }
  |      ^^^^^               ---- not covered
  = note: the matched value is of type `State`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
  |
6 ~         State::Busy => println!("busy"),
7 ~         State::Done => todo!(),
  |
```

**고치는 법**: 빠진 경우를 추가하세요. 정말 나머지를 한꺼번에 처리하려면 `_ =>`를 쓰되, 남용하면 이 안전장치가 사라집니다.

```rust
enum State {
    Idle,
    Busy,
    Done,
}
fn main() {
    let s = State::Idle;
    match s {
        State::Idle => println!("idle"),
        State::Busy => println!("busy"),
        State::Done => println!("done"),
    }
}
```

### E0282 — 타입을 추론할 수 없어요

- **한 줄 의미**: `Vec::new()`만으로는 안에 뭐가 들어갈지 알 수 없어요. 보통 `collect()`, `parse()`, `Vec::new()`에서 만납니다.
- **관련 챕터**: ch07 컬렉션/이터레이터 / ch02

최소 재현 코드 (`src/main.rs`):

```rust,compile_fail
fn main() {
    let v = Vec::new();
    println!("{}", v.len());
}
```

실제 rustc 1.97 출력 (발췌):

```text
error[E0282]: type annotations needed for `Vec<_>`
 --> src/main.rs:2:9
  |
2 |     let v = Vec::new();
  |         ^   ---------- type must be known at this point
  |
help: consider giving `v` an explicit type, where the type for type parameter `T` is specified
  |
2 |     let v: Vec<T> = Vec::new();
  |          ++++++++
```

**고치는 법**: 타입을 적어 주세요: `let v: Vec<i32> = Vec::new();` 또는 `collect::<Vec<_>>()`, `parse::<u16>()`.

```rust
fn main() {
    let v: Vec<i32> = Vec::new();
    println!("{}", v.len());
}
```

---

## 5. 에러가 한 번에 수십 개 나올 때

- 가장 위의 에러 하나만 고치고 `cargo check`를 다시 돌리세요. 타입 하나가 틀리면 그 타입을 쓰는 모든 곳에서 연쇄적으로 에러가 나곤 합니다.
- `warning`은 일단 무시하고 `error`부터 고치세요. (`unused variable` 경고는 `todo!()`가 남은 연습 파일에서 자연스럽게 생겨요. 실습 파일 맨 위의 `#![allow(...)]`가 그것을 숨겨 주고 있습니다.)
- 라이프타임/빌림 에러가 여러 개 겹치면, 일단 `.clone()`으로 소유권 문제를 없애 **돌아가는 코드를 먼저** 만든 뒤, 이후에 복제를 줄여 나가는 것이 입문 단계에서는 현실적인 전략입니다. 단, 마음대로 `clone()`을 뿌리는 습관은 [02 문서](02_vibe_coding_with_rust.md)의 "clone 폭탄" 항목을 참고해 경계하세요.

## 6. AI에게 에러를 물어볼 때

에러를 "고쳐 줘"라고만 던지면 학습 효과가 사라집니다. 이렇게 물어보세요.

```text
아래 코드에서 cargo check 가 이런 에러를 냅니다.
(코드 전체 또는 관련 함수)
(에러 메시지 전체 - 잘라내지 말고 처음부터 끝까지)

1. 이 에러가 "어떤 규칙" 때문에 나는지 소유권/빌림/라이프타임 용어로 설명해 주세요.
2. 해결 방법을 2가지 이상 보여 주고 각각의 장단점(복사 비용, API 변화)을 알려 주세요.
3. 아직 코드는 고치지 마세요. 제가 먼저 고쳐 보겠습니다.
```

- 에러 메시지는 **전체**를 붙여 넣으세요. `help:`와 `note:`에 단서가 있습니다.
- Rust 버전과 edition(이 저장소는 edition 2024)을 함께 알려 주면 오래된 해법을 피할 수 있어요.

더 큰 그림은 [02. AI 에이전트와 Rust 바이브 코딩](02_vibe_coding_with_rust.md), 개념 대응표는 [01. C++/Python/Rust 치트시트](01_cpp_python_rust_cheatsheet.md)를 보세요.
