//! # Ch04. 참조·빌림·슬라이스 - 텍스트 분석기
//!
//! ## 목표
//! - `&T`(읽기 빌림) 와 `&mut T`(쓰기 빌림) 로 소유권을 넘기지 않고 값을 읽고 고친다.
//! - "`&mut` 하나 **또는** `&` 여러 개" 규칙(aliasing XOR mutation)과 "dangling 불가" 규칙을 체감한다.
//! - `String` vs `&str`, `Vec<T>` vs `&[T]` 를 구분하고, 함수 매개변수에는 `&str` / `&[T]` 를 쓰는 이유를 이해한다.
//! - UTF-8 문자열에서 **바이트 수 != 글자 수** 임을 한글 문장으로 확인한다.
//!
//! ## 대응 파일
//! - Python: `../python/main.py`
//! - C++   : `../cpp/main.cpp`   (`std::string_view`, `std::span` 과 비교해 보세요)
//!
//! ## 진행 방법
//! 1. `TODO(1)` ~ `TODO(10)` 을 순서대로 채웁니다. (`todo!()` 를 지우고 직접 구현)
//! 2. 하나 끝낼 때마다 `cargo test -p ch04_borrowing` 으로 확인합니다.
//! 3. 맨 아래 `experiments()` 의 🔬 실험을 하나씩 열어 컴파일러의 말을 읽어 보세요. (이 장의 핵심!)
//! 4. 막히면 Python / C++ 의 같은 이름 함수 -> 그래도 안 되면 `examples/solution.rs`.
//!
//! ## 완료 기준
//! - `cargo test -p ch04_borrowing` 이 모두 통과
//! - `../../scripts/compare.sh ch04` 에서 Python / C++ / Rust 출력이 똑같음 (한글 포함 바이트 단위로 동일)

// 다 풀고 나면 이 줄을 지워 보세요. (경고가 어떤 걸 알려주는지 읽어 보는 것도 공부예요)
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

const TEXT: &str =
    "The quick brown fox jumps over the lazy dog. It was a sunny day! Did you enjoy the afternoon?";
const KOREAN: &str = "안녕하세요 러스트";

// ─────────────────────────────────────────────
// TODO(1): 공백으로 나뉜 단어의 개수를 돌려주세요.
//   🐍 Python: len(text.split())                                     (python/main.py: word_count)
//   ⚙️ C++  : split_whitespace(text).size()                           (cpp/main.cpp: word_count)
//   🦀 Rust : 매개변수가 `&str` 이면 `&String`, 문자열 리터럴, 슬라이스를 전부 받을 수 있습니다. 읽기만 하니까 빌리면 충분해요.
//   💡 힌트: `text.split_whitespace()` 는 이터레이터 -> `.count()`
//   ❓ 왜?: `String` 을 받으면 호출자가 소유권을 잃거나 `.clone()` 해야 합니다. `&str` 이면 아무 부담이 없어요.
// ─────────────────────────────────────────────
fn word_count(text: &str) -> usize {
    todo!("TODO(1): 단어 수 세기")
}

// ─────────────────────────────────────────────
// TODO(2): 가장 긴 단어를 **원본 문자열의 슬라이스(&str)** 로 돌려주세요.
//          단어 양끝의 구두점(. , ! ? 등 ASCII 구두점)은 떼고, 길이가 같으면 "먼저 나온" 단어입니다.
//   🐍 Python: for w in text.split(): w = w.strip(punct); if len(w) > len(best): ...   (python/main.py: longest_word)
//   ⚙️ C++  : std::string_view 를 돌려줌 (원본보다 오래 쓰면 dangling!)               (cpp/main.cpp: longest_word)
//   🦀 Rust : 새 String 을 만들지 말고 text 의 일부를 빌려서 반환합니다. 컴파일러가 "원본보다 오래 못 산다" 를 보장해요.
//   💡 힌트: `text.split_whitespace()` 로 돌며 `w.trim_matches(|c: char| c.is_ascii_punctuation())`.
//           ⚠️ `max_by_key` 는 동률이면 **마지막** 요소를 돌려줘요 (Python max / C++ max_element 는 첫 번째).
//           테스트가 잡아 줄 거예요. `let mut best = "";` 에서 시작해 `if w.len() > best.len() { best = w; }` 가 가장 단순합니다.
//   ❓ 왜?: 시그니처 `fn(&str) -> &str` 에서 수명을 안 적어도 되는 건 "입력 참조가 하나면 출력은 그것에서 왔다" 는 생략 규칙 덕분입니다 (9장).
//   🔬 실험 6: 지역 String 을 만들어 그 일부를 반환하면? (E0515)
// ─────────────────────────────────────────────
fn longest_word(text: &str) -> &str {
    todo!("TODO(2): 가장 긴 단어를 슬라이스로")
}

// ─────────────────────────────────────────────
// TODO(3): 처음 나오는 `.` `!` `?` 까지(그 문자 포함)를 슬라이스로 돌려주세요. 없으면 text 전체.
//   🐍 Python: text[: min(ends) + 1]                                 (python/main.py: first_sentence)
//   ⚙️ C++  : text.substr(0, pos + 1)                                 (cpp/main.cpp: first_sentence)
//   🦀 Rust : `&text[..=i]` 처럼 범위로 자르면 복사 없이 "뷰" 가 만들어집니다 (Python 슬라이스는 복사).
//   💡 힌트: `text.find(['.', '!', '?'])` 는 `Option<usize>`(바이트 위치) -> `match` 로 Some / None 처리.
//           `..=i` 는 i 포함. 세 문자는 모두 1바이트라 `i + 1` 이 글자 경계예요.
//   ❓ 왜?: 슬라이스 경계가 UTF-8 글자 한가운데면 패닉합니다 (🔬 실험 11). `find` 가 준 위치는 항상 안전해요.
// ─────────────────────────────────────────────
fn first_sentence(text: &str) -> &str {
    todo!("TODO(3): 첫 문장 슬라이스")
}

// ─────────────────────────────────────────────
// TODO(4): s 를 **그 자리에서** 정리하세요: 앞뒤 공백 제거 + 연속 공백을 한 칸으로 + 소문자.
//   🐍 Python: " ".join(s.split()).lower()  (불변이라 새 문자열을 반환)   (python/main.py: normalize)
//   ⚙️ C++  : void normalize(std::string& s)                              (cpp/main.cpp: normalize)
//   🦀 Rust : `&mut String` = "독점적으로 빌려서 고칠 수 있는 String". 호출은 `normalize(&mut s)` 로 *호출부에도* 드러납니다.
//   💡 힌트: `let normalized = s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();`
//           그 다음 `*s = normalized;`  (`*s` = 빌린 String 자체. 대입하면 옛 값은 drop 됩니다)
//   ❓ 왜?: `s.split_whitespace()` 가 s 를 읽기 빌림하는 동안엔 s 를 못 바꿔요. 새 String 을 다 만든 뒤(빌림이 끝난 뒤)에 덮어쓰면 OK -> NLL.
//   🔬 실험 4: 호출부에서 `let s = String::from(..); normalize(&mut s);` 처럼 mut 를 빼먹으면? (E0596)
// ─────────────────────────────────────────────
fn normalize(s: &mut String) {
    todo!("TODO(4): s 를 제자리에서 정리")
}

// ─────────────────────────────────────────────
// TODO(5): nums 의 모든 원소에 k 를 곱하세요 (제자리에서).
//   🐍 Python: for i in range(len(nums)): nums[i] *= k               (python/main.py: scale_in_place)
//   ⚙️ C++  : void scale_in_place(std::span<int> nums, int k)          (cpp/main.cpp: scale_in_place)
//   🦀 Rust : `&mut [i32]` 는 Vec 전체든 일부(`&mut v[1..3]`)든 빌려서 고칠 수 있는 슬라이스입니다.
//   💡 힌트: `for n in nums.iter_mut() { *n *= k; }`  — `n` 은 `&mut i32` 라서 `*n` 으로 값에 접근해요.
//   ❓ 왜?: `iter_mut()` 으로 도는 동안에는 nums 에 다른 접근이 불가능해서 "순회 중 push" 같은 반복자 무효화 버그가 컴파일 에러가 됩니다 (🔬 실험 3).
// ─────────────────────────────────────────────
fn scale_in_place(nums: &mut [i32], k: i32) {
    todo!("TODO(5): 모든 원소에 k 곱하기")
}

// ─────────────────────────────────────────────
// TODO(6): nums 의 합을 돌려주세요. 빈 슬라이스면 0.
//   🐍 Python: sum(nums)                                              (python/main.py: sum_slice)
//   ⚙️ C++  : std::span<const int> 순회                                (cpp/main.cpp: sum_slice)
//   🦀 Rust : `&[i32]` 는 읽기 전용 슬라이스. `&v`(Vec 전체) 도 `&v[1..3]` 도 `&[1, 2, 3]`(배열)도 넘길 수 있어요.
//   💡 힌트: `nums.iter().sum()`  (반환 타입이 i32 라서 타입 추론이 됩니다)
// ─────────────────────────────────────────────
fn sum_slice(nums: &[i32]) -> i32 {
    todo!("TODO(6): 슬라이스의 합")
}

// ─────────────────────────────────────────────
// TODO(7): s 의 "글자 수" 를 돌려주세요. (바이트 수 아님!)  "안녕하세요 러스트" -> 9 (바이트는 25)
//   🐍 Python: len(s)  (str 은 코드포인트 단위)                        (python/main.py: char_count)
//   ⚙️ C++  : UTF-8 이어짐 바이트(0b10xxxxxx)를 제외하고 세기          (cpp/main.cpp: char_count)
//   🦀 Rust : `s.len()` 은 **바이트** 수. 글자 수는 `s.chars().count()` (O(n) 이에요).
//   💡 힌트: `chars()` 는 UTF-8 을 해석해서 `char`(유니코드 스칼라 값)를 하나씩 줍니다.
//   ❓ 왜?: Rust 의 `String` 은 항상 올바른 UTF-8 이라는 것을 타입이 보장합니다. 그래서 `s[0]` 같은 인덱싱은 막혀 있어요 (🔬 실험 10).
// ─────────────────────────────────────────────
fn char_count(s: &str) -> usize {
    todo!("TODO(7): 글자 수 세기")
}

// ─────────────────────────────────────────────
// TODO(8): 앞의 n 글자를 슬라이스로 돌려주세요. n 이 글자 수 이상이면 s 전체.
//   🐍 Python: s[:n]                                                  (python/main.py: first_n_chars)
//   ⚙️ C++  : UTF-8 글자 경계를 직접 찾아서 substr                      (cpp/main.cpp: first_n_chars)
//   🦀 Rust : 글자 n 개의 "바이트 위치" 를 찾아서 `&s[..위치]` 로 자릅니다.
//   💡 힌트: `s.char_indices().nth(n)` -> `Option<(바이트위치, char)>`.  Some((i, _)) => &s[..i], None => s
//   ❓ 왜?: `&s[0..n]` 처럼 글자 수를 바이트 위치로 착각하면 한글에서 패닉합니다 (🔬 실험 11).
// ─────────────────────────────────────────────
fn first_n_chars(s: &str, n: usize) -> &str {
    todo!("TODO(8): 앞 n 글자")
}

// ─────────────────────────────────────────────
// TODO(9): 글자 순서를 뒤집은 새 String 을 돌려주세요. "안녕" -> "녕안"
//   🐍 Python: s[::-1]                                                (python/main.py: reverse_chars)
//   ⚙️ C++  : UTF-8 글자 단위로 쪼개 거꾸로 이어 붙이기                  (cpp/main.cpp: reverse_chars)
//   🦀 Rust : 바이트를 뒤집으면 한글이 깨져요. `chars()` 를 `rev()` 로 뒤집어 `collect()`.
//   💡 힌트: `s.chars().rev().collect()`  (반환 타입이 String 이므로 collect 가 String 을 만듭니다)
//   ❓ 왜?: 여기서는 새 값을 만들어 돌려주므로 반환 타입이 `&str` 이 아니라 `String`(소유) 입니다. 빌린 것을 반환할 수 없는 경우예요.
// ─────────────────────────────────────────────
fn reverse_chars(s: &str) -> String {
    todo!("TODO(9): 글자 단위로 뒤집기")
}

// ─────────────────────────────────────────────
// TODO(10): s 의 UTF-8 바이트를 소문자 16진수 두 자리씩, 공백으로 이어 돌려주세요. "가" -> "ea b0 80"
//   🐍 Python: " ".join(f"{b:02x}" for b in s.encode())               (python/main.py: hex_bytes)
//   ⚙️ C++  : std::format("{:02x}", ...)                               (cpp/main.cpp: hex_bytes)
//   🦀 Rust : `s.bytes()` 는 `u8` 이터레이터. `format!("{b:02x}")` 로 각 바이트를 문자열로 만들어 `join(" ")`.
//   💡 힌트: `.map(|b| format!(...)).collect::<Vec<_>>().join(" ")`
//   ❓ 왜?: 한글 '가'(U+AC00)가 3바이트라는 걸 눈으로 확인하는 게 목적이에요. ASCII 는 1바이트, 한글 대부분은 3바이트입니다.
// ─────────────────────────────────────────────
fn hex_bytes(s: &str) -> String {
    todo!("TODO(10): 바이트를 16진수로")
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


// ═════════════════════════════════════════════
// 🔬 실험실: 빌림 검사기(borrow checker)와 대화하기  (이 장의 핵심!)
//   한 블록씩 주석(`// `)을 풀고 `cargo check -p ch04_borrowing` 를 실행해 에러 메시지를 읽으세요.
//   다 본 뒤에는 다시 주석 처리하세요. (이 함수는 어디서도 호출되지 않아요)
//   에러 메시지 읽는 법: ① "first borrow occurs here" 어디서 빌렸나 ② "second borrow occurs here" 무엇과 충돌하나
//   ③ "first borrow later used here" 왜 아직 끝나지 않았나 -- 이 세 지점이 늘 같은 모양입니다.
// ═════════════════════════════════════════════
fn experiments() {
    // [실험 1] 같은 값을 &mut 로 두 번.  기대 에러: E0499 (cannot borrow `s` as mutable more than once at a time)
    //   ❓ 질문: `a.push('!')` 줄을 지우면(= a 를 이후에 쓰지 않으면) 에러가 사라질까요? (NLL: 빌림은 "마지막 사용" 까지)
    //
    // let mut s = String::from("hello");
    // let a = &mut s;
    // let b = &mut s;
    // a.push('!');
    // b.push('?');

    // [실험 2] 읽기 빌림이 살아 있는 동안 수정.  기대 에러: E0502 (cannot borrow `v` as mutable because it is also borrowed as immutable)
    //   ❓ 질문: C++ 이라면? `auto& first = v[0]; v.push_back(4);` 후 first 를 읽으면 재할당된 메모리를 읽는 UB 입니다.
    //
    // let mut v = vec![1, 2, 3];
    // let first = &v[0];
    // v.push(4);
    // println!("{first}");

    // [실험 3] 순회 중에 컬렉션 수정 (반복자 무효화).  기대 에러: E0502
    //   ❓ 질문: C++ 에서 range-for 안에서 push_back 하는 버그와 같은 상황입니다. Rust 는 어떻게 막나요?
    //
    // let mut v = vec![1, 2, 3];
    // for x in &v {
    //     if *x == 2 { v.push(10); }
    // }

    // [실험 4] mut 없이 / `&` 로 받은 걸 수정.  기대 에러: E0596 (cannot borrow `s` as mutable, as it is not declared as mutable)
    //   ❓ 질문: 두 번째 형태(`fn f(s: &String) { s.push('!') }`)에서 컴파일러가 제안하는 수정은?
    //
    // let s = String::from("hi");
    // s.push('!');
    //
    //   (두 번째 형태: `&` 로 빌린 것을 수정 — 기대 에러: E0596 (`s` is a `&` reference, so it cannot be borrowed as mutable))
    // fn add_exclaim(s: &String) {
    //     s.push('!');
    // }

    // [실험 5] dangling 참조 반환 1: 수명 지정 불가.  기대 에러: E0106 (missing lifetime specifier)
    //   ❓ 질문: 에러 메시지의 "this function's return type contains a borrowed value, but there is no value for it to be borrowed from" 가 무슨 뜻인가요?
    //
    // fn dangle() -> &String {
    //     let s = String::from("hi");
    //     &s
    // }

    // [실험 6] dangling 참조 반환 2: 지역 변수 참조.  기대 에러: E0515 (cannot return value referencing local variable `owned`)
    //   ❓ 질문: TODO(2) 의 longest_word 가 `to_lowercase()` 로 만든 새 String 의 단어를 반환하려 하면 같은 에러입니다. 왜 그럴까요?
    //
    // fn first_lower(text: &str) -> &str {
    //     let owned = text.to_lowercase();
    //     owned.split_whitespace().next().unwrap()
    // }

    // [실험 7] 스코프 밖으로 새는 참조.  기대 에러: E0597 (`s` does not live long enough)
    //   ❓ 질문: C++ 의 `int* p; { int x = 1; p = &x; } *p;` 와 같은 상황이죠. Rust 는 어느 줄에서 "x 가 죽는다" 고 알려 주나요?
    //
    // let r;
    // {
    //     let s = String::from("short-lived");
    //     r = &s;
    // }
    // println!("{r}");

    // [실험 8] 빌려 준 동안 소유권 이동.  기대 에러: E0505 (cannot move out of `s` because it is borrowed)
    //
    // let s = String::from("hello");
    // let r = &s;
    // let moved = s;
    // println!("{r} {moved}");

    // [실험 9] String 이 필요한 곳에 &str.  기대 에러: E0308 (expected `String`, found `&str`)
    //   ❓ 질문: 컴파일러의 제안(`t.to_string()`)은 무슨 일을 하나요? (힙 할당 + 복사) 반대로 &String -> &str 은 왜 자동으로 되나요? (deref 강제 변환)
    //
    // fn takes_string(s: String) { println!("{s}"); }
    // takes_string("literal");

    // [실험 10] 문자열 인덱싱.  기대 에러: E0277 (the type `str` cannot be indexed by `{integer}`)
    //   ❓ 질문: 왜 막아 놨을까요? (한글은 글자당 3바이트라 "n 번째 글자" 가 O(1) 이 아니고, 바이트 하나는 글자가 아닐 수 있어요)
    //
    // let s = String::from("안녕");
    // let c = s[0];

    // [실험 11] 글자 한가운데를 자르면 -> 컴파일은 되지만 "실행 중 패닉".
    //   기대 출력: byte index ... is not a char boundary; it is inside '안' (bytes 0..3 of string)
    //   ❓ 질문: `&s[0..3]` 으로 바꾸면? 안전하게 앞 n 글자를 얻는 방법은? (TODO(8))
    //
    // let s = String::from("안녕하세요");
    // let part = &s[0..2];
    // println!("{part}");

    // [실험 12] 인자가 둘인데 참조를 돌려주면? 기대 에러: E0106 (...whether it is borrowed from `a` or `b`)
    //   ❓ 질문: 컴파일러가 제안하는 `<'a>` 는 무엇일까요? (9장 라이프타임에서 정식으로 배웁니다)
    //
    // fn longest(a: &str, b: &str) -> &str {
    //     if a.len() >= b.len() { a } else { b }
    // }

    // [실험 13] 고전 문제: 슬라이스를 들고 있는데 원본을 비우면.  기대 에러: E0502
    //   ❓ 질문: Python 이라면 word 는 복사본이라 안전합니다. Rust 는 복사 없이 가리키기만 하므로 이런 규칙이 필요해요.
    //
    // let mut s = String::from("hello world");
    // let word = s.split(' ').next().unwrap();
    // s.clear();
    // println!("{word}");
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
