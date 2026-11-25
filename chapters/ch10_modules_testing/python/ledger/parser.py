"""CSV -> Transaction (Rust: parser.rs)."""
from __future__ import annotations

from dataclasses import dataclass

from .model import Category, Transaction


@dataclass
class ParseError(Exception):
    """Rust 는 enum 4개 변형, Python 은 kind 문자열 하나로 단순화했어요."""
    kind: str
    line: int
    text: str = ""
    found: int = 0

    def __str__(self) -> str:
        if self.kind == "field_count":
            return f"line {self.line}: expected 4 fields, found {self.found}"
        if self.kind == "bad_date":
            return f"line {self.line}: invalid date '{self.text}'"
        if self.kind == "unknown_category":
            return f"line {self.line}: unknown category '{self.text}'"
        return f"line {self.line}: invalid amount '{self.text}'"


def _all_digits(s: str) -> bool:
    return s.isascii() and s.isdigit()


def _parse_date(s: str) -> tuple[int, int, int] | None:
    parts = s.split("-")
    if len(parts) != 3 or len(parts[0]) != 4 or not all(_all_digits(p) for p in parts):
        return None
    y, m, d = (int(p) for p in parts)
    if 1 <= m <= 12 and 1 <= d <= 31:
        return y, m, d
    return None


def _parse_int(s: str) -> int | None:
    body = s[1:] if s.startswith("-") else s
    return int(s) if _all_digits(body) else None


def parse_line(line_no: int, line: str) -> Transaction:
    """실패하면 ParseError 를 던집니다 (Rust 는 Result 로 돌려줘요)."""
    fields = line.split(",")
    if len(fields) != 4:
        raise ParseError("field_count", line_no, found=len(fields))
    date = _parse_date(fields[0])
    if date is None:
        raise ParseError("bad_date", line_no, text=fields[0])
    category = Category.parse(fields[1])
    if category is None:
        raise ParseError("unknown_category", line_no, text=fields[1])
    amount = _parse_int(fields[2])
    if amount is None:
        raise ParseError("bad_amount", line_no, text=fields[2])
    return Transaction(date[0], date[1], date[2], category, amount, fields[3])


def parse_ledger(text: str) -> tuple[list[Transaction], list[ParseError]]:
    ok: list[Transaction] = []
    errors: list[ParseError] = []
    for i, line in enumerate(text.splitlines()):
        if not line.strip() or line.startswith("date,"):
            continue
        try:
            ok.append(parse_line(i + 1, line))
        except ParseError as e:
            errors.append(e)
    return ok, errors
