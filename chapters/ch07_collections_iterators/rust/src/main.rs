//! # Ch07. 컬렉션·클로저·이터레이터 — 성적 데이터 분석
//!
//! ## 목표
//! - `Vec` / `HashMap` / `HashSet` / `BTreeMap` 을 용도에 맞게 고르고 `entry` API 를 쓴다.
//! - 클로저(`Fn` / `FnMut`)를 만들고 함수에 넘기고 돌려준다.
//! - 이터레이터 어댑터(`map`/`filter`/`filter_map`/`collect`/`sum`/`count`)와 `iter` / `iter_mut` 를 구분한다.
//!
//! ## 대응 파일
//! - Python : `../python/main.py`  (defaultdict, Counter, 컴프리헨션)
//! - C++    : `../cpp/main.cpp`    (std::map, ranges::views, 람다)
//!
//! ## 진행 방법
//! 1. `TODO(1)` 부터 번호 순서대로 `todo!()` 를 구현합니다.
//! 2. `cargo test -p ch07_collections_iterators` 로 확인합니다.
//! 3. 프로그램은 `data/scores.csv` 를 읽으므로 **챕터 디렉터리**에서 `cargo run -q -p ch07_collections_iterators` 로 실행하세요.
//!
//! ## 완료 기준
//! `cargo test -p ch07_collections_iterators` 가 모두 초록불이고
//! `../../scripts/compare.sh ch07` 에서 세 언어의 출력이 같으면 끝입니다.

// 다 풀고 나면 이 줄을 지워 보세요 (아직 쓰지 않은 코드에 대한 경고를 잠시 꺼 둡니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;

#[derive(Debug, Clone, PartialEq)]
struct Record {
    student: String,
    subject: String,
    score: u32,
}

// ─────────────────────────────────────────────
// TODO(1): "alice,math,92" 한 줄을 Record 로 바꿉니다. 각 필드의 앞뒤 공백은 제거합니다.
//   필드가 3개가 아니거나, 이름/과목이 비었거나, 점수가 (음수 포함) 숫자가 아니면 None.
//   🐍 Python: [p.strip() for p in line.split(",")]            (python/main.py: parse_record)
//   ⚙️ C++  : getline(ss, p, ',') 반복                          (cpp/main.cpp: parse_record)
//   🦀 Rust : `let mut parts = line.split(',').map(str::trim);` 로 "게으른" 이터레이터를 만들고
//             `parts.next()?` 로 하나씩 꺼냅니다. (`?` 는 Option 에서도 동작: None 이면 함수가 None 반환)
//   💡 힌트: 점수는 `parts.next()?.parse().ok()?` — parse 의 Result 를 `.ok()` 로 Option 으로 바꿉니다.
//           마지막에 `parts.next().is_some()` 이면 필드가 4개 이상이라는 뜻입니다.
//   ❓ 왜?: split 은 한꺼번에 Vec 을 만들지 않고 필요할 때마다 다음 조각을 찾습니다(laziness).
// ─────────────────────────────────────────────
fn parse_record(line: &str) -> Option<Record> {
    todo!("TODO(1): 한 줄 -> Option<Record>")
}

// ─────────────────────────────────────────────
// TODO(2): 전체 텍스트를 읽습니다. 첫 줄(헤더)과 공백뿐인 줄은 무시하고,
//   (읽은 레코드들, 형식이 잘못되어 건너뛴 줄 수) 를 돌려줍니다.
//   🐍 Python: [r for l in lines if (r := parse_record(l)) is not None]   (python/main.py: parse_records)
//   ⚙️ C++  : for 루프 + push_back                                        (cpp/main.cpp: parse_records)
//   🦀 Rust : `text.lines().skip(1).filter(..).collect()` → `.filter_map(parse_record).collect()`
//   💡 힌트: `filter_map(f)` 는 f 가 Some 인 것만 벗겨서 모읍니다.
//           중간 결과를 `let lines: Vec<&str> = ...collect();` 로 한 번 모아 두면 개수를 셀 수 있어요.
//           `.filter_map(|l| parse_record(l))` 에서 `l` 이 `&&str` 일 수 있어 클로저로 감싸면 편합니다.
//   ❓ 왜?: collect() 가 돌려줄 타입은 변수에 적은 타입(`Vec<Record>`)으로 정해집니다 (타입 추론).
// ─────────────────────────────────────────────
fn parse_records(text: &str) -> (Vec<Record>, usize) {
    todo!("TODO(2): skip + filter + filter_map + collect")
}

// ─────────────────────────────────────────────
// TODO(3): 학생별 평균. 이름 순으로 정렬된 BTreeMap<String, f64> 로 돌려줍니다.
//   🐍 Python: defaultdict(list) 에 append                    (python/main.py: student_averages)
//   ⚙️ C++  : std::map<string, vector<int>> 의 operator[]      (cpp/main.cpp: student_averages)
//   🦀 Rust : entry API — `map.entry(키).or_default().push(값)`
//   💡 힌트: 먼저 `BTreeMap<String, Vec<u32>>` 에 점수를 모은 뒤,
//           `.into_iter().map(|(name, v)| (name, 평균)).collect()` 로 평균 맵을 만듭니다.
//           평균 = `v.iter().sum::<u32>() as f64 / v.len() as f64`.
//   ❓ 왜?: entry 는 "있으면 그 자리, 없으면 새로 넣고 그 자리"를 한 번의 조회로 처리해 줍니다
//           (두 번 조회하다가 생기는 버그/비효율 방지).
// ─────────────────────────────────────────────
fn student_averages(records: &[Record]) -> BTreeMap<String, f64> {
    todo!("TODO(3): entry().or_default() 로 모으고 평균 내기")
}

// ─────────────────────────────────────────────
// TODO(4): 과목별 최고 득점자: 과목 -> (학생, 점수). 동점이면 먼저 나온 사람을 유지합니다.
//   🐍 Python: dict.get + 비교                                (python/main.py: subject_tops)
//   ⚙️ C++  : map.find                                        (cpp/main.cpp: subject_tops)
//   🦀 Rust : `entry(과목).and_modify(|best| ...).or_insert_with(|| ...)`
//   💡 힌트: and_modify 의 클로저는 기존 값의 `&mut` 를 받습니다. 더 높은 점수일 때만 `*best = (...)`.
//   ❓ 왜?: 클로저가 주변 변수(`r`)를 "빌려서" 씁니다. 이게 C++ 의 `[&]` 캡처에 해당합니다.
// ─────────────────────────────────────────────
fn subject_tops(records: &[Record]) -> BTreeMap<String, (String, u32)> {
    todo!("TODO(4): and_modify + or_insert_with")
}

// ─────────────────────────────────────────────
// TODO(5): 점수 -> 등급. 90 이상 'A', 80 이상 'B', 70 이상 'C', 60 이상 'D', 나머지 'F'.
//   그리고 등급 -> 개수 HashMap 을 만드는 grade_histogram 도 구현합니다. (아래 두 함수 모두 TODO(5))
//   🐍 Python: Counter(grade_of(s) for s in scores)           (python/main.py: grade_histogram)
//   ⚙️ C++  : unordered_map 의 ++hist[g]                       (cpp/main.cpp: grade_histogram)
//   🦀 Rust : 범위 패턴 `90.. => 'A'`, `80..=89 => 'B'` / `*hist.entry(g).or_insert(0) += 1`
//   💡 힌트: entry(..).or_insert(0) 는 `&mut usize` 를 돌려주므로 앞에 `*` 를 붙여 값을 올립니다.
//   ❓ 왜?: HashMap 은 순서가 보장되지 않습니다. 그래서 main 에서는 'A'..'F' 를 직접 순회하며 꺼냅니다.
// ─────────────────────────────────────────────
fn grade_of(score: u32) -> char {
    todo!("TODO(5): 범위 패턴 match")
}

fn grade_histogram(scores: &[u32]) -> HashMap<char, usize> {
    todo!("TODO(5): entry API 로 개수 세기")
}

// ─────────────────────────────────────────────
// TODO(6): "bonus 점을 더하고 100 으로 제한하는" 클로저를 만들어 돌려줍니다.
//   🐍 Python: return lambda s: min(s + bonus, 100)           (python/main.py: make_curve)
//   ⚙️ C++  : return [bonus](int s) { ... };  (std::function) (cpp/main.cpp: make_curve)
//   🦀 Rust : 반환 타입 `impl Fn(u32) -> u32` — 클로저의 정확한 타입은 컴파일러만 아는 익명 타입입니다.
//   💡 힌트: `move |s| (s + bonus).min(100)`. `move` 가 없으면 bonus 를 "빌리는" 클로저가 되어
//           함수가 끝나는 순간 dangling → 컴파일 에러(E0373)입니다.
//   ❓ 왜?: C++ 에서는 `[&bonus]` 로 잘못 쓰면 컴파일은 되지만 런타임에 터집니다. Rust 는 컴파일러가 막아 줍니다.
// ─────────────────────────────────────────────
fn make_curve(bonus: u32) -> impl Fn(u32) -> u32 {
    // 아래 임시 클로저를 지우고 직접 만드세요. (todo! 가 클로저 "안"에 있어야 컴파일됩니다)
    move |s: u32| -> u32 { todo!("TODO(6): (s + bonus) 를 100 으로 제한") }
}

// ─────────────────────────────────────────────
// TODO(7): 슬라이스의 모든 점수에 f 를 "제자리에서" 적용합니다.
//   🐍 Python: for i, s in enumerate(scores): scores[i] = f(s)   (python/main.py: apply_in_place)
//   ⚙️ C++  : for (int& s : scores) s = f(s);                    (cpp/main.cpp: apply_in_place)
//   🦀 Rust : `for s in scores.iter_mut() { *s = f(*s); }`
//   💡 힌트: iter() 는 `&T`, iter_mut() 는 `&mut T`, into_iter() 는 `T`(소유권 이동)를 꺼냅니다.
//   ❓ 왜?: 순회 중에 `scores.push(..)` 같은 걸 하면 어떻게 될까요? (🔬 실험 1 참고)
// ─────────────────────────────────────────────
fn apply_in_place(scores: &mut [u32], f: impl Fn(u32) -> u32) {
    todo!("TODO(7): iter_mut")
}

// ─────────────────────────────────────────────
// TODO(8): (60점 이상인 개수, 조건 검사 클로저가 호출된 횟수) 를 돌려줍니다.
//   클로저 안에서 `calls` 라는 지역 변수를 1씩 증가시키세요 — 상태를 바꾸는 클로저는 FnMut 입니다.
//   🐍 Python: nonlocal calls                                   (python/main.py: count_passing)
//   ⚙️ C++  : [&calls](int s) { ++calls; ... }                  (cpp/main.cpp: count_passing)
//   🦀 Rust : `scores.iter().filter(|&&s| { calls += 1; s >= 60 }).count()`
//   💡 힌트: iter() 의 원소는 `&u32`, filter 는 거기에 또 `&` 를 씌워 `&&u32` 를 줍니다. 패턴 `|&&s|` 로 벗깁니다.
//           `let mut calls = 0;` 이어야 합니다. 체인이 끝난(count 호출 후) 다음 줄에서 calls 를 읽을 수 있어요.
//   ❓ 왜?: 어댑터는 게으르기 때문에 `count()` 같은 소비자(consumer)가 있어야 비로소 실행됩니다.
// ─────────────────────────────────────────────
fn count_passing(scores: &[u32]) -> (usize, usize) {
    todo!("TODO(8): FnMut 클로저 + filter + count")
}

// ─────────────────────────────────────────────
// TODO(9): 평균 내림차순, 동점이면 이름 오름차순으로 정렬한 Vec<(이름, 평균)>.
//   🐍 Python: sorted(items, key=lambda kv: (-kv[1], kv[0]))     (python/main.py: ranking)
//   ⚙️ C++  : std::sort + 비교 람다                              (cpp/main.cpp: ranking)
//   🦀 Rust : `v.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)))`
//   💡 힌트: f64 는 NaN 때문에 `Ord` 가 아니라서 `sort()` 를 못 씁니다. `total_cmp` 를 쓰세요.
//           `Ordering::then_with` 는 "앞 비교가 같을 때만 다음 기준을 본다"입니다.
//           먼저 `averages.iter().map(|(k, v)| (k.clone(), *v)).collect()` 로 Vec 을 만드세요.
//   ❓ 왜?: sort_by 의 클로저는 비교 규칙을 "데이터"처럼 넘기는 것입니다 (전략 패턴이 한 줄로).
// ─────────────────────────────────────────────
fn ranking(averages: &BTreeMap<String, f64>) -> Vec<(String, f64)> {
    todo!("TODO(9): Vec 으로 모으고 sort_by")
}

// ─────────────────────────────────────────────
// TODO(10): 등장한 과목 이름 목록 (중복 제거, 사전순).
//   🐍 Python: sorted({r.subject for r in records})              (python/main.py: distinct_subjects)
//   ⚙️ C++  : std::set<string>                                   (cpp/main.cpp: distinct_subjects)
//   🦀 Rust : HashSet 에 모아 중복을 없애고, Vec 으로 옮겨 sort() 합니다.
//   💡 힌트: `records.iter().map(|r| r.subject.as_str()).collect::<HashSet<&str>>()`
//           HashSet 은 순서가 없으므로 출력 전에 정렬이 필수입니다. (BTreeSet 을 써도 되지만 연습은 HashSet 으로!)
//   ❓ 왜?: `HashSet<&str>` 는 원본 레코드의 문자열을 "빌려" 가리킬 뿐 복사하지 않습니다.
// ─────────────────────────────────────────────
fn distinct_subjects(records: &[Record]) -> Vec<String> {
    todo!("TODO(10): HashSet 으로 중복 제거 후 정렬")
}

// ── 제공되는 코드 ───────────────────────────────────────────

fn print_histogram(title: &str, scores: &[u32]) {
    println!("{title}");
    let hist = grade_histogram(scores);
    for g in ['A', 'B', 'C', 'D', 'F'] {
        let n = hist.get(&g).copied().unwrap_or(0);
        println!("{g}: {} ({n})", "#".repeat(n));
    }
}

fn main() {
    let text = fs::read_to_string("data/scores.csv").expect("data/scores.csv 를 읽을 수 없어요");
    let (records, skipped) = parse_records(&text);
    println!("records: {} (skipped {skipped})", records.len());
    let subjects = distinct_subjects(&records);
    println!("subjects: {}", subjects.join(", "));

    println!("== average per student ==");
    let averages = student_averages(&records);
    for (name, avg) in &averages {
        println!("{name:<6}: {avg:.2}");
    }

    println!("== top per subject ==");
    for (subject, (name, score)) in subject_tops(&records) {
        println!("{subject:<8}: {name} ({score})");
    }

    // 이터레이터 체인: 레코드에서 점수만 뽑아 Vec<u32> 로 모읍니다.
    let mut scores: Vec<u32> = records.iter().map(|r| r.score).collect();
    print_histogram("== grade histogram ==", &scores);

    let curve = make_curve(5);
    apply_in_place(&mut scores, &curve);
    print_histogram("== after +5 curve ==", &scores);

    let raw: Vec<u32> = records.iter().map(|r| r.score).collect();
    let (passing, calls) = count_passing(&raw);
    println!("== passing ==");
    println!("passing: {passing} of {} (predicate called {calls} times)", raw.len());

    println!("== ranking ==");
    for (i, (name, avg)) in ranking(&averages).iter().enumerate() {
        println!("{}. {name:<6} {avg:.2}", i + 1);
    }

    // 🔬 실험 1: 순회 중 수정. 아래 주석을 풀어 보세요. (기대: E0502 cannot borrow `scores` as mutable because it is also borrowed as immutable)
    //   ❓ Python 에서는 같은 코드가 어떻게 동작하나요? C++ 에서는요? (반복자 무효화 = 미정의 동작)
    // for s in scores.iter() {
    //     if *s > 90 {
    //         scores.push(*s);
    //     }
    // }

    // 🔬 실험 2: 클로저가 소유권을 가져가는 경우 (FnOnce). 주석을 풀어 보세요. (기대: E0382 use of moved value: `consume`)
    //   `drop(names)` 처럼 소유권을 소비하는 클로저는 한 번밖에 못 부릅니다.
    // let names = vec!["a".to_string()];
    // let consume = move || drop(names);
    // consume();
    // consume();

    // 🔬 실험 3: TODO(6) 의 make_curve 에서 `move` 를 지워 보세요. (기대: E0373 closure may outlive the current function)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str = "student,subject,score\n\
        alice,math,90\nalice,english,80\nbob,math,70\n bob , english , 61\nbob,science,xx\n\ncarol,math,90\n";

    fn recs() -> Vec<Record> {
        parse_records(CSV).0
    }

    #[test]
    fn parse_record_ok_and_bad() {
        assert_eq!(
            parse_record(" eve , art , 77 "),
            Some(Record {
                student: "eve".into(),
                subject: "art".into(),
                score: 77
            })
        );
        assert_eq!(parse_record("a,b"), None);
        assert_eq!(parse_record("a,b,c"), None);
        assert_eq!(parse_record("a,b,1,2"), None);
        assert_eq!(parse_record("a,b,-3"), None);
        assert_eq!(parse_record(""), None);
    }

    #[test]
    fn parse_records_counts_skipped() {
        let (r, skipped) = parse_records(CSV);
        assert_eq!(r.len(), 5);
        assert_eq!(skipped, 1);
        assert_eq!(r[3].student, "bob");
    }

    #[test]
    fn averages_are_per_student_and_sorted() {
        let avg = student_averages(&recs());
        let names: Vec<&str> = avg.keys().map(|s| s.as_str()).collect();
        assert_eq!(names, ["alice", "bob", "carol"]);
        assert!((avg["alice"] - 85.0).abs() < 1e-9);
        assert!((avg["bob"] - 65.5).abs() < 1e-9);
    }

    #[test]
    fn tops_keep_first_on_tie() {
        let tops = subject_tops(&recs());
        assert_eq!(tops["math"], ("alice".to_string(), 90));
        assert_eq!(tops["english"], ("alice".to_string(), 80));
        assert_eq!(tops.len(), 2);
    }

    #[test]
    fn grades_and_histogram() {
        assert_eq!(grade_of(100), 'A');
        assert_eq!(grade_of(90), 'A');
        assert_eq!(grade_of(89), 'B');
        assert_eq!(grade_of(70), 'C');
        assert_eq!(grade_of(60), 'D');
        assert_eq!(grade_of(59), 'F');
        let h = grade_histogram(&[95, 91, 85, 10, 0]);
        assert_eq!(h[&'A'], 2);
        assert_eq!(h[&'B'], 1);
        assert_eq!(h[&'F'], 2);
        assert_eq!(h.get(&'C'), None);
    }

    #[test]
    fn curve_closure_caps_at_100() {
        let c = make_curve(5);
        assert_eq!(c(50), 55);
        assert_eq!(c(98), 100);
        let mut v = vec![10, 96];
        apply_in_place(&mut v, make_curve(10));
        assert_eq!(v, [20, 100]);
    }

    #[test]
    fn fnmut_counts_calls() {
        assert_eq!(count_passing(&[59, 60, 100, 0]), (2, 4));
        assert_eq!(count_passing(&[]), (0, 0));
    }

    #[test]
    fn ranking_sorts_desc_then_name() {
        let mut m = BTreeMap::new();
        m.insert("zed".to_string(), 80.0);
        m.insert("amy".to_string(), 80.0);
        m.insert("bo".to_string(), 90.5);
        let r = ranking(&m);
        let names: Vec<&str> = r.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["bo", "amy", "zed"]);
    }

    #[test]
    fn distinct_subjects_sorted() {
        assert_eq!(distinct_subjects(&recs()), ["english", "math"]);
    }
}
