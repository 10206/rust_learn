// Ch10 C++20 참조 구현: 가계부. model / parser / report 를 여러 .cpp 로 나눠 함께 컴파일합니다.
//   g++ -std=c++20 -O2 -Wall -Wextra cpp/*.cpp -o /tmp/x   (scripts/compare.sh 가 이렇게 해 줍니다)
// CMake 를 쓴다면 add_executable(ledger main.cpp model.cpp parser.cpp report.cpp) 가 필요해요.
// Rust 는 Cargo 가 mod 선언을 따라 파일을 알아서 찾아 줍니다.
#include <format>
#include <fstream>
#include <iostream>
#include <sstream>

#include "model.hpp"
#include "parser.hpp"
#include "report.hpp"

int main() {
    std::ifstream in("data/ledger.csv");
    std::stringstream buf;
    buf << in.rdbuf();
    const std::string text = buf.str();

    auto [txs, errors] = ledger::parse_ledger(text);
    std::cout << "loaded " << txs.size() << " transactions, " << errors.size() << " errors\n";
    for (const auto& e : errors) std::cout << e.to_string() << "\n";

    std::cout << "== monthly totals ==\n";
    for (const auto& [ym, total] : ledger::monthly_totals(txs)) {
        std::cout << std::format("{}-{:02}  income {:>9}  expense {:>9}  net {:>9}\n", ym.first, ym.second,
                                 ledger::format_cents(total.income_cents),
                                 ledger::format_cents(total.expense_cents),
                                 ledger::format_cents(total.net()));
    }

    std::cout << "== expenses by category ==\n";
    for (const auto& [category, cents] : ledger::expense_by_category(txs))
        std::cout << std::format("{:<10} {:>9}\n", category, ledger::format_cents(cents));
}
