"""Ch09 Python 참조 구현: 로그 파서.

Python 은 가비지 컬렉터(참조 카운팅)가 객체를 살려 두기 때문에 "원본보다 오래 사는 참조" 문제가 없어요.
대신 슬라이싱(line[a:b])은 항상 **새 str 을 복사**해 만듭니다. Rust 의 &str 슬라이스는 복사가 없습니다.
"""
from __future__ import annotations

from dataclasses import dataclass

LEVELS = ["DEBUG", "INFO", "WARN", "ERROR"]


@dataclass(frozen=True)
class LogEntry:
    level: str
    module: str
    message: str

    def __str__(self) -> str:  # Rust: impl Display for LogEntry
        return f"[{self.level}] {self.module}: {self.message}"


def parse_line(line: str) -> LogEntry | None:
    parts = line.split(" ", 3)
    if len(parts) < 4:
        return None
    _date, _time, level, rest = parts
    if level not in LEVELS:
        return None
    module, sep, message = rest.partition(": ")
    if not sep or not module or " " in module or not message:
        return None
    return LogEntry(level, module, message)


def parse_log(text: str) -> tuple[list[LogEntry], int]:
    entries: list[LogEntry] = []
    skipped = 0
    for line in text.splitlines():
        if not line.strip():
            continue
        entry = parse_line(line)
        if entry is None:
            skipped += 1
        else:
            entries.append(entry)
    return entries, skipped


def filter_by_level(entries: list[LogEntry], level: str) -> list[LogEntry]:
    return [e for e in entries if e.level == level]


def longest_message(entries: list[LogEntry]) -> str | None:
    if not entries:
        return None
    return max((e.message for e in entries), key=len)


def count_by_module(entries: list[LogEntry]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for e in entries:
        counts[e.module] = counts.get(e.module, 0) + 1
    return dict(sorted(counts.items()))  # Rust 의 BTreeMap 처럼 키 순서로 정렬


def count_by_level(entries: list[LogEntry]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for e in entries:
        counts[e.level] = counts.get(e.level, 0) + 1
    return dict(sorted(counts.items()))


def longer(a: str, b: str) -> str:
    return b if len(b) > len(a) else a


def main() -> None:
    with open("data/app.log", encoding="utf-8") as f:
        text = f.read()
    entries, skipped = parse_log(text)
    print(f"parsed {len(entries)} entries, skipped {skipped} malformed lines")

    for level in ["ERROR", "WARN"]:
        print(f"== {level} entries ==")
        for e in filter_by_level(entries, level):
            print(e)

    print("== longest message ==")
    m = longest_message(entries)
    if m is not None:
        print(f"{m} ({len(m.encode())} bytes)")

    print("== per module ==")
    for module, n in count_by_module(entries).items():
        print(f"{module}: {n}")

    print("== per level ==")
    for level, n in count_by_level(entries).items():
        print(f"{level}: {n}")

    print("== longer ==")
    print(f'longer("abc", "de") = {longer("abc", "de")}')


if __name__ == "__main__":
    main()
