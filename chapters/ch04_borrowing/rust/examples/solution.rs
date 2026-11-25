//! Ch04. 텍스트 분석기 - 정답
//!
//! 실행:  cargo run -q -p ch04_borrowing --example solution
//! 테스트: cargo test -p ch04_borrowing --example solution

const TEXT: &str =
    "The quick brown fox jumps over the lazy dog. It was a sunny day! Did you enjoy the afternoon?";
const KOREAN: &str = "안녕하세요 러스트";

/// `&str` 를 받으면 `&String`, 문자열 리터럴, 슬라이스를 모두 넘길 수 있어요. (읽기만 하는 함수의 정석)
fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

/// 반환 타입 `&str` 의 수명은 인자 `text` 의 수명과 자동으로 같아집니다 (수명 생략 규칙, 9장).
/// 새 String 을 만들지 않고 원본의 일부를 "빌려서" 돌려주므로 복사가 없어요.
fn longest_word(text: &str) -> &str {
    let mut best = "";
    for w in text.split_whitespace() {
        let w = w.trim_matches(|c: char| c.is_ascii_punctuation());
        // `max_by_key` 는 동률이면 "마지막" 요소를 돌려줘서 Python/C++ 과 결과가 달라져요. 그래서 직접 비교합니다.
        if w.len() > best.len() {
            best = w;
        }
    }
    best
}

/// 첫 `.` `!` `?` 까지(그 문자 포함). 셋 다 1바이트 문자라서 `..=i` 로 안전하게 자를 수 있어요.
fn first_sentence(text: &str) -> &str {
    match text.find(['.', '!', '?']) {
        Some(i) => &text[..=i],
        None => text,
    }
}

/// `&mut String`: 호출자의 String 을 "독점적으로 빌려" 그 자리에서 고칩니다. 호출은 `normalize(&mut s)`.
fn normalize(s: &mut String) {
    // 먼저 읽기 빌림(split_whitespace)으로 새 문자열을 만들고, 그 빌림이 끝난 뒤에 s 를 덮어씁니다.
    let normalized = s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    *s = normalized; // `*s` 는 "s 가 가리키는 String 자체". 대입하면 옛 값은 drop 됩니다.
}

/// `&mut [i32]`: Vec, 배열, 또는 그 일부분(`&mut v[1..3]`)을 빌려 고칩니다.
fn scale_in_place(nums: &mut [i32], k: i32) {
    for n in nums.iter_mut() {
        *n *= k;
    }
}

fn sum_slice(nums: &[i32]) -> i32 {
    nums.iter().sum()
}

/// `len()` 은 "바이트 수"입니다. 글자 수는 `chars().count()`.
fn char_count(s: &str) -> usize {
    s.chars().count()
}

/// 앞의 n 글자. `char_indices` 는 (바이트 위치, 글자) 쌍을 주므로 글자 경계에서 안전하게 자를 수 있어요.
fn first_n_chars(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((byte_index, _)) => &s[..byte_index],
        None => s,
    }
}

fn reverse_chars(s: &str) -> String {
    s.chars().rev().collect()
}

fn hex_bytes(s: &str) -> String {
    s.bytes().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}

fn main() {
    println!("== 1. text stats ==");
    println!("text: {TEXT}");
    println!("words: {}", word_count(TEXT));
    println!("longest: {}", longest_word(TEXT));
    println!("longest (tie): {}", longest_word("one two six"));
    println!("first sentence: {}", first_sentence(TEXT));
    println!("first sentence (no punctuation): {}", first_sentence("hello there"));

    println!("== 2. normalize in place ==");
    let mut s = String::from("  Hello,   WORLD   from  Rust!  ");
    println!("before: \"{s}\"");
    normalize(&mut s);
    println!("after: \"{s}\"");

    println!("== 3. slices of numbers ==");
    let mut nums = vec![1, 2, 3, 4, 5];
    println!("nums: {nums:?}");
    println!("sum: {}", sum_slice(&nums));
    scale_in_place(&mut nums, 3);
    println!("after scale x3: {nums:?}");
    println!("first three sum: {}", sum_slice(&nums[..3]));
    println!("last two sum: {}", sum_slice(&nums[3..]));

    println!("== 4. UTF-8 ==");
    println!("text: {KOREAN}");
    println!("words: {}", word_count(KOREAN));
    println!("bytes: {}", KOREAN.len());
    println!("chars: {}", char_count(KOREAN));
    println!("first 3 chars: {}", first_n_chars(KOREAN, 3));
    println!("first byte-slice [0..3]: {}", &KOREAN[0..3]);
    println!("reversed: {}", reverse_chars(KOREAN));
    println!("bytes of 가: {}", hex_bytes("가"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_count_basic() {
        assert_eq!(word_count("a b  c\td\n"), 4);
        assert_eq!(word_count(""), 0);
        assert_eq!(word_count("   "), 0);
        assert_eq!(word_count(KOREAN), 2);
    }

    #[test]
    fn longest_word_strips_punctuation_and_prefers_first() {
        assert_eq!(longest_word("hi, there! wonderful?"), "wonderful");
        assert_eq!(longest_word("one two six"), "one");
        assert_eq!(longest_word(""), "");
        assert_eq!(longest_word(TEXT), "afternoon");
    }

    #[test]
    fn longest_word_borrows_from_the_original() {
        let text = String::from("tiny enormous mid");
        let w = longest_word(&text);
        let start = text.as_ptr() as usize;
        let p = w.as_ptr() as usize;
        assert!(p >= start && p + w.len() <= start + text.len(), "새 문자열이 아니라 원본의 일부여야 해요");
    }

    #[test]
    fn first_sentence_cases() {
        assert_eq!(first_sentence("Hi there. Bye."), "Hi there.");
        assert_eq!(first_sentence("What? No!"), "What?");
        assert_eq!(first_sentence("Wow! ok."), "Wow!");
        assert_eq!(first_sentence("no end"), "no end");
        assert_eq!(first_sentence(""), "");
    }

    #[test]
    fn normalize_in_place() {
        let mut s = String::from("  Hello,\t WORLD \n from  Rust!  ");
        normalize(&mut s);
        assert_eq!(s, "hello, world from rust!");
        let mut empty = String::from("   ");
        normalize(&mut empty);
        assert_eq!(empty, "");
    }

    #[test]
    fn scale_only_the_borrowed_part() {
        let mut nums = [1, 2, 3, 4, 5];
        scale_in_place(&mut nums[1..4], 10);
        assert_eq!(nums, [1, 20, 30, 40, 5]);
        let mut v = vec![2, 4];
        scale_in_place(&mut v, -1);
        assert_eq!(v, [-2, -4]);
    }

    #[test]
    fn sum_of_slices() {
        let v = vec![1, 2, 3, 4];
        assert_eq!(sum_slice(&v), 10);
        assert_eq!(sum_slice(&v[1..3]), 5);
        assert_eq!(sum_slice(&[]), 0);
    }

    #[test]
    fn bytes_vs_chars() {
        assert_eq!(KOREAN.len(), 25);
        assert_eq!(char_count(KOREAN), 9);
        assert_eq!(char_count("abc"), 3);
        assert_eq!(char_count(""), 0);
    }

    #[test]
    fn first_n_chars_respects_boundaries() {
        assert_eq!(first_n_chars(KOREAN, 3), "안녕하");
        assert_eq!(first_n_chars(KOREAN, 0), "");
        assert_eq!(first_n_chars(KOREAN, 100), KOREAN);
        assert_eq!(first_n_chars("hello", 2), "he");
    }

    #[test]
    fn reverse_and_hex() {
        assert_eq!(reverse_chars("abc"), "cba");
        assert_eq!(reverse_chars("안녕"), "녕안");
        assert_eq!(hex_bytes("가"), "ea b0 80");
        assert_eq!(hex_bytes("A"), "41");
        assert_eq!(hex_bytes(""), "");
    }
}
