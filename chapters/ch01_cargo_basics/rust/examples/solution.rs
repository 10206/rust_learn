//! Ch01. 온도 변환표 - 정답
//!
//! 실행:  cargo run -q -p ch01_cargo_basics --example solution
//! 테스트: cargo test -p ch01_cargo_basics --example solution
//!
//! 정답 파일은 "이렇게도 풀 수 있다"의 한 예일 뿐이에요. 출력만 같다면 다르게 풀어도 괜찮습니다.

/// `format!` 은 화면에 찍지 않고 `String` 을 만들어 돌려줍니다 (C++ std::format, Python f-string).
fn greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

// 함수의 마지막 식(끝에 `;` 없음)이 곧 반환값입니다. `return` 은 필요 없어요.
fn c_to_f(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn f_to_c(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

/// `{:>7.1}` = 폭 7, 오른쪽 정렬, 소수점 아래 1자리
fn table_line(c: f64) -> String {
    format!("{:>7.1} {:>7.1}", c, c_to_f(c))
}

fn print_table() {
    println!("{:>7} {:>7}", "C", "F");
    // (-10..=40) 은 -10 부터 40 까지 "40 포함" 범위. step_by(10) 으로 10씩 건너뜁니다.
    for c in (-10..=40).step_by(10) {
        // c 는 i32 로 추론됩니다. 정수 -> 실수는 손실이 없으므로 f64::from 을 쓸 수 있어요.
        println!("{}", table_line(f64::from(c)));
    }
}

/// `&[f64]` : f64 배열/벡터를 "빌려서" 읽기만 하는 타입 (4장에서 자세히).
fn min_max_avg(values: &[f64]) -> (f64, f64, f64) {
    // f64 는 NaN 때문에 Ord 가 아니라서 iter().min() 을 못 써요. fold + f64::min 을 씁니다.
    let lo = values.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let sum: f64 = values.iter().sum();
    (lo, hi, sum / values.len() as f64)
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
