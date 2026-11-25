//! Ch02. 콜라츠 통계 - 정답
//!
//! 실행:  cargo run -q -p ch02_syntax_basics --example solution
//! 테스트: cargo test -p ch02_syntax_basics --example solution

// ───────── 콜라츠 기본 ─────────

/// `if` 는 식이라서 두 갈래의 값이 그대로 함수의 반환값이 됩니다. (삼항 연산자가 필요 없어요)
fn collatz_next(n: u64) -> u64 {
    if n.is_multiple_of(2) { n / 2 } else { 3 * n + 1 }
}

/// 1 에 도달할 때까지의 단계 수.  `mut` 로 선언한 변수만 바꿀 수 있습니다.
fn collatz_steps(mut n: u64) -> u32 {
    let mut steps = 0;
    while n != 1 {
        n = collatz_next(n);
        steps += 1;
    }
    steps
}

/// 사슬 도중 만나는 최댓값 (시작값 포함).
fn collatz_peak(mut n: u64) -> u64 {
    let mut peak = n;
    while n != 1 {
        n = collatz_next(n);
        peak = peak.max(n);
    }
    peak
}

/// `match` 는 모든 경우를 다뤄야 합니다(exhaustive). `_` 는 "나머지 전부".
fn classify_steps(steps: u32) -> &'static str {
    match steps {
        0 => "start",
        1..=10 => "short",
        11..=50 => "medium",
        _ => "long",
    }
}

/// (n, steps) 튜플 반환. 같은 길이면 먼저 나온 n 을 유지하려고 `>` (엄격히 큰 경우만 갱신) 를 씁니다.
fn longest_chain(limit: u64) -> (u64, u32) {
    let mut best = (1, 0);
    for n in 1..=limit {
        let s = collatz_steps(n);
        if s > best.1 {
            best = (n, s);
        }
    }
    best
}

fn max_peak(limit: u64) -> (u64, u64) {
    let mut best = (1, 1);
    for n in 1..=limit {
        let p = collatz_peak(n);
        if p > best.1 {
            best = (n, p);
        }
    }
    best
}

// ───────── 오버플로 ─────────

/// `checked_mul` 은 넘치면 None. `?` 는 "None 이면 여기서 바로 None 을 반환" (6장에서 자세히).
fn checked_next_u8(n: u8) -> Option<u8> {
    if n.is_multiple_of(2) {
        Some(n / 2)
    } else {
        n.checked_mul(3)?.checked_add(1)
    }
}

fn checked_next_u32(n: u32) -> Option<u32> {
    if n.is_multiple_of(2) {
        Some(n / 2)
    } else {
        n.checked_mul(3)?.checked_add(1)
    }
}

/// start 에서 1 까지 가는 동안 처음 넘치는 순간까지 "성공한" 단계 수. 안 넘치면 None.
fn overflow_step_u8(start: u8) -> Option<u32> {
    let mut n = start;
    let mut steps = 0;
    while n != 1 {
        match checked_next_u8(n) {
            Some(next) => n = next,
            None => return Some(steps),
        }
        steps += 1;
    }
    None
}

fn overflow_step_u32(start: u32) -> Option<u32> {
    let mut n = start;
    let mut steps = 0;
    while n != 1 {
        match checked_next_u32(n) {
            Some(next) => n = next,
            None => return Some(steps),
        }
        steps += 1;
    }
    None
}

/// 라벨(`'outer`)을 붙이면 안쪽 루프에서 바깥 루프까지 한 번에 빠져나올 수 있어요.
fn find_first_exceeding(limit: u64, ceiling: u64) -> Option<(u64, u64)> {
    let mut found = None;
    'outer: for n in 1..=limit {
        let mut x = n;
        loop {
            if x > ceiling {
                found = Some((n, x));
                break 'outer;
            }
            if x == 1 {
                break;
            }
            x = collatz_next(x);
        }
    }
    found
}

// ───────── as 변환 ─────────

/// `as u8` : 하위 8비트만 남깁니다 (300 -> 44, -1 -> 255).
fn to_u8(x: i32) -> u8 {
    x as u8
}

/// `as i32` : 소수부를 0 쪽으로 버립니다. 범위를 넘으면 최솟값/최댓값으로 "포화"해요 (C++ 은 UB).
fn trunc_to_i32(x: f64) -> i32 {
    x as i32
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
