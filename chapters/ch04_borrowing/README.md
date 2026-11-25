# Ch04. 참조·빌림·슬라이스

> 소유권을 넘기지 않고 "잠깐 빌려" 읽거나 고친다. 규칙은 단 하나의 문장: **쓰기 빌림(`&mut`) 하나, 또는 읽기 빌림(`&`) 여러 개 — 둘을 동시에는 안 된다.**  ·  예상 소요: 이론 70분 / 실습 70분  ·  선행: Ch03

3장에서 "값을 넘기면 소유권이 이동해서 원래 변수를 못 쓴다"를 배웠죠. 그럼 함수에 값을 줄 때마다 돌려받아야 할까요? 아닙니다.
**빌림(borrowing)** 이 그 해답이고, 이 장과 3장이 Rust 의 심장입니다. 그리고 이 규칙들이 C++ 의 *iterator invalidation*, *dangling pointer*, *data race* 를 컴파일 에러로 바꿔 줍니다.

## 🎯 학습 목표

- `&T` (공유/읽기 빌림) 와 `&mut T` (독점/쓰기 빌림) 를 쓰고, "하나의 `&mut` XOR 여러 개의 `&`" 규칙(aliasing XOR mutation)이 **왜** 존재하는지 설명한다.
- 참조가 가리키는 값보다 오래 살 수 없다는 것(dangling 불가)과, 참조의 수명이 "마지막 사용"까지라는 것(NLL)을 이해한다.
- `String` vs `&str`, `Vec<T>` vs `&[T]` 의 차이를 알고, **함수 매개변수는 `&str` / `&[T]` 로 받는 것이 관용구**인 이유를 설명한다.
- UTF-8 문자열에서 바이트 수와 글자 수가 다름을 이해하고, 문자열을 안전하게 자르고 순회한다.
- `E0499`, `E0502`, `E0596`, `E0106`, `E0515`, `E0597` 을 읽고 고친다.

---

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 읽기 전용 참조 | `const T&` | (모든 변수가 참조) | `&T` — 몇 개든 동시에 가능 |
| 수정 가능 참조 | `T&` (여러 개 가능, 검사 없음) | 리스트 등은 어디서든 수정 가능 | `&mut T` — **동시에 딱 하나**, 읽기 참조와 공존 불가 |
| 호출부에서 수정 가능성 | 안 보임 (`f(x)` 인지 `f(x)` 인지) | 안 보임 | **보임**: `f(&mut x)` |
| 포인터 | `T*` (null, 산술, 해제 후 사용 가능) | 없음 | 참조는 **항상 유효**, null 없음 (`Option<&T>` 사용) |
| 문자열 소유 | `std::string` | `str` (불변) | `String` (소유, 가변, 힙) |
| 문자열 뷰 | `std::string_view` (dangling 가능, **검사 없음**) | 슬라이스는 **복사** | `&str` (수명 검사됨, 복사 없음) |
| 배열 뷰 | `std::span<T>` | 슬라이스는 **복사** | `&[T]` / `&mut [T]` |
| `s.size()` / `len(s)` | 바이트 수 | **글자(코드포인트) 수** | `s.len()` = **바이트 수**, `s.chars().count()` = 글자 수 |
| `s[i]` | 바이트 | 글자 | **불가** (컴파일 에러). `&s[a..b]` 는 바이트 범위, 글자 경계 아니면 패닉 |
| 순회 중 컨테이너 수정 | 반복자 무효화 (UB) | `RuntimeError` / 조용한 버그 | **컴파일 에러** (`E0502`) |
| 지역 변수 참조 반환 | 컴파일됨, UB | 해당 없음 | **컴파일 에러** (`E0106` / `E0515`) |

---

## 📘 이론

> 이 README 의 코드 조각은 표준 라이브러리만 쓰므로 <https://play.rust-lang.org> 에 그대로 붙여 넣어 실행할 수 있어요.

### 1. 빌림이란: `&` 와 `&mut`

```rust
fn len_of(s: &String) -> usize {    // 읽기만: 공유 참조
    s.len()
}                                   // 참조는 소유자가 아니므로 여기서 drop 되어도 String 은 그대로

fn append_bang(s: &mut String) {    // 고치려면: 독점 참조
    s.push('!');
}

fn main() {
    let mut owned = String::from("hi");
    println!("{}", len_of(&owned));  // &owned = "owned 를 빌려줄게"  -> 2
    append_bang(&mut owned);        // &mut owned = "고치도록 빌려줄게"
    println!("{owned}");            // hi!   (소유권은 한 번도 안 움직였으니 계속 사용 가능)
}
```

- `&x` 로 **빌려주는 쪽**과 `fn f(x: &T)` 로 **빌리는 쪽** 모두에 `&` 가 보입니다. 호출부만 봐도 "이 함수가 내 값을 읽기만 하는지(`&`) 바꿀 수 있는지(`&mut`) 소유하는지(무표시)"가 드러나요.
  C++ 의 `f(x)` 는 값 복사인지 참조인지 시그니처를 봐야 알 수 있는 것과 대조적이죠.
- 참조는 **포인터 하나** 크기입니다. 빌려도 복사/이동이 없어서 싸요.
- 참조를 통해 값에 접근할 때는 보통 `*` 를 안 써도 됩니다 (메서드 호출·필드 접근은 자동으로 따라가요: *auto-deref*). 값 자체를 바꿀 때만 `*n += 1` 처럼 씁니다.

> 💡 **쉬운 설명 — 도서관 책 빌리기**
> 소유권은 "내 책", 빌림은 "도서관에서 빌린 책" 입니다. 책을 *읽기만* 하는 사람(`&`)은 동시에 여러 명이어도 괜찮아요.
> 하지만 책에 *메모하며 고치는* 사람(`&mut`)이 있으면, 그동안 다른 사람이 읽거나 고치면 내용이 뒤죽박죽이 되겠죠?
> 그래서 "고치는 사람 한 명 **또는** 읽는 사람 여러 명" 만 허용하고, 빌린 사람이 책을 돌려주기 전에 주인이 책을 버리는(drop) 일도 막습니다.

### 2. 핵심 규칙 두 가지

> **규칙 1.** 어느 순간에든 한 값에 대해 **`&mut` 참조가 딱 하나**이거나, **`&` 참조가 여러 개**이다. (둘이 섞이면 안 됨)
> **규칙 2.** 참조는 **항상 유효해야** 한다 (가리키는 값보다 오래 살 수 없다).

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;            // ✅ 읽기 참조 여러 개
    println!("{r1} {r2}");  // r1, r2 는 여기가 마지막 사용

    let w = &mut s;         // ✅ 위 읽기 참조들이 이미 끝났으니 OK  (NLL, 아래 설명)
    w.push('!');
    println!("{w}");
}
```

**NLL (Non-Lexical Lifetimes)**: 참조의 "살아 있는 구간"은 *중괄호 끝이 아니라 **마지막으로 사용된 줄까지*** 입니다.
그래서 `r1`, `r2` 를 마지막으로 쓴 뒤에는 `&mut s` 를 만들어도 괜찮아요. 규칙 위반은 "빌림이 *아직 쓰일 예정인데* 충돌하는 접근이 끼어들 때" 에만 발생합니다.

#### 왜 이런 규칙이 필요한가? — C++ 의 iterator invalidation

```cpp
std::vector<int> v = {1, 2, 3};
int& first = v[0];       // v 의 내부 버퍼를 가리키는 참조
v.push_back(4);          // 용량 부족 -> 새 버퍼로 재할당, 옛 버퍼 해제!
std::cout << first;      // UB: 해제된 메모리를 읽음. 컴파일은 되고, 가끔만 터져서 더 무섭다
```

Rust 에서 같은 코드는 **컴파일 에러**입니다:

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];       // 읽기 빌림 시작
v.push(4);               // &mut 가 필요 -> 읽기 빌림이 아직 살아 있어서 충돌!
println!("{first}");     // 읽기 빌림이 여기까지 필요함
```
```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> b2.rs:4:5
  |
3 |     let first = &v[0];
  |                  - immutable borrow occurs here
4 |     v.push(4);
  |     ^^^^^^^^^ mutable borrow occurs here
5 |     println!("{first}");
  |                ----- immutable borrow later used here
```

"쓰는 쪽이 있으면 읽는 쪽은 없어야 한다" 라는 한 가지 규칙으로 **반복자 무효화, 해제 후 사용, 데이터 레이스(13장)** 가 한꺼번에 사라집니다.
부수 효과: `&mut` 가 *유일*하다는 보장 덕분에 컴파일러가 더 공격적으로 최적화할 수 있어요 (C 의 `restrict` 가 항상 켜져 있는 셈).

> ⚠️ **C++ 개발자가 헷갈리는 점**
> - C++ 의 `T&` 는 같은 객체에 몇 개든 만들 수 있고 컴파일러는 신경 쓰지 않습니다. Rust 의 `&mut T` 는 **독점 접근권**에 가까워요.
>   "참조를 만든다 = 락을 잡는다" 에 비유하면 이해가 쉽습니다 (읽기 락은 여러 개, 쓰기 락은 하나). 단 실행 중 비용은 0, 모두 컴파일 타임 검사입니다.
> - 처음엔 이 규칙 때문에 코드를 *설계*부터 다시 하게 됩니다 (예: 구조체의 두 필드를 따로 `&mut` 로 빌리려면 필드를 나눠서 빌려야 함). 그건 대개 "원래 설계가 aliasing 에 의존했다" 는 신호예요.
> - 읽기 참조 `&T` 는 `Copy` 라서 마음껏 복사되지만, `&mut T` 는 `Copy` 가 아닙니다 (복사하면 "하나뿐" 이 깨지니까요).

> ⚠️ **Python 개발자가 헷갈리는 점**
> Python 은 `def f(lst): lst.append(1)` 이 호출자의 리스트를 바꿔도 호출부(`f(a)`)에 아무 표시가 없습니다. Rust 는 `f(&mut a)` 로 *호출부에서도* 보이게 강제해요.
> 그리고 Python 의 슬라이스(`a[1:3]`)는 **복사본**이지만, Rust 의 슬라이스(`&a[1..3]`)는 **복사 없는 뷰**라서 원본을 수정하려면 빌림 규칙이 적용됩니다.

### 3. dangling 참조는 존재할 수 없다

```rust
fn dangle() -> &String {          // ❌ E0106
    let s = String::from("hi");
    &s                            // s 는 이 함수가 끝나면 drop 됨 -> 참조만 남으면 "허공을 가리킴"
}
```
```text
error[E0106]: missing lifetime specifier
 --> b3.rs:1:16
  |
1 | fn dangle() -> &String {
  |                ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
help: instead, you are more likely to want to return an owned value
  |
1 - fn dangle() -> &String {
1 + fn dangle() -> String {
```

해결은 소유권을 **반환**하는 것 (`-> String`, 3장!). 컴파일러가 이렇게 직접 제안합니다.
빌린 값을 반환하는 함수에서 참조가 가리키는 대상이 함수 *밖*에서 오는지 컴파일러가 이해하는 방법이 **수명(lifetime)** 이고, 9장에서 정식으로 다룹니다. 지금은 두 가지만 기억하세요.

- 입력 참조가 **하나**면, 반환 참조는 *그 입력에서 왔다* 고 컴파일러가 자동으로 이해합니다 (`fn longest_word(text: &str) -> &str` 처럼 수명 표기 생략 가능).
- 입력 참조가 **둘 이상**인데 참조를 반환하면 컴파일러가 "어느 쪽에서 온 거야?" 하고 묻습니다 (`E0106`, 🔬 실험 12). 그때 `'a` 표기가 필요해져요 (9장).

지역 변수에서 만든 값의 참조를 돌려주려 하는 다른 형태:

```text
error[E0515]: cannot return value referencing local variable `owned`
 --> b9.rs:3:5
  |
3 |     owned.split_whitespace().next().unwrap()
  |     -----^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |     |
  |     returns a value referencing data owned by the current function
  |     `owned` is borrowed here
```

스코프 밖으로 새는 참조 (`E0597`):

```rust
let r;
{
    let s = String::from("short-lived");
    r = &s;
}                        // s 는 여기서 drop
println!("{r}");         // r 은 죽은 s 를 가리킴
```
```text
error[E0597]: `s` does not live long enough
 --> b4.rs:5:13
  |
4 |         let s = String::from("short-lived");
  |             - binding `s` declared here
5 |         r = &s;
  |             ^^ borrowed value does not live long enough
6 |     }
  |     - `s` dropped here while still borrowed
7 |     println!("{r}");
  |                - borrow later used here
```

> 💡 **쉬운 설명 — 컴파일러는 "주소의 유통기한"을 본다**
> 주소(참조)는 건물이 있어야 의미가 있죠. 건물(값)이 철거(drop)된 뒤에도 주소(참조)만 들고 있으면 안 됩니다.
> C++ 은 이를 프로그래머의 주의력에 맡기고, Rust 는 컴파일러가 모든 참조에 대해 "가리키는 값이 이 참조보다 오래 사나?" 를 검사해요.

### 4. `String` vs `&str`

| | `String` | `&str` |
|---|---|---|
| 의미 | **소유**하는 문자열 (힙, 늘이고 줄일 수 있음) | 어딘가에 있는 UTF-8 글자의 **일부를 빌려 보는 뷰** |
| 메모리 | (ptr, len, cap) 24바이트 + 힙 데이터 | (ptr, len) 16바이트 (뚱뚱한 포인터, *fat pointer*) |
| 만드는 법 | `String::from("hi")`, `"hi".to_string()`, `format!(..)` | 문자열 리터럴 `"hi"` (`&'static str`), `&s[..]`, `s.as_str()` |
| C++ 대응 | `std::string` | `std::string_view` (단, 수명 검사가 있음) |
| 수정 | 가능 (`push_str`, `clear`, …) | 불가 (읽기 전용 뷰) |

```rust
fn main() {
    let literal: &str = "hello";              // 프로그램 바이너리에 박힌 글자를 가리킴
    let owned: String = literal.to_string();  // 힙에 복사해 새 소유자를 만듦
    let view: &str = &owned[1..3];            // owned 의 일부를 복사 없이 가리킴  -> "el"
    let whole: &str = &owned;                 // &String 이 &str 로 자동 변환됨 (deref 강제 변환)
    println!("{literal} {owned} {view} {whole}");
}
```

**함수 매개변수의 황금률: 읽기만 하면 `&str`, `&[T]` 로 받아라.**

```rust
fn word_count(text: &str) -> usize { text.split_whitespace().count() }

fn main() {
    let owned = String::from("a b c");
    println!("{}", word_count(&owned));      // &String -> &str 자동 변환
    println!("{}", word_count("literal d"));  // 리터럴도 OK
    println!("{}", word_count(&owned[2..]));  // 일부분도 OK
}
```

| 매개변수 타입 | 호출자가 받는 제약 | 언제? |
|---|---|---|
| `&str` | 거의 없음 (String, 리터럴, 슬라이스 모두 OK) | **읽기만 할 때 (기본값)** |
| `&String` | String 만 가능, 리터럴 불가 | 거의 쓰지 않음 (clippy 가 `&str` 을 권함) |
| `&mut String` | `String` 변수를 `mut` 로 가진 쪽 | 문자열을 제자리에서 늘리거나 고칠 때 |
| `String` | 소유권을 넘겨야 함 (또는 `clone`) | 함수가 문자열을 **저장/소비**할 때 |

반환 타입은 반대 방향 규칙: 입력의 일부면 `&str`(복사 없음, 수명은 입력과 같음), 새로 만든 값이면 `String`.

> ⚠️ **C++ 개발자가 헷갈리는 점**
> `std::string_view` 와 거의 같은 아이디어지만 결정적 차이는 "**원본이 먼저 죽는 코드는 컴파일이 안 된다**" 는 점입니다.
> C++ 에서 `std::string_view f() { std::string s = "..."; return s; }` 는 경고 없이 컴파일되고 쓰레기를 출력해요 (`cpp/main.cpp` 의 주석 예시, 직접 확인했습니다).

### 5. 슬라이스: `&[T]` 와 `&mut [T]`

```rust
fn sum_slice(nums: &[i32]) -> i32 { nums.iter().sum() }
fn scale(nums: &mut [i32], k: i32) { for n in nums.iter_mut() { *n *= k; } }

fn main() {
    let mut v = vec![1, 2, 3, 4, 5];
    println!("{}", sum_slice(&v));        // 15   Vec 전체 (&Vec<i32> -> &[i32])
    println!("{}", sum_slice(&v[..3]));   // 6    앞 3개 (범위 문법 ..3, 1..4, 2.., ..=3)
    scale(&mut v[3..], 10);               // 뒤 2개만 수정
    println!("{v:?}");                    // [1, 2, 3, 40, 50]
    println!("{}", sum_slice(&[7, 8]));   // 15   배열 리터럴도 OK
}
```

- **슬라이스 = 연속된 메모리의 일부를 가리키는 (포인터, 길이)** 입니다. Vec, 배열 어디서든 만들 수 있고 복사하지 않아요 (`std::span` 과 같음).
- 범위가 데이터를 벗어나면 **패닉** (`range end index 7 out of range for slice of length 5`). C++ 의 UB 가 아니에요.
- 한 번에 두 구간을 `&mut` 로 빌리고 싶다면 `split_at_mut` 으로 겹치지 않는다는 것을 증명해 줍니다:

```rust
fn main() {
    let mut nums = [1, 2, 3, 4, 5, 6];
    let (left, right) = nums.split_at_mut(3);   // 겹치지 않는 두 &mut 조각
    left[0] = 10;
    right[0] = 40;
    println!("{nums:?}");                        // [10, 2, 3, 40, 5, 6]
}
```

### 6. UTF-8 — 바이트와 글자는 다르다

Rust 의 `String` / `&str` 은 **항상 올바른 UTF-8** 입니다. 영어(ASCII)는 글자당 1바이트, 한글 대부분은 **3바이트**예요.

```rust
fn main() {
    let k = "가나다";
    println!("{} {}", k.len(), k.chars().count());                  // 9 3     (바이트, 글자)
    println!("{:?}", k.char_indices().collect::<Vec<_>>());         // [(0, '가'), (3, '나'), (6, '다')]
    println!("{:?} {:?}", k.chars().nth(1), k.bytes().nth(1));      // Some('나') Some(176)
    println!("{}", &k[0..3]);                                       // 가   (바이트 범위, 글자 경계)
    // println!("{}", &k[0..2]);                                    // ❌ 실행 중 패닉! 글자 한가운데
    // let c = k[0];                                                // ❌ E0277: 문자열은 정수로 인덱싱 불가
}
```

```text
thread 'main' panicked at b12.rs:3:18:
end byte index 2 is not a char boundary; it is inside '안' (bytes 0..3 of string)
```

| 방법 | 단위 | 비용 |
|---|---|---|
| `s.len()` | 바이트 수 | O(1) |
| `s.chars().count()` | 글자(유니코드 스칼라 값) 수 | O(n) |
| `s.chars()` / `s.char_indices()` | 글자 순회 / (바이트 위치, 글자) 순회 | |
| `s.bytes()` | 바이트 순회 | |
| `&s[a..b]` | **바이트** 범위 슬라이스. 경계가 글자 중간이면 패닉 | O(1) |
| `s.char_indices().nth(n)` | n 번째 글자의 바이트 위치 찾기 | O(n) |

**왜 `s[0]` 이 막혀 있을까요?** "n 번째 글자" 를 O(1) 로 못 찾고(UTF-8 은 가변 길이), 바이트 하나는 글자가 아닐 수도 있으며, 어떤 의미("바이트? 글자? 화면에 보이는 한 칸?")를 원하는지 모호하기 때문입니다.
Python 은 `s[0]` 이 편한 대신 내부적으로 문자열 전체를 글자당 1·2·4바이트 고정 폭 중 하나로 저장해서(PEP 393) 한글/이모지가 섞이면 메모리를 더 쓰고, Rust/C++ 는 UTF-8 로 저장해서 작습니다. `len()` 의 의미가 달라서 **Python → Rust 이식 시 버그의 단골**이에요.

> 💡 **쉬운 설명 — 바이트, 글자, 그리고 "보이는 한 글자"**
> 바이트는 컴퓨터가 저장하는 단위, 글자(`char`)는 유니코드 한 칸입니다. 'A' 는 1바이트, '가' 는 3바이트, 이모지는 4바이트예요.
> 더 복잡하게는 "한글 자모가 분리된 형태(ㅎ+ㅏ+ㄴ)나 이모지 조합"처럼 *화면에서 한 글자로 보이지만 여러 `char` 인 경우*(grapheme cluster)도 있습니다.
> std 는 `char` 까지만 지원하고, 그 위는 `unicode-segmentation` crate 의 몫입니다. 이 실습의 한글은 "완성형"이라 `char` 하나가 글자 하나예요.

### 7. 설계 가이드: 소유? 빌림? 가변 빌림?

함수를 쓸 때 매번 이 순서로 자문하세요.

1. 이 함수가 값을 **저장하거나 소비**하나? → 소유권을 받는다 (`String`, `Vec<T>`, `T`).
2. 읽기만 하나? → `&T` (`&str`, `&[T]`).
3. 호출자의 값을 **고쳐야** 하나? → `&mut T`.
4. 새 값을 **만들어서** 돌려주나? → 반환 타입은 소유 타입 (`String`, `Vec<T>`).
5. 입력의 **일부**를 돌려주나? → 반환 타입은 `&str` / `&[T]` (수명이 입력과 묶임).

이 5개가 Rust 시그니처 읽기의 80% 입니다. AI 가 만든 코드를 리뷰할 때도 가장 먼저 함수 시그니처에서 이 질문들을 해 보세요.

---

## 🧯 자주 만나는 컴파일 에러

모두 **실제 rustc 1.97 출력**이고, 🔬 실험실에서 재현할 수 있습니다.

### 1) `&mut` 가 동시에 둘 — `E0499`

```rust
let mut s = String::from("hello");
let a = &mut s;
let b = &mut s;
a.push('!');
b.push('?');
```
```text
error[E0499]: cannot borrow `s` as mutable more than once at a time
 --> b1.rs:4:13
  |
3 |     let a = &mut s;
  |             ------ first mutable borrow occurs here
4 |     let b = &mut s;
  |             ^^^^^^ second mutable borrow occurs here
5 |     a.push('!');
  |     - first borrow later used here
```
**읽는 법**: 첫 번째 빌림 → 두 번째 빌림(충돌) → 첫 번째가 *아직 쓰이는* 곳. **고치기**: 하나를 쓰고 끝낸 뒤에 다음 빌림을 만들기 (순서 바꾸기), 또는 `a` 만 쓰도록 정리.

### 2) 읽기 빌림이 살아 있는데 수정 — `E0502`

위 "이론 2" 의 `first = &v[0]; v.push(4)` 예시. 순회 중 수정도 같은 에러입니다:

```rust
let mut v = vec![1, 2, 3];
for x in &v {
    if *x == 2 { v.push(10); }
}
```
```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> b14.rs:4:22
  |
3 |     for x in &v {
  |              --
  |              |
  |              immutable borrow occurs here
  |              immutable borrow later used here
4 |         if *x == 2 { v.push(10); }
  |                      ^^^^^^^^^^ mutable borrow occurs here
```
**고치기**: 바꿀 항목을 먼저 모은 뒤 순회가 끝나고 수정 (`let to_add: Vec<_> = ...; v.extend(to_add);`), 또는 `retain` / `iter_mut` 같은 전용 API 사용.

고전적인 슬라이스 버전 (The Rust Book 의 `first_word` 문제):

```rust
let mut s = String::from("hello world");
let word = first_word(&s);   // &str: s 를 가리킴
s.clear();                   // &mut 필요 -> 충돌!
println!("{word}");
```
```text
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
 --> b15.rs:4:5
  |
3 |     let word = first_word(&s);
  |                           -- immutable borrow occurs here
4 |     s.clear();
  |     ^^^^^^^^^ mutable borrow occurs here
5 |     println!("{word}");
  |                ---- immutable borrow later used here
```
`clear()` 후에 `word` 가 가리키는 글자들이 사라질 테니까요. Python 이었다면 `word` 는 복사본이라 아무 일 없었겠지만, 대신 복사 비용이 들었죠.

### 3) `mut` 를 안 붙이고 수정 — `E0596`

```text
error[E0596]: cannot borrow `s` as mutable, as it is not declared as mutable
 --> b6.rs:3:5
  |
3 |     s.push('!');
  |     ^ cannot borrow as mutable
help: consider changing this to be mutable
  |
2 |     let mut s = String::from("hi");
  |         +++
```
`&` 로 받은 것을 고치려 해도 같은 코드입니다:

```text
error[E0596]: cannot borrow `*s` as mutable, as it is behind a `&` reference
 --> b5.rs:2:5
  |
2 |     s.push('!');
  |     ^ `s` is a `&` reference, so it cannot be borrowed as mutable
help: consider changing this to be a mutable reference
  |
1 | fn add_exclaim(s: &mut String) {
  |                    +++
```

### 4) 수명이 모호한 참조 반환 — `E0106`

```rust
fn longest(a: &str, b: &str) -> &str {
    if a.len() >= b.len() { a } else { b }
}
```
```text
error[E0106]: missing lifetime specifier
 --> b7.rs:1:33
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
9장에서 `'a` 를 배우면 이 제안이 무슨 뜻인지 이해됩니다. 지금은 "컴파일러가 해결책을 이미 알려 주고 있다" 만 기억하세요.

### 5) 지역 변수의 참조를 반환 — `E0106` / `E0515` / `E0597`

이론 3 절에 실제 출력이 있습니다. 공통 해법은 **소유한 값을 반환**(`String`)하거나, 입력에서 온 참조를 돌려주는 구조로 바꾸는 것입니다.

### 6) `&str` 와 `String` 혼동 — `E0308`

```rust
fn takes_string(s: String) { println!("{s}"); }
takes_string("literal");
```
```text
error[E0308]: mismatched types
 --> b13.rs:4:18
  |
4 |     takes_string(t);
  |     ------------ ^ expected `String`, found `&str`
  |     |
  |     arguments to this function are incorrect
help: try using a conversion method
  |
4 |     takes_string(t.to_string());
  |                   ++++++++++++
```
소유가 정말 필요한 함수라면 제안대로 `.to_string()` (힙 할당 + 복사). 읽기만 하는 함수라면 **함수 쪽을 `&str` 로 바꾸는 것**이 정답입니다.

### 7) 문자열 인덱싱 — `E0277`

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
 --> b11.rs:3:15
  |
3 |     let c = s[0];
  |               ^ string indices are ranges of `usize`
  |
  = note: you can use `.chars().nth()` or `.bytes().nth()`
          for more information, see chapter 8 in The Book: ...
```

### 8) 빌려 준 동안 이동 — `E0505`

```text
error[E0505]: cannot move out of `s` because it is borrowed
 --> b10.rs:4:17
  |
2 |     let s = String::from("hello");
  |         - binding `s` declared here
3 |     let r = &s;
  |             -- borrow of `s` occurs here
4 |     let moved = s;
  |                 ^ move out of `s` occurs here
5 |     println!("{r} {moved}");
  |                - borrow later used here
```
3장(이동)과 4장(빌림)의 만남입니다: 누가 빌려 보고 있는 값의 소유권을 넘겨 버리면 빌린 쪽이 허공을 가리키게 되니 금지.

---

## 🛠️ 실습: 텍스트 분석기

### 무엇을 만드나요

영어 문장과 한글 문장을 분석하는 작은 도구입니다. 모든 함수는 **문자열을 복사하지 않고 빌려서** 일합니다 (새 값을 만들어야 하는 `reverse_chars` 와 `hex_bytes` 제외).

```text
== 1. text stats ==
text: The quick brown fox jumps over the lazy dog. It was a sunny day! Did you enjoy the afternoon?
words: 19
longest: afternoon                  <- 원본 문장 속 "afternoon" 을 가리키는 &str (복사 없음)
longest (tie): one                  <- 길이가 같으면 먼저 나온 단어
first sentence: The quick brown fox jumps over the lazy dog.
== 2. normalize in place ==
before: "  Hello,   WORLD   from  Rust!  "
after: "hello, world from rust!"     <- &mut String 으로 제자리 수정
== 3. slices of numbers ==
after scale x3: [3, 6, 9, 12, 15]   <- &mut [i32]
first three sum: 18                  <- &nums[..3]
== 4. UTF-8 ==
text: 안녕하세요 러스트
bytes: 25
chars: 9                             <- 바이트 25 != 글자 9
first 3 chars: 안녕하
first byte-slice [0..3]: 안
reversed: 트스러 요세하녕안
bytes of 가: ea b0 80
```

### 파일 구성

| 파일 | 역할 |
|---|---|
| `python/main.py` | Python 참조 구현 (문자열 불변, 슬라이스 = 복사, `len` = 글자 수) |
| `cpp/main.cpp` | C++20 참조 구현 (`string_view`, `span`, UTF-8 직접 처리, dangling 예시 주석) |
| `rust/src/main.rs` | ★ 여러분이 채울 파일 (TODO 10개 + 🔬 실험 13개) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법

```bash
cd chapters/ch04_borrowing
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch04_borrowing                         # TODO 가 남아 있으면 패닉(정상)
cargo test -p ch04_borrowing
cargo run -q -p ch04_borrowing --example solution
../../scripts/compare.sh ch04                          # 내 코드 비교
../../scripts/compare.sh ch04 --solution               # 정답 비교
```

### TODO 목록

| 번호 | 내용 | 관련 개념 |
|---|---|---|
| TODO(1) | `word_count` | `&str` 매개변수, 이터레이터 |
| TODO(2) | `longest_word` → `&str` | **원본의 슬라이스 반환**, 수명 생략, `max_by_key` 함정 |
| TODO(3) | `first_sentence` → `&str` | `find`, 범위 슬라이스 `&text[..=i]` |
| TODO(4) | `normalize(&mut String)` | **`&mut` 로 제자리 수정**, NLL |
| TODO(5) | `scale_in_place(&mut [i32], k)` | `&mut [T]`, `iter_mut`, `*n` |
| TODO(6) | `sum_slice(&[i32])` | `&[T]`, Vec/배열/부분 슬라이스 |
| TODO(7) | `char_count` | 바이트 vs 글자 |
| TODO(8) | `first_n_chars` → `&str` | `char_indices`, 글자 경계 |
| TODO(9) | `reverse_chars` → `String` | `chars().rev().collect()` |
| TODO(10) | `hex_bytes` | `bytes()`, `format!("{b:02x}")` |

### 완료 기준

- `cargo test -p ch04_borrowing` 10개 모두 `ok`
- `../../scripts/compare.sh ch04` 에서 `✅ python vs rust : 출력이 똑같아요` (한글 포함 바이트 단위 동일)

### 🔬 실험 목록 (`rust/src/main.rs` 의 `experiments()`)

| 실험 | 해볼 것 | 기대 결과 |
|---|---|---|
| 1 | `&mut s` 를 두 번 | `E0499` |
| 2 | `&v[0]` 를 들고 `v.push` | `E0502` |
| 3 | `for x in &v` 안에서 `v.push` | `E0502` |
| 4 | `mut` 없이 / `&String` 으로 받아 수정 | `E0596` |
| 5 | `fn dangle() -> &String` | `E0106` |
| 6 | 지역 `String` 의 일부를 반환 | `E0515` |
| 7 | 안쪽 스코프의 값을 바깥 참조에 담기 | `E0597` |
| 8 | 빌려 준 값을 `let moved = s;` | `E0505` |
| 9 | `String` 자리에 `&str` | `E0308` |
| 10 | `s[0]` | `E0277` |
| 11 | 한글을 `&s[0..2]` 로 자르기 | **런타임 패닉** (char boundary) |
| 12 | 참조 인자 둘 + 참조 반환 | `E0106` (→ 9장 예고) |
| 13 | 슬라이스를 든 채 `s.clear()` | `E0502` |

실험을 하면서 **각 에러에 대해 "C++ 이라면 이 코드의 결과는?"** 을 한 줄씩 적어 보세요. 대부분 "컴파일되고, 가끔 터짐" 일 겁니다. 그것이 Rust 가 해 주는 일입니다.

**추가 도전**: `longest_word` 를 `max_by_key(|w| w.len())` 로 구현해 보세요. 어떤 테스트가 왜 깨지나요? (`max_by_key` 는 동률이면 *마지막* 요소, `min_by_key` 는 *첫* 요소를 돌려줘요. `rev()` 와 조합하거나 직접 루프로 해결)

---

## 🤖 바이브 코딩 포인트

- **함수 시그니처의 `String` / `&String` / `Vec<T>` / `&Vec<T>` 를 찾으세요.** 읽기만 하는 함수가 `&String` 이나 `&Vec<T>` 를 받고 있으면 `&str` / `&[T]` 로 바꿔 달라고 하세요 (clippy 의 `ptr_arg` 경고와 같은 지적). 소유를 받는 함수는 정말 저장/소비하는지 확인합니다.
- **`.to_string()` / `.clone()` / `String::from` 이 반환값을 만들려고 쓰였는지** 보세요. 입력의 일부를 돌려주는 함수라면 `&str` 반환(복사 없음)이 가능한지 물어보세요: *"할당 없이 입력 슬라이스를 반환하도록 바꿔 줘."*
- **`&mut` 가 많은 시그니처, `&mut self` 가 많은 코드**는 상태가 퍼져 있다는 신호입니다. 한 함수가 여러 곳을 동시에 `&mut` 로 잡으려 해서 AI 가 `clone`, `Rc<RefCell<>>`, 인덱스 우회 등으로 도망쳤는지 확인하세요.
- **`unsafe`, `raw pointer`, `transmute`, `'static` 남용**: 빌림 에러를 만나 AI 가 수명 문제를 `'static` 이나 `unsafe` 로 덮었다면 거부하세요. 거의 항상 설계(소유/빌림 경계) 문제입니다. 프롬프트: *"unsafe/'static 없이, 소유권을 반환하거나 구조를 바꿔서 빌림 에러를 해결해 줘."*
- **문자열 처리에서 `len()` / `[i..j]` 를 쓰는 곳**을 보세요. 한글/이모지가 들어올 수 있다면 `chars()` / `char_indices()` / `get(i..j)` (경계가 아니면 `None` 을 돌려주는 안전한 슬라이싱) 를 쓰는지 확인합니다. `&s[a..b]` 가 사용자 입력에 쓰이면 패닉 위험이에요.
- **컴파일 에러(E0499/E0502)를 AI 에게 붙여 줄 때**는 에러 전문과 함께 *"의도: 무엇을 수정하려 했는지"* 를 한 줄 덧붙이세요. AI 가 단순 clone 으로 때우지 않고 순서 변경/범위 축소 같은 정석 해법을 고릅니다.
- 순회하면서 수정하려는 AI 코드는 `retain`, `iter_mut`, `drain`, 인덱스 순회 중 어떤 방식을 택했는지 확인하세요.

---

## ✅ 셀프 체크

**Q1.** 아래 코드가 컴파일되는지 판단하고, 안 된다면 어떤 줄이 문제인지 말해 보세요.

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
println!("{first}");
v.push(4);
```

<details><summary>답</summary>

**컴파일됩니다.** `first` 의 마지막 사용이 `println!` 이므로 NLL 에 의해 읽기 빌림이 거기서 끝나고, 이후의 `v.push(4)` 는 충돌하지 않아요. `println!` 과 `push` 의 순서를 바꾸면 `E0502` 입니다.
</details>

**Q2.** 왜 읽기만 하는 함수의 매개변수는 `&String` 이 아니라 `&str` 이 좋을까요?

<details><summary>답</summary>

`&str` 은 `&String`, 문자열 리터럴, 문자열의 일부(`&s[1..3]`) 를 모두 받을 수 있어 호출자의 제약이 적습니다 (`&String` 은 자동으로 `&str` 로 바뀝니다). `&String` 은 `String` 변수만 받을 수 있고 추가로 이중 포인터라 비효율적이에요.
</details>

**Q3.** `"안녕하세요".len()` 과 `"안녕하세요".chars().count()` 는 각각 얼마인가요? `&"안녕하세요"[0..2]` 는?

<details><summary>답</summary>

`15` 와 `5`. 한글 한 글자는 UTF-8 로 3바이트입니다. `[0..2]` 는 첫 글자 '안'(바이트 0..3) 의 한가운데라서 **런타임 패닉**(`byte index 2 is not a char boundary`)입니다. 안전하게 하려면 `s.get(0..2)` (None 반환) 또는 `char_indices` 로 경계를 찾으세요.
</details>

**Q4.** C++ 의 `std::string_view` 와 Rust 의 `&str` 의 가장 큰 차이는?

<details><summary>답</summary>

모양(포인터+길이)은 같지만, Rust 는 `&str` 이 가리키는 원본보다 오래 살지 못하도록 **컴파일러가 수명을 검사**합니다. C++ 의 `string_view` 는 원본이 먼저 사라져도 컴파일되고 UB 가 됩니다.
</details>

**Q5.** `fn longest(a: &str, b: &str) -> &str` 이 컴파일 에러인 이유는 무엇이고, 컴파일러가 제안하는 고침은 무엇일까요? (힌트: 9장 예고)

<details><summary>답</summary>

반환 참조가 `a` 에서 온 건지 `b` 에서 온 건지 시그니처만으로 알 수 없어서 `E0106`. 제안은 수명 매개변수를 선언해 "두 입력과 출력이 같은 수명"임을 적는 것: `fn longest<'a>(a: &'a str, b: &'a str) -> &'a str` (9장).
</details>

**Q6.** `fn normalize(s: &mut String)` 를 호출하는 쪽에서 `let s = String::from("A"); normalize(&mut s);` 라고 쓰면 어떤 에러가 날까요?

<details><summary>답</summary>

`E0596: cannot borrow `s` as mutable, as it is not declared as mutable`. `s` 를 `let mut s` 로 선언해야 `&mut s` 를 만들 수 있어요. 즉 "고쳐도 되는 값" 이라는 의사 표시가 소유자 쪽에도 필요합니다.
</details>

---

## 📚 더 알아보기

- The Rust Book 4.2 *References and Borrowing*: <https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html>
- The Rust Book 4.3 *The Slice Type*: <https://doc.rust-lang.org/book/ch04-03-slices.html>
- The Rust Book 8.2 *Storing UTF-8 Encoded Text with Strings*: <https://doc.rust-lang.org/book/ch08-02-strings.html>
- Rust by Example — *Borrowing*: <https://doc.rust-lang.org/rust-by-example/scope/borrow.html>
- std 문서: [`str`](https://doc.rust-lang.org/std/primitive.str.html) (메서드 목록이 길지만 훑어볼 가치가 있어요), [`slice`](https://doc.rust-lang.org/std/primitive.slice.html), [`char`](https://doc.rust-lang.org/std/primitive.char.html)
- 이어서: 9장 *Lifetimes* — 이 장의 `E0106` 을 정식으로 해결합니다.
