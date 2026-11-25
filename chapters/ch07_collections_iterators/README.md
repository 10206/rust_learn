# Ch07. 컬렉션·클로저·이터레이터

> `Vec`/`HashMap`/`HashSet` 으로 데이터를 담고, 클로저와 이터레이터 체인으로 "루프 없이" 변환한다 — 그리고 그것이 손으로 쓴 루프만큼 빠르다.  ·  예상 소요: 이론 50분 / 실습 70분  ·  선행: Ch06 (에러 처리), Ch04 (빌림)

## 🎯 학습 목표
- `Vec`, `HashMap`, `HashSet`, `BTreeMap` 의 차이를 알고 용도에 맞게 고른다.
- `entry` API 로 "있으면 수정, 없으면 삽입"을 한 번의 조회로 처리한다.
- 클로저가 주변 변수를 **빌리는지(`Fn`/`FnMut`) / 가져가는지(`FnOnce`, `move`)** 읽고 설명한다.
- `iter()` / `iter_mut()` / `into_iter()` 의 차이를 소유권 관점에서 설명한다.
- `map`/`filter`/`filter_map`/`enumerate`/`zip`/`fold`/`sum`/`collect` 체인을 읽고 쓴다. 이터레이터가 **게으르다(lazy)** 는 것을 안다.

## 🔁 C++ / Python 과 한눈에 비교

| 개념 | C++20 | Python | Rust |
|---|---|---|---|
| 동적 배열 | `std::vector<T>` | `list` | `Vec<T>` |
| 해시맵 | `std::unordered_map` | `dict` (삽입 순서 유지) | `HashMap<K, V>` (**순서 무작위**) |
| 정렬된 맵 | `std::map` | (없음, `sorted()`) | `BTreeMap<K, V>` |
| 집합 | `std::unordered_set` / `std::set` | `set` | `HashSet<T>` / `BTreeSet<T>` |
| 없으면 삽입 | `m[k]` (기본값 생성) / `try_emplace` | `defaultdict`, `setdefault` | `m.entry(k).or_insert(v)` |
| 익명 함수 | `[캡처](인자){ }` | `lambda x: ...` | `\|x\| ...` / `move \|x\| ...` |
| 변환 체인 | `v \| views::filter \| views::transform` (C++20) | 컴프리헨션 / 제너레이터 | `v.iter().filter().map().collect()` |
| 체인 → 컨테이너 | `ranges::to` (C++23) / 수동 루프 | `list(...)`, `[... for ...]` | `.collect::<Vec<_>>()` |
| 누적 | `std::accumulate` | `sum`, `functools.reduce` | `.sum()`, `.fold(init, f)` |
| 순회 중 수정 | 반복자 무효화 → **UB** | 런타임 에러/조용한 버그 | **컴파일 에러** |

## 📘 이론

> 코드 조각은 <https://play.rust-lang.org> 에서 그대로 실행됩니다 (표준 라이브러리만 사용).

### 1. 컬렉션 4총사

```rust
use std::collections::{BTreeMap, HashMap, HashSet};

fn main() {
    // Vec: 연속된 메모리의 가변 길이 배열
    let mut v = vec![3, 1, 2];
    v.push(4);
    v.sort();
    println!("{v:?} len={} first={:?}", v.len(), v.first()); // [1, 2, 3, 4] len=4 first=Some(1)
    // v[10] 은 panic. 안전하게 접근: v.get(10) -> Option<&i32>

    // HashMap: 키 -> 값. 순회 순서는 실행마다 다를 수 있음!
    let mut ages: HashMap<String, u32> = HashMap::new();
    ages.insert("alice".to_string(), 30);
    println!("{:?}", ages.get("alice"));         // Some(30)  (&str 로 조회 가능)
    println!("{:?}", ages.get("zed"));           // None

    // BTreeMap: 키 순서대로 정렬되어 있음 → 출력이 결정적
    let mut sorted = BTreeMap::new();
    sorted.insert("b", 2);
    sorted.insert("a", 1);
    println!("{sorted:?}");                      // {"a": 1, "b": 2}

    // HashSet: 중복 없는 집합
    let set: HashSet<i32> = [1, 2, 2, 3].into_iter().collect();
    println!("{}", set.len());                   // 3
}
```

| 컨테이너 | 언제 | 비용 |
|---|---|---|
| `Vec<T>` | 기본 선택. 순서 있는 목록 | 끝 삽입 O(1) |
| `HashMap<K,V>` | 키로 빠른 조회 | 평균 O(1), **순서 없음** |
| `BTreeMap<K,V>` | 정렬된 순회, 범위 조회, 결정적 출력 | O(log n) |
| `HashSet<T>` / `BTreeSet<T>` | 중복 제거, 포함 여부 | 위와 같음 |

💡 **쉬운 설명** — Python 의 `dict` 는 삽입 순서를 기억하지만 Rust 의 `HashMap` 은 **의도적으로 순서를 섞습니다** (해시 공격 방어용 무작위 시드). 그래서 "출력 순서가 중요하면 `BTreeMap` 을 쓰거나 키를 모아 정렬"하는 것이 습관입니다. 이 챕터의 실습도 그렇게 합니다.

⚠️ **C++ 개발자가 헷갈리는 점**
- `HashMap` 의 `m[key]` 는 키가 없으면 **panic** 입니다 (C++ `operator[]` 처럼 기본값을 만들지 않아요). 삽입은 `insert`/`entry`, 조회는 `get` (→ `Option<&V>`).
- 정렬(`sort`)은 컨테이너 메서드, `f64` 는 `NaN` 때문에 `sort()` 불가 → `sort_by(|a, b| a.total_cmp(b))`.

### 2. `entry` API — "있으면 수정, 없으면 삽입"

단어 수 세기:

```rust
use std::collections::HashMap;

fn main() {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for w in "a b a c b a".split_whitespace() {
        *counts.entry(w).or_insert(0) += 1;      // 없으면 0 삽입, 있든 없든 &mut usize 반환
    }
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort();                                 // HashMap 은 순서가 없으니 정렬 후 출력
    println!("{pairs:?}");                        // [("a", 3), ("b", 2), ("c", 1)]
}
```

그룹핑은 `entry(k).or_default().push(v)`:

```rust
let mut groups: std::collections::BTreeMap<char, Vec<&str>> = Default::default();
for w in ["apple", "avocado", "banana"] {
    groups.entry(w.chars().next().unwrap()).or_default().push(w);
}
// {'a': ["apple", "avocado"], 'b': ["banana"]}
```

`and_modify(|v| ...).or_insert(초기값)` 은 "있으면 이렇게 고치고, 없으면 이걸 넣어" 입니다.

💡 **쉬운 설명** — `entry` 는 "그 키의 칸"을 가리키는 손잡이입니다. 칸이 비어 있으면 채우고, 어쨌든 그 칸의 내용물에 대한 `&mut` 를 돌려줍니다. `if (!m.contains(k)) m[k] = ..; m[k] += 1;` 처럼 **두 번 찾지 않아도** 됩니다.

### 3. 클로저 — 주변 변수를 잡는 익명 함수

```rust
fn main() {
    let factor = 3;
    let times = |x: i32| x * factor;     // factor 를 "빌려서" 캡처 (불변 참조)
    println!("{}", times(5));            // 15

    let mut calls = 0;
    let mut counter = || { calls += 1; calls };   // calls 를 "&mut 로 빌려" 캡처
    counter();
    counter();
    println!("{calls}");                 // 2  (counter 를 더는 안 쓰므로 빌림이 끝남)
}
```

클로저가 변수를 **어떻게 쓰느냐**에 따라 컴파일러가 자동으로 trait 을 붙입니다.

| trait | 클로저가 하는 일 | 몇 번 부를 수 있나 | C++ 대응 |
|---|---|---|---|
| `Fn` | 캡처한 값을 **읽기만** | 여러 번, 동시에도 | `[&]`/`[=]` 읽기 전용 람다, `const operator()` |
| `FnMut` | 캡처한 값을 **수정** | 여러 번 (`mut` 로 선언 필요) | `[&]` 로 수정하는 람다 / `mutable` 람다 |
| `FnOnce` | 캡처한 값을 **소비(move out)** | 딱 한 번 | `std::move` 로 캡처값을 넘겨 버리는 람다 |

`Fn ⊂ FnMut ⊂ FnOnce` 관계라서, 함수 인자에서 `impl FnOnce` 가 가장 넓게(관대하게) 받고 `impl Fn` 이 가장 엄격합니다. 함수가 *한 번만* 부른다면 `FnOnce`, 여러 번 부른다면 `FnMut`/`Fn` 으로 받는 게 일반적입니다.

**`move` 클로저** — 캡처를 *빌리지 않고 가져갑니다*. 클로저가 만든 함수보다 오래 살아야 할 때(반환, 스레드로 전달) 필수입니다.

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n          // move 가 없으면 E0373: n 이 함수 끝에서 사라지는데 빌리고 있음
}
```

```cpp
// C++ 에서는 같은 실수가 컴파일되고, 실행하면 dangling 입니다:
auto make_adder(int n) { return [&n](int x) { return x + n; }; }   // ❌ n 은 이미 소멸
```

⚠️ **C++ 개발자가 헷갈리는 점**
- C++ 은 캡처 방식을 **직접 적고**(`[&]`, `[=]`, `[x = std::move(x)]`) 틀려도 컴파일됩니다. Rust 는 **사용 방식에서 추론**하고(읽기 → `&`, 수정 → `&mut`, 소비 → move), 위험하면 컴파일 에러로 막습니다. `move` 는 "빌리지 말고 가져가라"는 명시 스위치입니다.
- 클로저마다 **고유한 익명 타입**을 가집니다. 그래서 반환/인자 타입에 `impl Fn(i32) -> i32` 또는 제네릭 `F: Fn(i32) -> i32`, 여러 종류를 한 컨테이너에 담을 땐 `Box<dyn Fn(i32) -> i32>` 를 씁니다 (Ch08).

### 4. 이터레이터 — 세 가지 순회 방법

```rust
fn main() {
    let mut v = vec![10, 20, 30];

    for x in v.iter() { println!("{x}"); }        // &i32 : 읽기 전용 빌림 (v 는 계속 사용 가능)
    for x in v.iter_mut() { *x += 1; }            // &mut i32 : 제자리 수정
    for x in v.into_iter() { println!("{x}"); }   // i32 : 소유권을 가져감 (v 는 이후 사용 불가)
    // println!("{v:?}");  // ❌ E0382: borrow of moved value: `v`
}
```

`for x in &v` 는 `v.iter()`, `for x in &mut v` 는 `v.iter_mut()`, `for x in v` 는 `v.into_iter()` 의 줄임입니다.

### 5. 어댑터 체인과 게으름(laziness)

```rust
fn main() {
    let scores = vec![92, 55, 78, 61, 100];

    let passed: Vec<u32> = scores
        .iter()                       // 이터레이터 생성 (아직 아무것도 실행 안 됨)
        .copied()                     // &u32 -> u32
        .filter(|&s| s >= 60)         // 어댑터: 조건에 맞는 것만
        .map(|s| s + 5)               // 어댑터: 변환
        .collect();                   // 소비자(consumer): 여기서 비로소 실행
    println!("{passed:?}");           // [97, 83, 66, 105]

    let total: u32 = scores.iter().sum();                              // 소비자
    let avg = total as f64 / scores.len() as f64;
    let best = scores.iter().max();                                    // Option<&u32>
    let first_fail = scores.iter().position(|&s| s < 60);              // Some(1): 찾으면 바로 멈춤 (short-circuit)
    let any_perfect = scores.iter().any(|&s| s == 100);                // true
    println!("{avg:.1} {best:?} {first_fail:?} {any_perfect}");

    for (i, s) in scores.iter().enumerate() { println!("{i}: {s}"); }  // 인덱스와 함께
    let names = ["a", "b"];
    let pairs: Vec<(&str, &u32)> = names.iter().copied().zip(scores.iter()).collect();  // 짝짓기
    let sum_sq = scores.iter().fold(0u32, |acc, &s| acc + s * s);      // 직접 누적
    println!("{pairs:?} {sum_sq}");
}
```

| 구분 | 메서드 |
|---|---|
| **어댑터** (게으름, 새 이터레이터 반환) | `map`, `filter`, `filter_map`, `enumerate`, `zip`, `skip`, `take`, `rev`, `chain`, `flat_map`, `copied`, `cloned`, `take_while` |
| **소비자** (실제 실행) | `collect`, `sum`, `count`, `max`/`min`, `fold`, `any`/`all`, `find`, `position`, `for_each`, `last` |

`collect()` 가 어떤 컨테이너를 만들지는 **타입 표기**가 정합니다: `Vec<_>`, `HashSet<_>`, `HashMap<_, _>`, `String`, 그리고 `Result<Vec<_>, E>` (첫 에러에서 멈춤!) 도 됩니다.

```rust
let nums: Result<Vec<i32>, _> = "1,2,x".split(',').map(|s| s.parse::<i32>()).collect();
// Err(ParseIntError { .. })  — 하나라도 실패하면 전체가 Err (Ch06 의 Result 와 연결)
```

**왜 "zero-cost" 인가요?** 위의 `filter().map().sum()` 체인은 컴파일 때 **하나의 루프**로 합쳐져 인라인됩니다. 중간 `Vec` 도 임시 객체도 없고, 숙련된 사람이 손으로 쓴 `for` 루프와 거의 같은 기계어가 나옵니다. C++20 `views`, Python 제너레이터와 같은 아이디어지만 Rust 는 **모든 타입이 컴파일 타임에 확정**되고 빌림 검사가 적용되는 점이 다릅니다.

💡 **쉬운 설명** — 이터레이터 체인은 "공장 라인의 설계도"입니다. `.filter().map()` 까지는 설계도만 그리고, `.collect()` 같은 소비자가 "생산 시작!" 버튼을 눌러야 물건이 하나씩 라인을 통과합니다. 그래서 `find` 처럼 찾으면 멈추는 소비자는 나머지 원소를 아예 보지 않습니다.

⚠️ **C++ 개발자가 헷갈리는 점**
- 이터레이터 무효화(순회 중 `push_back` → UB)가 Rust 에서는 **컴파일 에러**(E0502)입니다. 순회가 컨테이너를 빌리고 있어서 동시에 수정할 수 없기 때문입니다.
- `iter()` 는 참조(`&T`)를 줍니다. `.map(|x| x * 2)` 에서 `x` 가 `&i32` 인 것은 괜찮지만(`&i32 * i32` 구현 있음), 비교/패턴에서는 `|&x|` 나 `.copied()` 가 필요한 경우가 많습니다. 컴파일러 에러가 알려 줍니다.
- `.collect()` 의 대상 타입을 못 정하면 `E0283 type annotations needed` 가 납니다. `let v: Vec<_> = ...` 또는 `collect::<Vec<_>>()`.

## 🧯 자주 만나는 컴파일 에러

**1. `E0502` — 순회 중 수정**
```
error[E0502]: cannot borrow `scores` as mutable because it is also borrowed as immutable
   |     for s in scores.iter() {
   |              -------------
   |              |
   |              immutable borrow occurs here
   |         scores.push(*s);
   |         ^^^^^^^^^^^^^^^ mutable borrow occurs here
```
→ 의미: 순회하는 동안 컨테이너는 빌려진 상태입니다. 고치는 법: 새 Vec 에 모았다가 나중에 합치기, 인덱스 루프, `retain`/`extend` 등 전용 메서드.

**2. `E0282` / `E0283` — `collect` 의 타입을 모름**
```
error[E0283]: type annotations needed
  |     let v = it.collect();
  |         ^       ------- type must be known at this point
  = note: cannot satisfy `_: FromIterator<u32>`
```
→ `let v: Vec<u32> = ...collect();` 처럼 타입을 적어 주세요.

**3. `E0373` — 클로저가 지역 변수를 빌린 채 반환**
```
error[E0373]: closure may outlive the current function, but it borrows `bonus`, which is owned by the current function
help: to force the closure to take ownership of `bonus` (and any other referenced variables), use the `move` keyword
```
→ `move |s| ...` 를 쓰세요.

**4. `E0382` — `FnOnce` 클로저를 두 번 호출 / 이동된 값을 재사용**
```
error[E0382]: use of moved value: `consume`
  note: closure cannot be invoked more than once because it moves the variable `names` out of its environment
```
→ 한 번만 부르거나, 소비하지 않도록 `clone()`/참조를 쓰세요.

**5. `E0277` — `f64` 를 `sort()`/`max()` 하려 함**
```
error[E0277]: the trait bound `f64: Ord` is not satisfied
```
→ `sort_by(|a, b| a.total_cmp(b))`, `max_by(|a, b| a.total_cmp(b))`, 또는 `partial_cmp(..).unwrap()`.

**6. `E0599` — 이터레이터가 아닌 것에 어댑터 호출**
```
error[E0599]: `Vec<u32>` is not an iterator
  = note: the following trait bounds were not satisfied: ...
```
→ `v.iter().map(..)` 처럼 먼저 `.iter()`/`.into_iter()` 를 호출하세요.

## 🛠️ 실습: 성적 데이터 분석

### 무엇을 만드나요
`data/scores.csv` (student, subject, score)를 읽어
1. 학생별 평균, 2. 과목별 최고 득점자, 3. 등급 히스토그램 (원점수와 +5 보정 후), 4. 합격 개수와 클로저 호출 수, 5. 평균 순위표
를 출력합니다. CSV 에는 일부러 **망가진 줄**(`frank,english,abc`)과 **공백이 낀 줄**, 빈 줄이 들어 있어서 파싱 단계에서 걸러 냅니다.

```
records: 16 (skipped 1)
subjects: english, math, science
== average per student ==
alice : 90.00
bob   : 64.33
...
== top per subject ==
english : eve (94)
math    : carol (95)
science : alice (93)
== grade histogram ==
A: ###### (6)
...
== ranking ==
1. carol  92.00
...
```

### 파일 구성
| 경로 | 설명 |
|---|---|
| `data/scores.csv` | 입력 |
| `python/main.py` | `defaultdict` / `Counter` / 컴프리헨션 버전 |
| `cpp/main.cpp` | `std::map` / `views::transform` / 람다 버전 |
| `rust/src/main.rs` | **실습 파일** (TODO 10개) |
| `rust/examples/solution.rs` | 정답 |

### 실행 방법 (챕터 디렉터리 `chapters/ch07_collections_iterators` 에서)
```bash
python3 python/main.py
g++ -std=c++20 -O2 -Wall -Wextra cpp/main.cpp -o /tmp/x && /tmp/x
cargo run -q -p ch07_collections_iterators
cargo test -p ch07_collections_iterators
cargo run -q -p ch07_collections_iterators --example solution
../../scripts/compare.sh ch07
```

### TODO 목록
| 번호 | 내용 | 관련 개념 |
|---|---|---|
| 1 | `parse_record` | `split`+`map(trim)` 이터레이터, `?` on Option, `parse().ok()` |
| 2 | `parse_records` | `skip`, `filter`, `filter_map`, `collect` |
| 3 | `student_averages` | `BTreeMap`, `entry().or_default()`, `sum`, `into_iter().map().collect()` |
| 4 | `subject_tops` | `entry().and_modify().or_insert_with()`, 클로저의 `&mut` 인자 |
| 5 | `grade_of`, `grade_histogram` | 범위 패턴, `HashMap` + `entry` 로 개수 세기 |
| 6 | `make_curve` | 클로저 반환 (`impl Fn`), `move` |
| 7 | `apply_in_place` | `iter_mut`, `*s = ...` |
| 8 | `count_passing` | `FnMut` (상태를 바꾸는 클로저), `filter`+`count`, `\|&&s\|` |
| 9 | `ranking` | `sort_by`, `total_cmp`, `then_with` |
| 10 | `distinct_subjects` | `HashSet`, 정렬 후 출력 |

### 완료 기준
- `cargo test -p ch07_collections_iterators` 가 모두 통과
- `../../scripts/compare.sh ch07` 에서 세 언어의 출력이 동일

### 🔬 실험 목록
1. `main` 끝의 "순회 중 수정" 주석을 풀어 `E0502` 를 확인하세요. 같은 코드를 C++ (`for (auto& s : v) v.push_back(s);`) 로 쓰면 어떻게 되는지도 생각해 보세요.
2. `FnOnce` 실험의 주석을 풀어 `E0382` (`consume` 두 번 호출) 를 확인하세요.
3. TODO(6) 의 `move` 를 지워 `E0373` 을 확인하세요.
4. (선택) TODO(2) 의 체인에 `.inspect(|l| println!("visit {l}"))` 를 끼우고 `collect()` 를 지우면 아무것도 출력되지 않음을 확인해 **게으름**을 체험해 보세요 (경고 `unused ... must be used`).

## 🤖 바이브 코딩 포인트
- AI 가 `for i in 0..v.len() { v[i] ... }` 로 쓴 곳은 `iter()` / `iter().enumerate()` / `iter_mut()` 로 바꿀 수 있는지 보세요. 인덱스 루프는 경계 검사 비용과 off-by-one 위험이 있습니다. ("인덱스 루프 대신 이터레이터를 써 줘")
- `.clone()` 이 체인 곳곳에 있으면 소유권/빌림이 정리되지 않은 신호입니다. `.iter()` + `.cloned()`/`.copied()` 가 정말 필요한 지점인지 확인하세요.
- `HashMap` 을 그대로 `println!`/순회해서 출력하는 코드는 **실행마다 순서가 달라집니다.** 테스트가 flaky 해지는 흔한 원인이니 `BTreeMap` 이나 정렬을 요청하세요.
- `collect::<Vec<_>>()` 직후 바로 `.iter()` 로 또 돌리는 코드는 불필요한 중간 할당입니다 ("중간 Vec 없이 체인으로 이어 줘").
- 클로저가 `move` 인지, 인자가 `impl Fn` / `impl FnMut` / `FnOnce` 인지 시그니처를 읽어 호출 의미(여러 번? 한 번? 상태 변경?)를 파악하세요. 스레드로 보내는 클로저(Ch13)에는 `move` 가 거의 항상 필요합니다.
- `unwrap()` 이 `parse()`, `get()`, `max()` 에 붙어 있다면 데이터가 비었을 때 panic 합니다. `filter_map`, `unwrap_or`, `?` 로 바꿀 수 있는지 물어보세요.

## ✅ 셀프 체크
1. `iter()`, `iter_mut()`, `into_iter()` 는 각각 어떤 타입의 원소를 주고, 호출 후 원본은 어떻게 되나요?
<details><summary>답</summary>`&T` (원본 유지, 읽기), `&mut T` (원본 유지, 제자리 수정), `T` (소유권 이동 → 원본 사용 불가).</details>

2. `HashMap` 으로 단어 개수를 세는 한 줄을 써 보세요. 이때 왜 출력 전에 정렬이 필요한가요?
<details><summary>답</summary>`*counts.entry(word).or_insert(0) += 1;` — HashMap 은 순회 순서가 보장되지 않아 결과가 실행마다 달라질 수 있으므로 키를 정렬하거나 `BTreeMap` 을 씁니다.</details>

3. `let f = |x| x + n;` 이 `Fn`, `FnMut`, `FnOnce` 중 무엇이고 `n` 을 `move` 로 캡처하면 무엇이 달라지나요?
<details><summary>답</summary>`n` 을 읽기만 하므로 `Fn` 입니다. `move` 를 붙이면 `n` 을 빌리지 않고 복사/이동해 가져가므로 클로저가 `n` 의 원래 스코프보다 오래 살 수 있습니다 (trait 종류 자체는 클로저가 *하는 일*이 결정).</details>

4. `v.iter().map(|x| x * 2)` 만 쓰고 끝내면 무슨 일이 일어나나요?
<details><summary>답</summary>아무 일도 일어나지 않습니다(게으름). `collect`/`sum`/`for` 같은 소비자가 있어야 실행되고, 컴파일러가 `unused ... must be used` 경고를 줍니다.</details>

5. 순회 중에 같은 `Vec` 에 `push` 하면 C++ 과 Rust 에서 각각 어떻게 되나요?
<details><summary>답</summary>C++ 은 반복자 무효화로 미정의 동작(보통 크래시/오동작)이고, Rust 는 순회가 `Vec` 을 빌리고 있어 `E0502` 컴파일 에러입니다.</details>

## 📚 더 알아보기
- The Rust Book: [Ch08 Common Collections](https://doc.rust-lang.org/book/ch08-00-common-collections.html), [Ch13 Closures & Iterators](https://doc.rust-lang.org/book/ch13-00-functional-features.html)
- Rust by Example: [Closures](https://doc.rust-lang.org/rust-by-example/fn/closures.html), [Iterators](https://doc.rust-lang.org/rust-by-example/trait/iter.html), [HashMap](https://doc.rust-lang.org/rust-by-example/std/hash.html)
- std docs: [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html), [`HashMap::entry`](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry), [`std::collections`](https://doc.rust-lang.org/std/collections/index.html)
