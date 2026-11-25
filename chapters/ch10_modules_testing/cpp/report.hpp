// Ch10 C++: report (Rust: report.rs)
#pragma once
#include <cstdint>
#include <map>
#include <string>
#include <utility>
#include <vector>

#include "model.hpp"

namespace ledger {

struct MonthTotal {
    std::int64_t income_cents = 0;
    std::int64_t expense_cents = 0;
    std::int64_t net() const { return income_cents - expense_cents; }
};

std::map<std::pair<int, int>, MonthTotal> monthly_totals(const std::vector<Transaction>& txs);
std::map<std::string, std::int64_t> expense_by_category(const std::vector<Transaction>& txs);

}  // namespace ledger
