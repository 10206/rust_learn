// Ch05 자판기 상태 머신 (C++20 참조 구현).
// Rust 의 enum  <->  C++ 의 std::variant + std::visit.
// 차이: variant 는 별도 struct 를 일일이 선언해야 하고, visit 에 "일반 람다 fallback" 을 두면
//       빠뜨린 조합도 조용히 통과해 버립니다. Rust 의 match 는 빠뜨리면 컴파일 에러입니다.
#include <algorithm>
#include <cctype>
#include <format>
#include <iostream>
#include <optional>
#include <sstream>
#include <string>
#include <variant>
#include <vector>

enum class Item { Cola, Chips, Candy };

int price(Item i) {
    switch (i) {  // enum class + switch: default 를 빼면 -Wswitch 가 경고
        case Item::Cola: return 150;
        case Item::Chips: return 100;
        case Item::Candy: return 70;
    }
    return 0;
}

std::string item_name(Item i) {
    switch (i) {
        case Item::Cola: return "Cola";
        case Item::Chips: return "Chips";
        case Item::Candy: return "Candy";
    }
    return "";
}

std::string lower(std::string s) {
    std::transform(s.begin(), s.end(), s.begin(), [](unsigned char c) { return std::tolower(c); });
    return s;
}

std::optional<Item> item_from_name(const std::string& s) {
    std::string l = lower(s);
    if (l == "cola") return Item::Cola;
    if (l == "chips") return Item::Chips;
    if (l == "candy") return Item::Candy;
    return std::nullopt;
}

// ── Event ──
struct InsertCoin { int cents; };
struct Select { Item item; };
struct Cancel {};
struct Take {};
struct Kick {};
struct Repair {};
using Event = std::variant<InsertCoin, Select, Cancel, Take, Kick, Repair>;

// ── State ──
struct Idle {};
struct Collecting { int cents; };
struct Dispensing { Item item; int change; };
struct Refunding { int cents; };
struct OutOfService { std::string reason; };
using State = std::variant<Idle, Collecting, Dispensing, Refunding, OutOfService>;

// std::visit 용 헬퍼: 람다 여러 개를 하나의 호출 객체로 묶습니다 (C++ 의 관용구).
template <class... Ts> struct overloaded : Ts... { using Ts::operator()...; };

const std::vector<std::string> SCRIPT = {
    "coin 100", "coin 7", "coin 100", "select cola", "take", "select chips",
    "coin 500", "select candy", "take", "coin 50", "cancel", "take", "dance",
    "coin 100", "kick", "coin 100", "repair", "coin 10", "select chips",
};

std::string event_str(const Event& e) {
    return std::visit(overloaded{
        [](const InsertCoin& x) { return std::format("InsertCoin({})", x.cents); },
        [](const Select& x) { return std::format("Select({})", item_name(x.item)); },
        [](const Cancel&) { return std::string("Cancel"); },
        [](const Take&) { return std::string("Take"); },
        [](const Kick&) { return std::string("Kick"); },
        [](const Repair&) { return std::string("Repair"); },
    }, e);
}

bool is_valid_coin(int c) { return c == 10 || c == 50 || c == 100 || c == 500; }

std::optional<Event> parse_event(const std::string& line) {
    std::istringstream in(line);
    std::vector<std::string> parts;
    for (std::string w; in >> w;) parts.push_back(w);
    if (parts.empty()) return std::nullopt;
    const std::string& cmd = parts[0];
    if (cmd == "coin" && parts.size() == 2) {
        const std::string& a = parts[1];
        if (!std::all_of(a.begin(), a.end(), [](unsigned char c) { return std::isdigit(c); })) return std::nullopt;
        return Event{InsertCoin{std::stoi(a)}};
    }
    if (cmd == "select" && parts.size() == 2) {
        auto item = item_from_name(parts[1]);
        if (!item) return std::nullopt;
        return Event{Select{*item}};
    }
    if (parts.size() != 1) return std::nullopt;
    if (cmd == "cancel") return Event{Cancel{}};
    if (cmd == "take") return Event{Take{}};
    if (cmd == "kick") return Event{Kick{}};
    if (cmd == "repair") return Event{Repair{}};
    return std::nullopt;
}

std::string state_str(const State& s) {
    return std::visit(overloaded{
        [](const Idle&) { return std::string("Idle"); },
        [](const Collecting& x) { return std::format("Collecting({})", x.cents); },
        [](const Dispensing& x) { return std::format("Dispensing({}, change={})", item_name(x.item), x.change); },
        [](const Refunding& x) { return std::format("Refunding({})", x.cents); },
        [](const OutOfService& x) { return std::format("OutOfService({})", x.reason); },
    }, s);
}

// (state, event) 두 variant 를 동시에 visit. 마지막 템플릿 람다가 "나머지 전부" 입니다.
State transition(const State& state, const Event& event) {
    return std::visit(overloaded{
        [](const OutOfService&, const Repair&) -> State { return Idle{}; },
        [](const OutOfService& s, const Kick&) -> State { return s; },
        [](const auto&, const Kick&) -> State { return OutOfService{"kicked"}; },
        [](const Idle&, const InsertCoin& c) -> State {
            return is_valid_coin(c.cents) ? State{Collecting{c.cents}} : State{Idle{}};
        },
        [](const Collecting& s, const InsertCoin& c) -> State {
            return is_valid_coin(c.cents) ? State{Collecting{s.cents + c.cents}} : State{s};
        },
        [](const Collecting& s, const Select& sel) -> State {
            if (s.cents >= price(sel.item)) return Dispensing{sel.item, s.cents - price(sel.item)};
            return s;
        },
        [](const Collecting& s, const Cancel&) -> State { return Refunding{s.cents}; },
        [](const Dispensing&, const Take&) -> State { return Idle{}; },
        [](const Refunding&, const Take&) -> State { return Idle{}; },
        [](const auto& s, const auto&) -> State { return s; },
    }, state, event);
}

std::optional<Item> sale_of(const State& state, const Event& event) {
    const auto* d = std::get_if<Dispensing>(&state);
    if (d && std::holds_alternative<Take>(event)) return d->item;
    return std::nullopt;
}

class Machine {
public:
    State state = Idle{};
    std::vector<Item> sold;

    void handle(const Event& event) {
        if (auto item = sale_of(state, event)) sold.push_back(*item);
        state = transition(state, event);
    }
    int revenue() const {
        int sum = 0;
        for (Item i : sold) sum += price(i);
        return sum;
    }
    // Rust 의 `fn into_sold(self)` 는 객체를 소비합니다. C++ 에서는 std::move 로 비슷하게 흉내 냅니다.
    std::vector<Item> into_sold() { return std::move(sold); }
};

int main() {
    Machine machine;
    int i = 0;
    for (const auto& line : SCRIPT) {
        ++i;
        std::string label;
        if (auto event = parse_event(line)) {
            machine.handle(*event);
            label = event_str(*event);
        } else {
            label = "? " + line;
        }
        std::cout << std::format("step {:02}: {:<16} -> {}\n", i, label, state_str(machine.state));
    }
    std::cout << "revenue: " << machine.revenue() << "\n";
    auto sold = machine.into_sold();
    std::string out = "sold: ";
    for (size_t k = 0; k < sold.size(); ++k) out += (k ? ", " : "") + item_name(sold[k]);
    std::cout << out << "\n";
}
