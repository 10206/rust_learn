//! Ch07 정답: 성적 데이터 분석
//!
//! 실행: `cargo run -q -p ch07_collections_iterators --example solution`
//! 테스트: `cargo test -p ch07_collections_iterators --example solution`

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;

#[derive(Debug, Clone, PartialEq)]
struct Record {
    student: String,
    subject: String,
    score: u32,
}

/// "alice,math,92" -> Some(Record). 필드 수가 다르거나 점수가 숫자가 아니면 None.
fn parse_record(line: &str) -> Option<Record> {
    let mut parts = line.split(',').map(str::trim);
    let student = parts.next()?;
    let subject = parts.next()?;
    let score = parts.next()?.parse().ok()?;
    if parts.next().is_some() || student.is_empty() || subject.is_empty() {
        return None;
    }
    Some(Record {
        student: student.to_string(),
        subject: subject.to_string(),
        score,
    })
}

/// 헤더(첫 줄)와 빈 줄을 건너뛰고 읽습니다. (읽은 레코드, 건너뛴 줄 수) 를 돌려줍니다.
fn parse_records(text: &str) -> (Vec<Record>, usize) {
    let lines: Vec<&str> = text
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .collect();
    // filter_map: Some 만 남기고 벗겨 줍니다.
    let records: Vec<Record> = lines.iter().filter_map(|l| parse_record(l)).collect();
    let skipped = lines.len() - records.len();
    (records, skipped)
}

/// 학생별 평균. BTreeMap 은 키 순서대로 정렬되어 있어 출력이 결정적입니다.
fn student_averages(records: &[Record]) -> BTreeMap<String, f64> {
    let mut scores: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    for r in records {
        // entry API: 없으면 빈 Vec 을 넣고, 있든 없든 그 값의 &mut 를 돌려줍니다.
        scores.entry(r.student.clone()).or_default().push(r.score);
    }
    scores
        .into_iter()
        .map(|(name, v)| {
            let sum: u32 = v.iter().sum();
            (name, sum as f64 / v.len() as f64)
        })
        .collect()
}

/// 과목별 최고 득점자: 과목 -> (학생, 점수). 동점이면 먼저 나온 사람.
fn subject_tops(records: &[Record]) -> BTreeMap<String, (String, u32)> {
    let mut tops: BTreeMap<String, (String, u32)> = BTreeMap::new();
    for r in records {
        tops.entry(r.subject.clone())
            .and_modify(|best| {
                if r.score > best.1 {
                    *best = (r.student.clone(), r.score);
                }
            })
            .or_insert_with(|| (r.student.clone(), r.score));
    }
    tops
}

fn grade_of(score: u32) -> char {
    match score {
        90.. => 'A',
        80..=89 => 'B',
        70..=79 => 'C',
        60..=69 => 'D',
        _ => 'F',
    }
}

/// 등급 -> 개수. HashMap 은 순서가 없으니 출력하는 쪽에서 A..F 순으로 꺼냅니다.
fn grade_histogram(scores: &[u32]) -> HashMap<char, usize> {
    let mut hist = HashMap::new();
    for &s in scores {
        *hist.entry(grade_of(s)).or_insert(0) += 1;
    }
    hist
}

/// 점수에 bonus 를 더하고 100 으로 제한하는 클로저를 만들어 돌려줍니다.
/// `move`: bonus 를 클로저 안으로 복사해 가져갑니다 (함수가 끝나도 살아 있어야 하니까요).
fn make_curve(bonus: u32) -> impl Fn(u32) -> u32 {
    move |s| (s + bonus).min(100)
}

/// 모든 점수에 f 를 제자리에서 적용합니다. (iter_mut: 각 원소의 &mut 를 빌려 줍니다)
fn apply_in_place(scores: &mut [u32], f: impl Fn(u32) -> u32) {
    for s in scores.iter_mut() {
        *s = f(*s);
    }
}

/// (60점 이상 개수, 조건 검사가 호출된 횟수). 클로저가 `calls` 를 수정하므로 FnMut 입니다.
fn count_passing(scores: &[u32]) -> (usize, usize) {
    let mut calls = 0;
    let passing = scores
        .iter()
        .filter(|&&s| {
            calls += 1;
            s >= 60
        })
        .count();
    (passing, calls)
}

/// 평균 내림차순, 동점이면 이름 오름차순으로 정렬한 순위표.
fn ranking(averages: &BTreeMap<String, f64>) -> Vec<(String, f64)> {
    let mut v: Vec<(String, f64)> = averages.iter().map(|(k, v)| (k.clone(), *v)).collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v
}

/// 등장한 과목 이름(중복 제거, 사전순).
fn distinct_subjects(records: &[Record]) -> Vec<String> {
    let set: HashSet<&str> = records.iter().map(|r| r.subject.as_str()).collect();
    let mut v: Vec<String> = set.into_iter().map(String::from).collect();
    v.sort();
    v
}

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
    println!(
        "passing: {passing} of {} (predicate called {calls} times)",
        raw.len()
    );

    println!("== ranking ==");
    for (i, (name, avg)) in ranking(&averages).iter().enumerate() {
        println!("{}. {name:<6} {avg:.2}", i + 1);
    }
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
