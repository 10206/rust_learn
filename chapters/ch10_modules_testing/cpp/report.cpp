#include "report.hpp"

namespace ledger {

std::map<std::pair<int, int>, MonthTotal> monthly_totals(const std::vector<Transaction>& txs) {
    std::map<std::pair<int, int>, MonthTotal> totals;  // std::map = 키 정렬 (Rust BTreeMap)
    for (const auto& t : txs) {
        auto& total = totals[{t.year, t.month}];
        if (t.is_income()) total.income_cents += t.amount_cents;
        else total.expense_cents += -t.amount_cents;
    }
    return totals;
}

std::map<std::string, std::int64_t> expense_by_category(const std::vector<Transaction>& txs) {
    std::map<std::string, std::int64_t> out;
    for (const auto& t : txs)
        if (t.amount_cents < 0) out[category_name(t.category)] += -t.amount_cents;
    return out;
}

}  // namespace ledger
