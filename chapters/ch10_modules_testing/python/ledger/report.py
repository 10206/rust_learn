"""리포트 (Rust: report.rs)."""
from __future__ import annotations

from dataclasses import dataclass

from .model import Transaction


@dataclass
class MonthTotal:
    income_cents: int = 0
    expense_cents: int = 0

    def net(self) -> int:
        return self.income_cents - self.expense_cents


def monthly_totals(txs: list[Transaction]) -> dict[tuple[int, int], MonthTotal]:
    totals: dict[tuple[int, int], MonthTotal] = {}
    for t in txs:
        total = totals.setdefault(t.month_key(), MonthTotal())
        if t.is_income():
            total.income_cents += t.amount_cents
        else:
            total.expense_cents += -t.amount_cents
    return dict(sorted(totals.items()))  # Rust BTreeMap 처럼 키 순서


def expense_by_category(txs: list[Transaction]) -> dict[str, int]:
    out: dict[str, int] = {}
    for t in txs:
        if t.amount_cents < 0:
            name = t.category.as_str()
            out[name] = out.get(name, 0) - t.amount_cents
    return dict(sorted(out.items()))
