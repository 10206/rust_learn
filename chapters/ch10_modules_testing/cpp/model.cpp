#include "model.hpp"

#include <cstdio>
#include <cstdlib>

namespace ledger {

std::optional<Category> parse_category(const std::string& s) {
    if (s == "food") return Category::Food;
    if (s == "transport") return Category::Transport;
    if (s == "salary") return Category::Salary;
    if (s == "rent") return Category::Rent;
    if (s == "other") return Category::Other;
    return std::nullopt;
}

std::string category_name(Category c) {
    switch (c) {
        case Category::Food: return "food";
        case Category::Transport: return "transport";
        case Category::Salary: return "salary";
        case Category::Rent: return "rent";
        case Category::Other: return "other";
    }
    return "other";
}

std::string format_cents(std::int64_t cents) {
    const char* sign = cents < 0 ? "-" : "";
    long long a = cents < 0 ? -static_cast<long long>(cents) : cents;
    char buf[64];
    std::snprintf(buf, sizeof buf, "%s%lld.%02lld", sign, a / 100, a % 100);
    return buf;
}

}  // namespace ledger
