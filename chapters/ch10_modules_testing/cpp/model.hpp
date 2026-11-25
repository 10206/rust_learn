// Ch10 C++: model (Rust: model.rs)
// C++ 은 "선언(.hpp) + 정의(.cpp)" 를 직접 나눠 적고, include guard 도 챙겨야 해요.
// Rust 는 선언/정의 분리가 없고 `mod model;` 한 줄이면 됩니다.
#pragma once
#include <cstdint>
#include <optional>
#include <string>

namespace ledger {  // Rust 의 모듈 = C++ 의 namespace 와 비슷 (하지만 Rust 는 파일 구조와 1:1)

enum class Category { Food, Transport, Salary, Rent, Other };

std::optional<Category> parse_category(const std::string& s);
std::string category_name(Category c);

struct Transaction {
    int year;
    int month;
    int day;
    Category category;
    std::int64_t amount_cents;  // 수입 +, 지출 -
    std::string description;

    bool is_income() const { return amount_cents > 0; }
};

std::string format_cents(std::int64_t cents);

}  // namespace ledger
