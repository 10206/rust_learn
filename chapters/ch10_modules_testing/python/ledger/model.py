"""도메인 모델 (Rust: model.rs)."""
from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class Category(Enum):
    FOOD = "food"
    TRANSPORT = "transport"
    SALARY = "salary"
    RENT = "rent"
    OTHER = "other"

    @staticmethod
    def parse(s: str) -> Category | None:
        for c in Category:
            if c.value == s:
                return c
        return None

    def as_str(self) -> str:
        return self.value


@dataclass
class Transaction:
    year: int
    month: int
    day: int
    category: Category
    amount_cents: int  # 센트 단위 정수 (수입 +, 지출 -)
    description: str

    def is_income(self) -> bool:
        return self.amount_cents > 0

    def month_key(self) -> tuple[int, int]:
        # Rust 에서는 pub(crate). Python 에는 강제할 방법이 없어서 이름 규칙뿐입니다.
        return (self.year, self.month)


def format_cents(cents: int) -> str:
    sign = "-" if cents < 0 else ""
    a = abs(cents)
    return f"{sign}{a // 100}.{a % 100:02d}"
