"""Ch07 성적 데이터 분석 (Python 참조 구현).

Python 은 컴프리헨션/제너레이터가 Rust 의 이터레이터 체인과 비슷합니다.
차이: Rust 의 체인은 컴파일 때 하나의 루프로 합쳐지고(zero-cost), 타입이 모두 검사됩니다.
"""

from __future__ import annotations

from collections import Counter, defaultdict
from dataclasses import dataclass
from typing import Callable


@dataclass
class Record:
    student: str
    subject: str
    score: int


def parse_record(line: str) -> Record | None:
    parts = [p.strip() for p in line.split(",")]
    if len(parts) != 3:
        return None
    student, subject, score = parts
    if not student or not subject or not (score.isascii() and score.isdigit()):
        return None
    return Record(student, subject, int(score))


def parse_records(text: str) -> tuple[list[Record], int]:
    lines = [l for l in text.splitlines()[1:] if l.strip()]
    records = [r for l in lines if (r := parse_record(l)) is not None]  # Rust: filter_map
    return records, len(lines) - len(records)


def student_averages(records: list[Record]) -> dict[str, float]:
    scores: defaultdict[str, list[int]] = defaultdict(list)  # Rust: entry().or_default()
    for r in records:
        scores[r.student].append(r.score)
    return {name: sum(v) / len(v) for name, v in sorted(scores.items())}


def subject_tops(records: list[Record]) -> dict[str, tuple[str, int]]:
    tops: dict[str, tuple[str, int]] = {}
    for r in records:
        best = tops.get(r.subject)
        if best is None or r.score > best[1]:
            tops[r.subject] = (r.student, r.score)
    return dict(sorted(tops.items()))


def grade_of(score: int) -> str:
    if score >= 90:
        return "A"
    if score >= 80:
        return "B"
    if score >= 70:
        return "C"
    if score >= 60:
        return "D"
    return "F"


def grade_histogram(scores: list[int]) -> Counter[str]:
    return Counter(grade_of(s) for s in scores)


def make_curve(bonus: int) -> Callable[[int], int]:
    # 클로저가 bonus 를 "참조로" 캡처합니다. Rust 는 `move` 로 값을 가져갈지 빌릴지 명시합니다.
    return lambda s: min(s + bonus, 100)


def apply_in_place(scores: list[int], f: Callable[[int], int]) -> None:
    for i, s in enumerate(scores):
        scores[i] = f(s)


def count_passing(scores: list[int]) -> tuple[int, int]:
    calls = 0

    def passes(s: int) -> bool:
        nonlocal calls  # Rust: 클로저가 calls 를 수정 -> FnMut
        calls += 1
        return s >= 60

    passing = sum(1 for s in scores if passes(s))
    return passing, calls


def ranking(averages: dict[str, float]) -> list[tuple[str, float]]:
    return sorted(averages.items(), key=lambda kv: (-kv[1], kv[0]))


def distinct_subjects(records: list[Record]) -> list[str]:
    return sorted({r.subject for r in records})


def print_histogram(title: str, scores: list[int]) -> None:
    print(title)
    hist = grade_histogram(scores)
    for g in "ABCDF":
        n = hist.get(g, 0)
        print(f"{g}: {'#' * n} ({n})")


def main() -> None:
    with open("data/scores.csv", encoding="utf-8") as f:
        text = f.read()
    records, skipped = parse_records(text)
    print(f"records: {len(records)} (skipped {skipped})")
    print("subjects: " + ", ".join(distinct_subjects(records)))

    print("== average per student ==")
    averages = student_averages(records)
    for name, avg in averages.items():
        print(f"{name:<6}: {avg:.2f}")

    print("== top per subject ==")
    for subject, (name, score) in subject_tops(records).items():
        print(f"{subject:<8}: {name} ({score})")

    scores = [r.score for r in records]
    print_histogram("== grade histogram ==", scores)

    curve = make_curve(5)
    apply_in_place(scores, curve)
    print_histogram("== after +5 curve ==", scores)

    raw = [r.score for r in records]
    passing, calls = count_passing(raw)
    print("== passing ==")
    print(f"passing: {passing} of {len(raw)} (predicate called {calls} times)")

    print("== ranking ==")
    for i, (name, avg) in enumerate(ranking(averages), start=1):
        print(f"{i}. {name:<6} {avg:.2f}")


if __name__ == "__main__":
    main()
