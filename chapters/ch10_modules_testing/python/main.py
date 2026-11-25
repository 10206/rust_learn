"""Ch10 Python 참조 구현: 가계부 (패키지 구조 버전).

python/ledger/ 폴더가 패키지입니다. `python3 python/main.py` 로 실행하면 python/ 가 import 경로에 들어가요.
"""
from ledger import format_cents, expense_by_category, monthly_totals, parse_ledger


def main() -> None:
    with open("data/ledger.csv", encoding="utf-8") as f:
        text = f.read()
    txs, errors = parse_ledger(text)
    print(f"loaded {len(txs)} transactions, {len(errors)} errors")
    for e in errors:
        print(e)

    print("== monthly totals ==")
    for (year, month), total in monthly_totals(txs).items():
        print(
            f"{year}-{month:02d}  income {format_cents(total.income_cents):>9}"
            f"  expense {format_cents(total.expense_cents):>9}"
            f"  net {format_cents(total.net()):>9}"
        )

    print("== expenses by category ==")
    for category, cents in expense_by_category(txs).items():
        print(f"{category:<10} {format_cents(cents):>9}")


if __name__ == "__main__":
    main()
