//! # Ch02. 변수·타입·표현식·제어 흐름 - 콜라츠 통계
//!
//! ## 목표
//! - `let` / `let mut` / shadowing, 정수·실수 타입, `as` 변환을 직접 써 본다.
//! - `if` / `match` / `loop` 가 "값을 돌려주는 식"이라는 것, 세미콜론의 의미를 몸으로 익힌다.
//! - 정수 오버플로(debug 패닉)와 `checked_` / `wrapping_` / `saturating_` 를 구분한다.
//!
//! ## 대응 파일
//! - Python: `../python/main.py`
//! - C++   : `../cpp/main.cpp`
//!
//! ## 진행 방법
//! 1. `TODO(1)` ~ `TODO(10)` 을 순서대로 채웁니다. (`todo!()` 를 지우고 구현)
//! 2. 하나 끝낼 때마다 `cargo test -p ch02_syntax_basics` 로 확인합니다.
//! 3. 맨 아래 `experiments()` 의 🔬 실험을 하나씩 열어 컴파일러 메시지를 읽어 보세요.
//! 4. 막히면 Python / C++ 의 같은 이름 함수 -> 그래도 안 되면 `examples/solution.rs`.
//!
//! ## 완료 기준
//! - `cargo test -p ch02_syntax_basics` 가 모두 통과
//! - `../../scripts/compare.sh ch02` 에서 Python / C++ / Rust 출력이 똑같음

// 다 풀고 나면 이 줄을 지워 보세요. (경고가 어떤 걸 알려주는지 읽어 보는 것도 공부예요)
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

// ───────── 콜라츠 기본 ─────────
// 콜라츠 규칙: n 이 짝수면 n/2, 홀수면 3n+1. 이걸 반복하면 (추측상) 언젠가 1 이 됩니다.

// ─────────────────────────────────────────────
// TODO(1): 규칙에 따라 다음 값을 돌려주세요.
//   🐍 Python: return n // 2 if n % 2 == 0 else 3 * n + 1     (python/main.py: collatz_next)
//   ⚙️ C++  : return n % 2 == 0 ? n / 2 : 3 * n + 1;           (cpp/main.cpp: collatz_next)
//   🦀 Rust : `if ... { a } else { b }` 자체가 값을 가지는 "식"입니다. 삼항 연산자가 없는 이유예요.
//   💡 힌트: 함수 본문 전체를 if/else 하나로 쓰고, 두 갈래 끝에 `;` 를 붙이지 마세요.
//           짝수 판정은 `n % 2 == 0` 이면 충분합니다. (clippy 는 `n.is_multiple_of(2)` 를 권해요)
//   ❓ 왜?: 식 중심 문법이면 "값이 없는 경우"가 문법적으로 사라져서, 변수를 미리 만들고 나중에 채우는 코드가 줄어요.
// ─────────────────────────────────────────────
fn collatz_next(n: u64) -> u64 {
    todo!("TODO(1): 짝수면 n/2, 홀수면 3n+1")
}

// ─────────────────────────────────────────────
// TODO(2): n 이 1 이 될 때까지 걸리는 단계 수를 돌려주세요. (1 이면 0 단계)
//   🐍 Python: while n != 1: n = collatz_next(n); steps += 1   (python/main.py: collatz_steps)
//   ⚙️ C++  : while (n != 1) { n = collatz_next(n); ++steps; }  (cpp/main.cpp: collatz_steps)
//   🦀 Rust : 매개변수도 `mut n: u64` 처럼 쓰면 함수 안에서 바꿀 수 있습니다. (호출한 쪽의 값은 안 바뀜 - 복사본)
//   💡 힌트: `let mut steps = 0;`  반환 타입이 u32 이므로 steps 는 u32 로 추론됩니다. `steps += 1` (`++` 은 없어요!)
//   ❓ 왜?: Rust 변수는 기본이 불변(immutable). "바뀌는 값"이 눈에 띄도록 `mut` 를 일부러 적게 만든 설계입니다.
//   🔬 실험 1: `mut` 를 빼고 `steps += 1` 을 하면? (experiments 함수 참고)
// ─────────────────────────────────────────────
fn collatz_steps(mut n: u64) -> u32 {
    todo!("TODO(2): 1 이 될 때까지 단계 수 세기")
}

// ─────────────────────────────────────────────
// TODO(3): 사슬 도중 나오는 최댓값(시작값 포함)을 돌려주세요.
//   🐍 Python: peak = max(peak, n)                              (python/main.py: collatz_peak)
//   ⚙️ C++  : if (n > peak) peak = n;                            (cpp/main.cpp: collatz_peak)
//   🦀 Rust : `peak = peak.max(n);`  — 정수에도 `.max()` / `.min()` 메서드가 있어요.
//   💡 힌트: 시작값 자체가 최댓값일 수 있으니 peak 의 초기값은 n.
// ─────────────────────────────────────────────
fn collatz_peak(mut n: u64) -> u64 {
    todo!("TODO(3): 사슬 중 최댓값")
}

// ─────────────────────────────────────────────
// TODO(4): 단계 수를 말로 분류하세요: 0 -> "start", 1..=10 -> "short", 11..=50 -> "medium", 그 외 -> "long".
//   🐍 Python: if/elif 사슬                                      (python/main.py: classify_steps)
//   ⚙️ C++  : if 사슬 (빠뜨려도 컴파일러가 모름)                  (cpp/main.cpp: classify_steps)
//   🦀 Rust : `match steps { 0 => "start", 1..=10 => ..., _ => ... }`  — 범위 패턴 `a..=b` 를 쓸 수 있어요.
//   💡 힌트: `_` 는 "나머지 전부". 이걸 지우면 컴파일러가 빠진 경우(`11..=u32::MAX` 등)를 알려 줍니다.
//   ❓ 왜?: match 는 "모든 경우를 다뤘는지" 컴파일러가 검사합니다(exhaustive). 나중에 enum 이 늘어나도 빠뜨릴 수 없어요.
//   🔬 실험 4: `_ => "long"` 줄을 지우면? (experiments 함수 참고)
// ─────────────────────────────────────────────
fn classify_steps(steps: u32) -> &'static str {
    todo!("TODO(4): match 로 분류하기")
}

// ─────────────────────────────────────────────
// TODO(5): 1..=limit 중 단계 수가 가장 많은 (n, steps) 튜플을 돌려주세요.
//          단계 수가 같으면 "먼저 나온 n" 을 유지합니다. (엄격히 클 때만 갱신)
//   🐍 Python: best = (n, s) if s > best[1]                       (python/main.py: longest_chain)
//   ⚙️ C++  : std::pair<uint64_t, uint32_t>                       (cpp/main.cpp: longest_chain)
//   🦀 Rust : 튜플 `(u64, u32)` 를 만들고 `best.0`, `best.1` 로 접근, `for n in 1..=limit` 로 순회.
//   💡 힌트: `let mut best = (1, 0);`  타입은 반환 타입 덕분에 자동으로 정해집니다.
// ─────────────────────────────────────────────
fn longest_chain(limit: u64) -> (u64, u32) {
    todo!("TODO(5): 가장 긴 사슬 찾기")
}

// ─────────────────────────────────────────────
// TODO(6): 1..=limit 중 사슬 최댓값(peak)이 가장 큰 (n, peak) 를 돌려주세요. 동률이면 먼저 나온 n.
//   🐍 Python / ⚙️ C++ : TODO(5) 와 같은 구조                      (python/main.py: max_peak)
//   🦀 Rust : TODO(5) 를 복사해서 고치면 됩니다. 반환 타입이 (u64, u64) 임에 주의!
// ─────────────────────────────────────────────
fn max_peak(limit: u64) -> (u64, u64) {
    todo!("TODO(6): peak 가 가장 큰 n 찾기")
}

// ───────── 오버플로 ─────────
// u8 은 0..=255, u32 는 0..=4_294_967_295. 범위를 넘는 계산을 하면?
//   - debug 빌드: panic ("attempt to multiply with overflow")
//   - release 빌드: 조용히 한 바퀴 돌아감(wrap)  <- 더 위험할 수 있어요
//   - 의도를 코드에 적는 방법: checked_xxx (None), wrapping_xxx (돌아감), saturating_xxx (끝에서 멈춤)

// ─────────────────────────────────────────────
// TODO(7): 콜라츠 다음 값을 "오버플로 없이" 계산하세요. 넘치면 None.
//          짝수면 Some(n / 2), 홀수면 3n+1 을 checked_mul(3) 과 checked_add(1) 로 계산합니다.
//          (u8 용과 u32 용 두 함수 모두 구현 - 본문은 똑같아요. 제네릭은 8장에서!)
//   🐍 Python: v = 3*n+1; return v if v <= max_value else None      (python/main.py: checked_next)
//   ⚙️ C++  : 더 넓은 타입으로 계산 후 비교 (std::optional)           (cpp/main.cpp: checked_next)
//   🦀 Rust : n.checked_mul(3)  ->  Option<u8>.  `?` 를 붙이면 "None 이면 이 함수가 즉시 None 반환".
//   💡 힌트: `n.checked_mul(3)?.checked_add(1)`  — 마지막 식이 이미 Option<u8> 이므로 Some(...) 으로 또 감싸지 마세요.
//   ❓ 왜?: 에러를 "값"으로 돌려주면 호출자가 반드시 처리하게 됩니다. 6장에서 Option / Result / `?` 를 본격적으로 다뤄요.
// ─────────────────────────────────────────────
fn checked_next_u8(n: u8) -> Option<u8> {
    todo!("TODO(7): u8 용 checked 다음 값")
}

fn checked_next_u32(n: u32) -> Option<u32> {
    todo!("TODO(7): u32 용 checked 다음 값")
}

// ─────────────────────────────────────────────
// TODO(8): start 에서 시작해 1 이 될 때까지 가다가, 처음으로 넘치는 순간까지 "성공한 단계 수" 를 Some(steps) 로,
//          끝까지 안 넘치면 None 을 돌려주세요. (u8 / u32 두 함수)
//   🐍 Python: overflow_step(start, max_value)                      (python/main.py: overflow_step)
//   ⚙️ C++  : std::optional<uint32_t> overflow_step(...)            (cpp/main.cpp: overflow_step)
//   🦀 Rust : `match checked_next_u8(n) { Some(next) => n = next, None => return Some(steps) }`
//   💡 힌트: `return` 은 함수 중간에서 빠져나올 때 씁니다. 루프가 끝난 뒤 마지막 식은 `None`.
//           start 는 1 이상이라고 가정해도 돼요 (0 은 영원히 0 이라 루프가 안 끝납니다!).
// ─────────────────────────────────────────────
fn overflow_step_u8(start: u8) -> Option<u32> {
    todo!("TODO(8): u8 사슬에서 처음 넘치는 단계")
}

fn overflow_step_u32(start: u32) -> Option<u32> {
    todo!("TODO(8): u32 사슬에서 처음 넘치는 단계")
}

// ─────────────────────────────────────────────
// TODO(9): 1..=limit 의 n 을 차례로 보면서 사슬 도중 값이 ceiling 보다 커지는 "첫" (n, 그 값) 을 Some 으로,
//          없으면 None. 찾는 즉시 바깥 for 까지 한 번에 빠져나오세요.
//   🐍 Python: 함수 안이라 그냥 return                              (python/main.py: find_first_exceeding)
//   ⚙️ C++  : goto 나 플래그 변수 필요                               (cpp/main.cpp: find_first_exceeding)
//   🦀 Rust : 루프에 라벨을 붙입니다.  `'outer: for n in 1..=limit { loop { ... break 'outer; } }`
//   💡 힌트: 결과를 담을 `let mut found = None;` 을 만들고, 찾으면 `found = Some((n, x)); break 'outer;`
//           안쪽 루프의 정상 종료(x == 1)는 라벨 없는 `break;`.
//   ❓ 왜?: 라벨 덕분에 "찾았다" 플래그 변수 없이도 중첩 루프를 깔끔하게 끝낼 수 있습니다.
// ─────────────────────────────────────────────
fn find_first_exceeding(limit: u64, ceiling: u64) -> Option<(u64, u64)> {
    todo!("TODO(9): labeled break 로 첫 번째 n 찾기")
}

// ───────── as 변환 ─────────

// ─────────────────────────────────────────────
// TODO(10): `as` 로 형 변환하세요.
//   to_u8: i32 -> u8   (300 -> 44, -1 -> 255 : 하위 8비트만 남음)
//   trunc_to_i32: f64 -> i32 (3.99 -> 3, -3.99 -> -3 : 소수부 버림)
//   🐍 Python: x & 0xFF / math.trunc(x)                              (python/main.py: to_u8, trunc_to_i32)
//   ⚙️ C++  : static_cast<uint8_t>(x) / static_cast<int32_t>(x)       (cpp/main.cpp)
//   🦀 Rust : `x as u8`.  `as` 는 "내가 책임지고 변환한다"는 선언이라, 손실이 있어도 컴파일러가 막지 않아요.
//   💡 힌트: 손실이 *없는* 변환은 `u64::from(x)`, 실패할 수 있으면 `u8::try_from(x)` (Result) 가 더 안전합니다.
//   ❓ 왜?: `as` 는 편하지만 조용히 값을 잘라 버립니다. AI 코드에서 `as` 가 보이면 "잘림이 문제 없는가?" 를 확인하세요.
//   🔬 실험 7: `let x: u32 = -1;` 은 왜 에러일까요? (experiments 함수 참고)
// ─────────────────────────────────────────────
fn to_u8(x: i32) -> u8 {
    todo!("TODO(10): i32 를 u8 로 as 변환")
}

fn trunc_to_i32(x: f64) -> i32 {
    todo!("TODO(10): f64 를 i32 로 as 변환")
}

// ───────── 출력 ─────────

fn basics_demo() {
    println!("== basics ==");
    // shadowing: 같은 이름으로 새 변수를 만듭니다. 타입도 바꿀 수 있어요.
    let x = 5;
    print!("shadowing: {x}");
    let x = x * 2;
    print!(" -> {x}");
    let x = format!("{x} apples");
    println!(" -> {x}");

    let point = (3, 4);
    let (px, py) = point; // 튜플 구조 분해
    println!("tuple: ({}, {}) -> x={px} y={py}", point.0, point.1);
    let arr = [10, 20, 30];
    println!("array: {:?} len={} sum={}", arr, arr.len(), arr.iter().sum::<i32>());

    println!(
        "div: 7/2={} -7/2={} -7%3={} rem_euclid={}",
        7 / 2,
        -7 / 2,
        -7 % 3,
        (-7_i32).rem_euclid(3)
    );
    println!(
        "casts: 300 as u8={} -1 as u8={} 3.99 as i32={} -3.99 as i32={}",
        to_u8(300),
        to_u8(-1),
        trunc_to_i32(3.99),
        trunc_to_i32(-3.99)
    );

    // if / 블록 / loop 모두 값을 가지는 "식"입니다.
    let label = if collatz_steps(27) > 100 { "big" } else { "small" };
    let block = {
        let a = 3;
        a * a // 세미콜론 없음 = 블록의 값
    };
    let mut counter = 0;
    let loop_value = loop {
        counter += 1;
        if counter == 5 {
            break counter * 2; // loop 만 break 로 값을 내보낼 수 있어요
        }
    };
    println!("exprs: if={label} block={block} loop={loop_value}");
}

fn overflow_demo() {
    println!("== overflow: u8 add ==");
    for (a, b) in [(250_u8, 5_u8), (250, 10)] {
        // checked: 넘치면 None / wrapping: 한 바퀴 돌아감 / saturating: 최댓값에서 멈춤
        println!(
            "{a} + {b}: checked={:?} wrapping={} saturating={}",
            a.checked_add(b),
            a.wrapping_add(b),
            a.saturating_add(b)
        );
    }

    println!("== overflow: collatz ==");
    // 1..=255 를 u8 로 돌려서 넘치는 시작값을 모읍니다. (start, step) 튜플의 Vec.
    let overflowing: Vec<(u8, u32)> =
        (1..=255_u8).filter_map(|s| overflow_step_u8(s).map(|st| (s, st))).collect();
    let (first_s, first_st) = overflowing[0];
    println!(
        "u8 starts 1..=255 that overflow: {} (first: {first_s} at step {first_st})",
        overflowing.len()
    );
    for start in [27_u32, 113_383, 159_487] {
        match overflow_step_u32(start) {
            None => println!("u32 start={start}: no overflow"),
            Some(step) => println!("u32 start={start}: overflow at step {step}"),
        }
    }
}

fn main() {
    basics_demo();

    println!("== collatz 1..=30 ==");
    println!("{:>3} {:>5} {:>6} kind", "n", "steps", "peak");
    for n in 1..=30 {
        let s = collatz_steps(n);
        println!("{:>3} {:>5} {:>6} {}", n, s, collatz_peak(n), classify_steps(s));
    }
    let (n, s) = longest_chain(30);
    println!("longest chain: n={n} steps={s}");
    let (n, p) = max_peak(30);
    println!("highest peak: n={n} peak={p}");

    overflow_demo();

    println!("== labeled break ==");
    for (limit, ceiling) in [(100, 1000), (30, 10000)] {
        match find_first_exceeding(limit, ceiling) {
            Some((n, value)) => {
                println!("first n<={limit} whose chain exceeds {ceiling}: n={n} value={value}")
            }
            None => println!("first n<={limit} whose chain exceeds {ceiling}: none"),
        }
    }
}


// ═════════════════════════════════════════════
// 🔬 실험실: 컴파일러가 막아 주는 것들
//   한 블록씩 주석을 풀고 `cargo check -p ch02_syntax_basics` 로 메시지를 읽어 보세요. 다 본 뒤엔 다시 주석 처리!
//   (이 함수는 어디서도 호출되지 않아서, 주석을 풀어도 프로그램 동작은 그대로입니다.)
// ═════════════════════════════════════════════
fn experiments() {
    // [실험 1] 불변 변수에 다시 대입.  기대 에러: E0384 (cannot assign twice to immutable variable)
    //   ❓ 질문: 컴파일러가 "let mut x" 로 고치라고 제안하나요? C++ 의 const 와 기본값이 어떻게 반대인가요?
    //
    // let x = 5;
    // x = 6;

    // [실험 2] 서로 다른 정수 타입끼리 연산.  기대 에러: E0308 + E0277 (cannot add `i64` to `i32`)
    //   ❓ 질문: C++ 은 int + long 을 자동 승격합니다. Rust 가 막는 이유는? 고치려면? (a as i64 + b)
    //
    // let a: i32 = 5;
    // let b: i64 = 10;
    // println!("{}", a + b);

    // [실험 3] if / else 의 두 갈래 타입이 다르면.  기대 에러: E0308 (`if` and `else` have incompatible types)
    //
    // let label = if 3 > 2 { "big" } else { 0 };

    // [실험 4] match 에서 경우를 빠뜨리면.  기대 에러: E0004 (non-exhaustive patterns)
    //   ❓ 질문: 에러 메시지가 "빠진 범위"를 정확히 알려 주나요?
    //
    // let n = 3;
    // let s = match n { 1 => "one", 2 => "two" };

    // [실험 5] 컴파일 타임에 알 수 있는 오버플로.  기대 에러: "this arithmetic operation will overflow" (lint, deny)
    //   (`cargo check` 는 이 검사를 건너뛰므로 `cargo build -p ch02_syntax_basics` 로 확인하세요)
    //
    // let x: u8 = 255;
    // let y = x + 1;

    // [실험 6] 런타임 오버플로.  이건 컴파일은 되고 "실행할 때" 패닉입니다.
    //   1) 주석을 풀고 `cargo run -p ch02_syntax_basics` (debug)   -> panicked ... attempt to add with overflow
    //   2) `cargo run --release -p ch02_syntax_basics`              -> 패닉 없이 y = 0 (한 바퀴 돌아감)
    //   ❓ 질문: 왜 release 쪽이 더 위험할 수 있을까요? 의도적인 감김이 필요하면 어떤 메서드를 쓰나요? (wrapping_add)
    //   (args().count() 는 컴파일러가 값을 미리 알아채지 못하게 하려는 장치일 뿐이에요.)
    //
    // let x: u8 = std::env::args().count() as u8 + 254;
    // let y = x + 1;
    // println!("y = {y}");

    // [실험 7] 부호 없는 타입에 음수.  기대 에러: E0600 (cannot apply unary operator `-` to type `u32`)
    //
    // let x: u32 = -1;

    // [실험 8] 범위를 벗어난 리터럴.  기대 에러: "literal out of range for `u8`" (lint, deny)
    //
    // let x: u8 = 300;

    // [실험 9] while 에서 값을 들고 break.  기대 에러: E0571 (`break` with value from a `while` loop)
    //   ❓ 질문: `loop` 는 되고 `while` 은 안 되는 이유는? (while 은 조건이 거짓이라 끝날 때 값이 없을 수 있어요)
    //
    // let mut n = 0;
    // let v = while n < 5 { n += 1; break n; };

    // [실험 10] 타입을 추론할 수 없을 때.  기대 에러: E0284 (type annotations needed)
    //   고치기: let guess: u32 = ...  또는  "42".parse::<u32>()  (turbofish)
    //
    // let guess = "42".parse().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_follows_rule() {
        assert_eq!(collatz_next(6), 3);
        assert_eq!(collatz_next(7), 22);
        assert_eq!(collatz_next(1), 4);
    }

    #[test]
    fn steps_and_peak() {
        assert_eq!(collatz_steps(1), 0);
        assert_eq!(collatz_steps(6), 8);
        assert_eq!(collatz_steps(27), 111);
        assert_eq!(collatz_peak(1), 1);
        assert_eq!(collatz_peak(7), 52);
        assert_eq!(collatz_peak(27), 9232);
    }

    #[test]
    fn classify_covers_ranges() {
        assert_eq!(classify_steps(0), "start");
        assert_eq!(classify_steps(10), "short");
        assert_eq!(classify_steps(11), "medium");
        assert_eq!(classify_steps(50), "medium");
        assert_eq!(classify_steps(51), "long");
    }

    #[test]
    fn longest_and_peak() {
        assert_eq!(longest_chain(30), (27, 111));
        assert_eq!(max_peak(30), (27, 9232));
        // 동률이면 먼저 나온 값: 1..=2 중 steps 최댓값은 n=2 (1 단계)
        assert_eq!(longest_chain(2), (2, 1));
    }

    #[test]
    fn checked_next_detects_overflow() {
        assert_eq!(checked_next_u8(10), Some(5));
        assert_eq!(checked_next_u8(85), None); // 3*85 = 255, +1 = 256 > 255
        assert_eq!(checked_next_u8(84), Some(42));
        assert_eq!(checked_next_u32(7), Some(22));
        assert_eq!(checked_next_u32(u32::MAX), None);
    }

    #[test]
    fn overflow_steps() {
        assert_eq!(overflow_step_u8(27), Some(11));
        assert_eq!(overflow_step_u8(1), None);
        assert_eq!(overflow_step_u8(2), None);
        assert_eq!(overflow_step_u32(27), None);
        assert_eq!(overflow_step_u32(159_487), Some(59));
    }

    #[test]
    fn labeled_break_search() {
        assert_eq!(find_first_exceeding(100, 1000), Some((27, 1186)));
        assert_eq!(find_first_exceeding(30, 10_000), None);
    }

    #[test]
    fn casts() {
        assert_eq!(to_u8(300), 44);
        assert_eq!(to_u8(-1), 255);
        assert_eq!(trunc_to_i32(3.99), 3);
        assert_eq!(trunc_to_i32(-3.99), -3);
        assert_eq!(trunc_to_i32(1e20), i32::MAX); // 포화
    }
}
