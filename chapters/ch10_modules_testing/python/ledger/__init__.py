"""가계부 패키지. (Rust 의 lib.rs 에 해당)

Python 은 폴더 + __init__.py 가 패키지입니다. Rust 는 `mod model;` 이라고 **선언해야** 파일이 모듈이 돼요.
Python 에는 pub/private 키워드가 없고 `_이름` 관례만 있습니다.
"""
from .model import Category, Transaction, format_cents
from .parser import ParseError, parse_ledger, parse_line
from .report import MonthTotal, expense_by_category, monthly_totals

# Rust 의 `pub use` (re-export) 와 비슷한 역할
__all__ = [
    "Category", "Transaction", "format_cents",
    "ParseError", "parse_ledger", "parse_line",
    "MonthTotal", "expense_by_category", "monthly_totals",
]
