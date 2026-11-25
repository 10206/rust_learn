// Ch11. 스마트 포인터 - C++20 참조 구현.
//   Box<T>           ~ std::unique_ptr<T>
//   Rc<T>            ~ std::shared_ptr<T>  (단, Rust 의 Rc 는 원자적(atomic) 카운트가 아님 -> 더 빠름)
//   Arc<T>           ~ std::shared_ptr<T>  (원자적 카운트)
//   Weak<T>          ~ std::weak_ptr<T>
//   RefCell<T>       ~ (대응 없음) 빌림 규칙을 "런타임"에 검사하는 상자
#include <algorithm>
#include <format>
#include <iostream>
#include <memory>
#include <string>
#include <vector>

// ───────────── Part A: 재귀 타입 Expr ─────────────
// 자기 자신을 값으로 담을 수 없으므로 unique_ptr (= Rust 의 Box) 로 간접 참조합니다.
// (Rust 의 enum Expr 에 해당하는 std::variant 버전도 가능하지만 여기서는 태그 구조체로 단순화)
struct Expr;
using ExprPtr = std::unique_ptr<Expr>;
struct Expr {
    enum class Kind { Num, Add, Mul, Neg } kind;
    double value = 0.0;
    ExprPtr a, b;
};

ExprPtr num(double v) { return std::make_unique<Expr>(Expr{Expr::Kind::Num, v, nullptr, nullptr}); }
ExprPtr add(ExprPtr l, ExprPtr r) { return std::make_unique<Expr>(Expr{Expr::Kind::Add, 0.0, std::move(l), std::move(r)}); }
ExprPtr mul(ExprPtr l, ExprPtr r) { return std::make_unique<Expr>(Expr{Expr::Kind::Mul, 0.0, std::move(l), std::move(r)}); }
ExprPtr neg(ExprPtr x) { return std::make_unique<Expr>(Expr{Expr::Kind::Neg, 0.0, std::move(x), nullptr}); }

double eval(const Expr& e) {
    switch (e.kind) {
        case Expr::Kind::Num: return e.value;
        case Expr::Kind::Add: return eval(*e.a) + eval(*e.b);
        case Expr::Kind::Mul: return eval(*e.a) * eval(*e.b);
        case Expr::Kind::Neg: return -eval(*e.a);
    }
    return 0.0;
}

std::string show(const Expr& e) {
    switch (e.kind) {
        case Expr::Kind::Num: return std::format("{:.1f}", e.value);
        case Expr::Kind::Add: return std::format("({} + {})", show(*e.a), show(*e.b));
        case Expr::Kind::Mul: return std::format("({} * {})", show(*e.a), show(*e.b));
        case Expr::Kind::Neg: return "-" + show(*e.a);
    }
    return "";
}

int depth(const Expr& e) {
    switch (e.kind) {
        case Expr::Kind::Num: return 1;
        case Expr::Kind::Add:
        case Expr::Kind::Mul: return 1 + std::max(depth(*e.a), depth(*e.b));
        case Expr::Kind::Neg: return 1 + depth(*e.a);
    }
    return 0;
}

bool is_num(const ExprPtr& e, double v) { return e->kind == Expr::Kind::Num && e->value == v; }

// Rust 에서는 Box 를 값으로 받아(소유권 이동) 분해합니다. 여기서는 unique_ptr 를 move 로 받습니다.
ExprPtr simplify(ExprPtr e) {
    switch (e->kind) {
        case Expr::Kind::Num: return e;
        case Expr::Kind::Add: {
            auto l = simplify(std::move(e->a));
            auto r = simplify(std::move(e->b));
            if (is_num(l, 0.0)) return r;
            if (is_num(r, 0.0)) return l;
            return add(std::move(l), std::move(r));
        }
        case Expr::Kind::Mul: {
            auto l = simplify(std::move(e->a));
            auto r = simplify(std::move(e->b));
            if (is_num(l, 0.0) || is_num(r, 0.0)) return num(0.0);
            if (is_num(l, 1.0)) return r;
            if (is_num(r, 1.0)) return l;
            return mul(std::move(l), std::move(r));
        }
        case Expr::Kind::Neg: {
            auto x = simplify(std::move(e->a));
            if (x->kind == Expr::Kind::Neg) return std::move(x->a);
            return neg(std::move(x));
        }
    }
    return e;
}

// ───────────── Part B: 대화 분기 트리 ─────────────
struct Node;
using NodePtr = std::shared_ptr<Node>;  // Rust: Rc<RefCell<Node>>
struct Node {
    std::string text;
    std::weak_ptr<Node> parent;  // Rust: Weak<RefCell<Node>>
    std::vector<NodePtr> children;
    int visits = 0;
};

NodePtr new_node(const std::string& text) {
    auto n = std::make_shared<Node>();
    n->text = text;
    return n;
}

NodePtr add_child(const NodePtr& parent, const std::string& text) {
    auto child = new_node(text);
    child->parent = parent;
    parent->children.push_back(child);
    return child;
}

std::string path_to_root(const NodePtr& node) {
    std::vector<std::string> parts{node->text};
    for (auto cur = node->parent.lock(); cur; cur = cur->parent.lock()) parts.push_back(cur->text);
    std::string out;
    for (auto it = parts.rbegin(); it != parts.rend(); ++it) {
        if (!out.empty()) out += " > ";
        out += *it;
    }
    return out;
}

void collect_leaf_paths(const NodePtr& node, std::vector<std::string>& out) {
    if (node->children.empty()) {
        out.push_back(path_to_root(node));
        return;
    }
    for (const auto& c : node->children) collect_leaf_paths(c, out);
}

std::vector<std::string> leaf_paths(const NodePtr& root) {
    std::vector<std::string> out;
    collect_leaf_paths(root, out);
    return out;
}

void visit(const NodePtr& node) { node->visits += 1; }

int total_visits(const NodePtr& root) {
    int sum = root->visits;
    for (const auto& c : root->children) sum += total_visits(c);
    return sum;
}

// ───────────── 순환 참조 실험 ─────────────
struct Link {
    std::string name;
    std::shared_ptr<Link> other;  // 강한 참조 -> 순환 시 누수
};
struct WeakLink {
    std::string name;
    std::shared_ptr<WeakLink> strong_other;
    std::weak_ptr<WeakLink> weak_other;
};

bool demo_cycle_leak() {
    auto a = std::make_shared<Link>(Link{"a", nullptr});
    auto b = std::make_shared<Link>(Link{"b", nullptr});
    a->other = b;
    b->other = a;
    std::cout << std::format("cycle: a strong={} b strong={}\n", a.use_count(), b.use_count());
    std::weak_ptr<Link> probe = a;
    a.reset();
    b.reset();
    return !probe.expired();
}

bool demo_weak_fixed() {
    auto a = std::make_shared<WeakLink>();
    auto b = std::make_shared<WeakLink>();
    a->strong_other = b;
    b->weak_other = a;
    std::cout << std::format("fixed: a strong={} b strong={}\n", a.use_count(), b.use_count());
    std::weak_ptr<WeakLink> probe = a;
    a.reset();
    b.reset();
    return !probe.expired();
}

int main() {
    std::cout << "== Part A: Expr ==\n";
    auto expr = mul(add(num(2.0), num(3.0)), neg(add(num(4.0), num(1.5))));
    std::cout << "expr: " << show(*expr) << "\n";
    std::cout << std::format("value: {:.2f}\n", eval(*expr));
    std::cout << "depth: " << depth(*expr) << "\n";
    auto messy = add(mul(num(1.0), neg(neg(num(7.0)))), num(0.0));
    std::cout << "messy: " << show(*messy) << "\n";
    auto simple = simplify(std::move(messy));  // messy 는 이제 비어 있음 (moved-from)
    std::cout << "simplified: " << show(*simple) << " (depth " << depth(*simple) << ")\n";

    std::cout << "== Part B: dialogue tree ==\n";
    auto root = new_node("Hello");
    auto order = add_child(root, "Order status");
    auto track = add_child(order, "Track package");
    add_child(order, "Cancel order");
    auto refund = add_child(root, "Refund");
    add_child(refund, "Refund policy");
    add_child(root, "Chat");
    std::cout << "paths:\n";
    for (const auto& p : leaf_paths(root)) std::cout << "  " << p << "\n";
    std::cout << "path_to_root(track) = " << path_to_root(track) << "\n";
    std::cout << std::format("strong_count(root)={} strong_count(order)={}\n", root.use_count(), order.use_count());
    visit(track);
    visit(track);
    visit(refund);
    visit(root);
    std::cout << "total visits: " << total_visits(root) << "\n";

    std::cout << "== Cycle experiment ==\n";
    const bool cycle_leaked = demo_cycle_leak();  // 함수 안의 출력이 먼저 나오도록 먼저 호출
    std::cout << "cycle leaked: " << (cycle_leaked ? "yes" : "no") << "\n";
    const bool fixed_leaked = demo_weak_fixed();
    std::cout << "fixed leaked: " << (fixed_leaked ? "yes" : "no") << "\n";
}
