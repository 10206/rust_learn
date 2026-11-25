//! # Ch01. Cargo와 첫 프로그램 - 온도 변환표
//!
//! ## 목표
//! - `cargo run` / `cargo test` / `cargo check` 로 프로그램을 돌려 본다.
//! - 함수 정의, 마지막 식이 반환값이 되는 규칙, `println!` / `format!` 사용법을 익힌다.
//!
//! ## 대응 파일
//! - Python: `../python/main.py`
//! - C++   : `../cpp/main.cpp`
//!
//! ## 진행 방법
//! 1. 아래 `TODO(1)` ~ `TODO(5)` 를 번호 순서대로 채웁니다. (`todo!()` 를 지우고 직접 구현)
//! 2. 하나 끝낼 때마다 `cargo test -p ch01_cargo_basics` 를 실행해 초록불이 늘어나는지 확인합니다.
//! 3. 막히면 먼저 Python / C++ 파일의 같은 이름 함수를 보고, 그래도 안 되면
//!    `examples/solution.rs` 를 열어 보세요. ("먼저 시도 -> 그다음 정답 엿보기")
//!
//! ## 완료 기준
//! - `cargo test -p ch01_cargo_basics` 가 모두 통과
//! - `../../scripts/compare.sh ch01` 에서 Python / C++ / Rust 출력이 똑같음
//!   (저장소 루트에서 실행한다면 `scripts/compare.sh ch01`)

// 다 풀고 나면 이 줄을 지워 보세요. (경고가 어떤 걸 알려주는지 읽어 보는 것도 공부예요)
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

// ─────────────────────────────────────────────
// TODO(1): 이름을 받아 "Hello, <name>!" 문자열을 만들어 돌려주세요. (화면 출력 X, 문자열 반환 O)
//   🐍 Python: f"Hello, {name}!"                       (python/main.py: greeting)
//   ⚙️ C++  : std::format("Hello, {}!", name)           (cpp/main.cpp: greeting)
//   🦀 Rust : `format!` 은 `println!` 과 똑같이 쓰지만 화면에 찍지 않고 `String` 을 돌려줍니다.
//   💡 힌트: format!("... {name} ...")  처럼 변수 이름을 중괄호 안에 바로 써도 됩니다.
//   ❓ 왜?: `println!`, `format!` 처럼 이름 끝에 `!` 가 붙은 것은 함수가 아니라 "매크로"예요.
//           컴파일할 때 코드로 펼쳐지고, 형식 문자열이 틀리면 컴파일 에러로 알려 줍니다.
// ─────────────────────────────────────────────
fn greeting(name: &str) -> String {
    todo!("TODO(1): format! 으로 인사말 문자열 만들기")
}

// ─────────────────────────────────────────────
// TODO(2): 섭씨 -> 화씨.  F = C * 9 / 5 + 32
//   🐍 Python: return c * 9.0 / 5.0 + 32.0              (python/main.py: c_to_f)
//   ⚙️ C++  : return c * 9.0 / 5.0 + 32.0;               (cpp/main.cpp: c_to_f)
//   🦀 Rust : 함수의 "마지막 식"이 반환값입니다. 끝에 `;` 를 붙이면 반환값이 사라져요!
//   💡 힌트: 숫자 리터럴도 f64 라면 `9.0` 처럼 소수점을 찍어야 합니다 (`9` 는 정수).
//   ❓ 왜?: Rust 는 정수와 실수를 자동으로 섞지 않습니다. 의도하지 않은 변환 버그를 막기 위해서예요.
//   🔬 실험(1): 아래 "experiments" 함수의 [실험 1] 을 열어 `;` 를 붙이면 어떤 에러가 나는지 보세요.
// ─────────────────────────────────────────────
fn c_to_f(c: f64) -> f64 {
    todo!("TODO(2): 섭씨를 화씨로")
}

// ─────────────────────────────────────────────
// TODO(3): 화씨 -> 섭씨.  C = (F - 32) * 5 / 9
//   🐍 Python: return (f - 32.0) * 5.0 / 9.0            (python/main.py: f_to_c)
//   ⚙️ C++  : return (f - 32.0) * 5.0 / 9.0;             (cpp/main.cpp: f_to_c)
//   🦀 Rust : TODO(2) 와 같은 규칙입니다. 연산 순서를 Python/C++ 과 똑같이 써야 출력이 1글자도 안 다릅니다.
//   💡 힌트: (f - 32.0) * 5.0 / 9.0
// ─────────────────────────────────────────────
fn f_to_c(f: f64) -> f64 {
    todo!("TODO(3): 화씨를 섭씨로")
}

// ─────────────────────────────────────────────
// TODO(4): 표의 한 줄을 만들어 돌려주세요. 섭씨 값과 화씨 값을 각각 폭 7, 소수점 1자리, 오른쪽 정렬하고
//          사이에 공백 한 칸.  예) table_line(-10.0) == "  -10.0    14.0"
//   🐍 Python: f"{c:>7.1f} {c_to_f(c):>7.1f}"            (python/main.py: table_line)
//   ⚙️ C++  : std::format("{:>7.1f} {:>7.1f}", ...)      (cpp/main.cpp: table_line)
//   🦀 Rust : format!("{:>7.1} {:>7.1}", a, b)  ->  `>` 오른쪽 정렬, `7` 폭, `.1` 소수 1자리
//   💡 힌트: 방금 만든 c_to_f 를 호출해서 재사용하세요.
//   ❓ 왜?: Rust 의 {:.1} 에는 f 가 필요 없어요. 타입(f64)을 컴파일러가 이미 알기 때문입니다.
// ─────────────────────────────────────────────
fn table_line(c: f64) -> String {
    todo!("TODO(4): 표 한 줄 문자열 만들기")
}

// ─────────────────────────────────────────────
// TODO(5): 표 전체를 출력하세요.
//          첫 줄은 헤더 "{:>7} {:>7}" 로 "C" 와 "F" 를 찍고,
//          -10 부터 40 까지(40 포함) 10 씩 늘려 가며 table_line 의 결과를 한 줄씩 찍습니다.
//   🐍 Python: for c in range(-10, 41, 10): print(table_line(float(c)))   (python/main.py: print_table)
//   ⚙️ C++  : for (int c = -10; c <= 40; c += 10) ...                       (cpp/main.cpp: print_table)
//   🦀 Rust : for c in (-10..=40).step_by(10) { ... }
//   💡 힌트: `a..=b` 는 b 를 "포함"하는 범위, `a..b` 는 b 를 제외하는 범위입니다.
//           c 는 정수(i32)로 추론되므로 f64 로 바꿔야 해요 -> f64::from(c)  (또는 c as f64)
//   ❓ 왜?: `f64::from(c)` 는 "손실 없는" 변환만 허용하고, `as` 는 줄어드는 변환도 해 버려요. (2장에서 자세히)
// ─────────────────────────────────────────────
fn print_table() {
    todo!("TODO(5): 헤더와 표 줄 출력하기")
}

// ─────────────────────────────────────────────
// TODO(6): 값 목록에서 (최솟값, 최댓값, 평균) 을 튜플로 돌려주세요.
//   🐍 Python: return min(v), max(v), sum(v) / len(v)    (python/main.py: min_max_avg)
//   ⚙️ C++  : std::minmax_element + std::accumulate        (cpp/main.cpp: min_max_avg)
//   🦀 Rust : f64 는 NaN 이 있어서 `.iter().min()` 을 쓸 수 없어요. fold 로 직접 접어 줍니다.
//   💡 힌트: values.iter().copied().fold(f64::INFINITY, f64::min)
//           합계: values.iter().sum::<f64>()  /  개수: values.len() as f64
//   ❓ 왜?: `&[f64]` 는 "f64 배열을 빌려 읽기만 한다"는 뜻의 타입입니다 (4장). 지금은 그냥 쓰면 돼요.
// ─────────────────────────────────────────────
fn min_max_avg(values: &[f64]) -> (f64, f64, f64) {
    todo!("TODO(6): 최솟값, 최댓값, 평균")
}

fn main() {
    println!("{}", greeting("Cargo"));
    println!("=== Celsius to Fahrenheit ===");
    print_table();

    println!("=== round trip (F -> C) ===");
    for f in [14.0, 32.0, 98.6, 212.0] {
        println!("{:>6.1} F = {:>6.1} C", f, f_to_c(f));
    }

    println!("=== sample readings ===");
    let readings = [22.5, 18.0, 30.1, 15.7, 25.3];
    let joined: Vec<String> = readings.iter().map(|r| format!("{r:.1}")).collect();
    println!("readings: {}", joined.join(", "));
    let (lo, hi, avg) = min_max_avg(&readings);
    println!("min: {lo:.1}  max: {hi:.1}  avg: {avg:.2}");
}

// ═════════════════════════════════════════════
// 🔬 실험실: 컴파일러와 친해지기
//   아래 코드는 일부러 주석 처리되어 있어요. 한 블록씩 주석을 풀고 `cargo check -p ch01_cargo_basics`
//   를 실행해 에러 메시지를 읽어 보세요. 다 보고 나면 다시 주석 처리합니다.
//   (이 함수는 어디서도 호출되지 않으니 주석만 풀어도 프로그램 동작에는 영향이 없어요.)
// ═════════════════════════════════════════════
fn experiments() {
    // [실험 1] 마지막 식 뒤에 `;` 를 붙이면?  기대 에러: E0308 (expected `f64`, found `()`)
    //   ❓ 질문: 컴파일러가 "remove this semicolon" 이라고 정확히 짚어 주나요?
    //
    // fn broken_c_to_f(c: f64) -> f64 {
    //     c * 9.0 / 5.0 + 32.0;
    // }

    // [실험 2] 정수 리터럴을 f64 자리에 넣으면?  기대 에러: E0308 (expected `f64`, found integer)
    //   ❓ 질문: C++ 에서는 c_to_f(100) 이 조용히 통과합니다. Rust 가 막는 이유는 무엇일까요?
    //
    // let boiling = c_to_f(100);

    // [실험 3] 매크로 `!` 를 빼먹으면?  기대 에러: E0423 (expected function, found macro `println`)
    //
    // println("hello");

    // [실험 4] 변수 이름 오타.  기대 에러: E0425 (cannot find value `tmep` in this scope)
    //   ❓ 질문: Python 이라면 이 오류를 언제 알게 될까요? (실행 중 NameError)
    //
    // let temp = 21.5;
    // println!("{}", tmep);

    // [실험 5] Vec 을 `{}` 로 찍으면?  기대 에러: E0277 (`Vec<{integer}>` doesn't implement `Display`)
    //   ❓ 질문: 에러 메시지가 대안(`{:?}`)을 알려 주나요?
    //
    // let v = vec![1, 2, 3];
    // println!("{}", v);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn greeting_works() {
        assert_eq!(greeting("Cargo"), "Hello, Cargo!");
    }

    #[test]
    fn celsius_to_fahrenheit() {
        assert!(close(c_to_f(0.0), 32.0));
        assert!(close(c_to_f(100.0), 212.0));
        assert!(close(c_to_f(-40.0), -40.0));
    }

    #[test]
    fn fahrenheit_to_celsius() {
        assert!(close(f_to_c(32.0), 0.0));
        assert!(close(f_to_c(212.0), 100.0));
        assert!(close(f_to_c(-40.0), -40.0));
    }

    #[test]
    fn round_trip() {
        for c in [-10.0, 0.0, 21.5, 40.0] {
            assert!(close(f_to_c(c_to_f(c)), c));
        }
    }

    #[test]
    fn table_line_is_aligned() {
        assert_eq!(table_line(-10.0), "  -10.0    14.0");
        assert_eq!(table_line(40.0), "   40.0   104.0");
    }

    #[test]
    fn stats() {
        let (lo, hi, avg) = min_max_avg(&[3.0, 1.0, 2.0]);
        assert!(close(lo, 1.0));
        assert!(close(hi, 3.0));
        assert!(close(avg, 2.0));
    }
}
