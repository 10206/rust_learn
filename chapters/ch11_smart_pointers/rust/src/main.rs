//! Ch11. 스마트 포인터 · 내부 가변성 — 실습
//!
//! 목표
//!   - `Box<T>` 로 재귀 타입(enum Expr)을 만들고 소유권째로 분해해 봅니다.
//!   - `Rc<RefCell<T>>` + `Weak` 로 부모/자식이 서로를 가리키는 트리를 만듭니다.
//!   - 강한 참조 순환이 왜 메모리 누수인지 `strong_count` 로 직접 확인합니다.
//!
//! 대응 파일: `../python/main.py`, `../cpp/main.cpp`
//!
//! 진행 방법
//!   1. TODO(1)부터 순서대로 `todo!()` 를 지우고 구현하세요.
//!   2. 🔬 실험 블록은 주석을 풀어서 컴파일러/런타임이 어떻게 반응하는지 보세요.
//!   3. 중간중간 `cargo test -p ch11_smart_pointers` 로 확인하세요.
//!
//! 완료 기준
//!   - `cargo test -p ch11_smart_pointers` 가 모두 통과
//!   - `../../scripts/compare.sh ch11` 에서 Python / C++ / Rust 출력이 똑같음

// 다 풀고 나면 이 줄을 지워 보세요 (그러면 안 쓰는 코드에 대한 경고가 보입니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// ───────────── Part A: 재귀 enum + Box ─────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

// 🔬 실험 1 (E0072): 위 enum 의 `Box<Expr>` 를 그냥 `Expr` 로 바꿔 보세요.
//   예) Add(Expr, Expr),
//   기대 에러: error[E0072]: recursive type `Expr` has infinite size
//   ❓ C++ 에서 `struct Node { Node next; };` 가 안 되는 것과 같은 이유입니다. 컴파일러가 제안하는
//      해결책(`Box<...>` 로 간접 참조)을 읽어 보세요. Box 는 크기가 항상 포인터 하나입니다.

// Box::new 를 매번 쓰기 번거로우니 생성 도우미를 둡니다. (이미 구현되어 있어요)
pub fn num(v: f64) -> Expr {
    Expr::Num(v)
}
pub fn add(l: Expr, r: Expr) -> Expr {
    Expr::Add(Box::new(l), Box::new(r))
}
pub fn mul(l: Expr, r: Expr) -> Expr {
    Expr::Mul(Box::new(l), Box::new(r))
}
pub fn neg(x: Expr) -> Expr {
    Expr::Neg(Box::new(x))
}

// ─────────────────────────────────────────────
// TODO(1): 식을 계산하세요.
//   🐍 Python: match e: case Add(l, r): return eval_expr(l) + eval_expr(r)   (python/main.py: eval_expr)
//   ⚙️ C++  : switch (e.kind) { case Add: return eval(*e.a) + eval(*e.b); }   (cpp/main.cpp: eval)
//   🦀 Rust : `match e { Expr::Add(l, r) => eval(l) + eval(r), .. }`
//   💡 힌트: `l` 의 타입은 `&Box<Expr>` 인데 `eval(&Expr)` 에 그대로 넘겨도 됩니다. (Deref 강제 변환)
//   ❓ 왜?: Box<T> 는 Deref<Target = T> 를 구현해서 `&Box<T>` 가 필요한 곳에서 `&T` 로 자동 변환됩니다.
// ─────────────────────────────────────────────
pub fn eval(e: &Expr) -> f64 {
    todo!("TODO(1): eval")
}

// ─────────────────────────────────────────────
// TODO(2): 사람이 읽기 좋은 문자열로 만드세요.
//   형식: Num -> "2.0" (소수 1자리), Add -> "(a + b)", Mul -> "(a * b)", Neg -> "-a"
//   🐍 Python: f"({show(l)} + {show(r)})"                     (python/main.py: show)
//   ⚙️ C++  : std::format("({} + {})", show(*e.a), show(*e.b)) (cpp/main.cpp: show)
//   🦀 Rust : format!("({} + {})", show(l), show(r))
//   💡 힌트: 소수 1자리는 `{:.1}` 또는 `{v:.1}`
// ─────────────────────────────────────────────
pub fn show(e: &Expr) -> String {
    todo!("TODO(2): show")
}

// ─────────────────────────────────────────────
// TODO(3): 트리의 깊이를 구하세요. (Num 하나 = 1)
//   🐍 Python: 1 + max(depth(l), depth(r))                    (python/main.py: depth)
//   ⚙️ C++  : 1 + std::max(depth(*e.a), depth(*e.b))           (cpp/main.cpp: depth)
//   🦀 Rust : `Expr::Add(l, r) | Expr::Mul(l, r) => ...` 처럼 패턴을 `|` 로 묶을 수 있어요.
//   💡 힌트: `a.max(b)` 는 usize 의 메서드입니다.
// ─────────────────────────────────────────────
pub fn depth(e: &Expr) -> usize {
    todo!("TODO(3): depth")
}

// ─────────────────────────────────────────────
// TODO(4): 식을 정리(simplify)해서 새 Expr 를 돌려주세요. (소유권을 받습니다: `e: Expr`)
//   규칙 (자식을 먼저 simplify 한 뒤 적용):
//     Add(0, x) / Add(x, 0) -> x
//     Mul(0, _) / Mul(_, 0) -> Num(0)
//     Mul(1, x) / Mul(x, 1) -> x
//     Neg(Neg(x))           -> x
//   🐍 Python: if l == Num(0.0): return r                     (python/main.py: simplify)
//   ⚙️ C++  : if (is_num(l, 0.0)) return r;  // unique_ptr 를 move  (cpp/main.cpp: simplify)
//   🦀 Rust : `match (simplify(*l), simplify(*r)) { (Expr::Num(z), x) if z == 0.0 => x, .. }`
//   💡 힌트: `*l` 은 `Box<Expr>` 안의 값을 밖으로 "꺼내는" 문법입니다. (Box 에서만 가능한 move)
//           `(Expr::Num(z), x) | (x, Expr::Num(z)) if z == 0.0 => x` 처럼 or-패턴도 쓸 수 있어요.
//   ❓ 왜?: Box 는 단독 소유자라서 안의 값을 move 해 나와도 안전합니다. (Rc 안의 값은 못 꺼냅니다)
// ─────────────────────────────────────────────
pub fn simplify(e: Expr) -> Expr {
    todo!("TODO(4): simplify")
}

// ───────────── Part B: Rc<RefCell<Node>> + Weak ─────────────

pub type NodeRef = Rc<RefCell<Node>>;

#[derive(Debug)]
pub struct Node {
    pub text: String,
    /// 부모는 Weak: 부모 <-> 자식이 서로 강하게 잡으면 영원히 해제되지 않습니다.
    pub parent: Weak<RefCell<Node>>,
    pub children: Vec<NodeRef>,
    pub visits: u32,
}

/// 새 노드를 만들어 `Rc<RefCell<..>>` 로 감싸서 돌려줍니다. (이미 구현되어 있어요)
pub fn new_node(text: &str) -> NodeRef {
    Rc::new(RefCell::new(Node {
        text: text.to_string(),
        parent: Weak::new(),
        children: Vec::new(),
        visits: 0,
    }))
}

// ─────────────────────────────────────────────
// TODO(5): 자식 노드를 만들어 부모에 연결하고, 자식 핸들을 돌려주세요.
//   🐍 Python: child.parent = weakref.ref(parent); parent.children.append(child)  (python/main.py: add_child)
//   ⚙️ C++  : child->parent = parent;  parent->children.push_back(child);          (cpp/main.cpp: add_child)
//   🦀 Rust : 자식 -> 부모는 `Rc::downgrade(parent)` (Weak), 부모 -> 자식은 `Rc::clone(&child)` (강한 참조)
//   💡 힌트: `child.borrow_mut().parent = ...;`  `parent.borrow_mut().children.push(...)`
//           `Rc::clone(&x)` 는 데이터를 복사하지 않고 카운트만 1 올립니다.
//   ❓ 왜?: 자식이 부모를 강하게 잡으면 부모 <-> 자식 순환이 생겨 트리 전체가 영원히 안 지워집니다.
// ─────────────────────────────────────────────
pub fn add_child(parent: &NodeRef, text: &str) -> NodeRef {
    todo!("TODO(5): add_child")
}

// ─────────────────────────────────────────────
// TODO(6): 루트부터 이 노드까지의 경로를 " > " 로 이어서 돌려주세요.
//   예) "Hello > Order status > Track package"
//   🐍 Python: while cur is not None: parts.append(cur.text); cur = cur.parent()   (python/main.py: path_to_root)
//   ⚙️ C++  : for (auto cur = node->parent.lock(); cur; cur = cur->parent.lock())   (cpp/main.cpp: path_to_root)
//   🦀 Rust : `node.borrow().parent.upgrade()` 는 `Option<Rc<..>>` (부모가 이미 죽었으면 None)
//   💡 힌트: 자식 -> 부모 방향으로 올라가며 모은 뒤 `parts.reverse()`, `parts.join(" > ")`
//           `while let Some(p) = cur { ... }`
//   ❓ 왜?: Weak 는 "있으면 쓰고, 없으면 말고" 입니다. upgrade() 가 Option 을 돌려주는 이유예요.
// ─────────────────────────────────────────────
pub fn path_to_root(node: &NodeRef) -> String {
    todo!("TODO(6): path_to_root")
}

// ─────────────────────────────────────────────
// TODO(7): 루트에서 모든 잎(leaf)까지의 경로를 DFS 순서(자식 추가 순서)로 돌려주세요.
//   🐍 Python: 자식이 없으면 [path_to_root(root)], 아니면 재귀                (python/main.py: leaf_paths)
//   ⚙️ C++  : collect_leaf_paths(...) 재귀                                      (cpp/main.cpp: leaf_paths)
//   🦀 Rust : `node.borrow().children.iter().flat_map(leaf_paths).collect()`
//   💡 힌트: 재귀 호출 안에서 같은 노드를 `borrow_mut()` 하면 안 됩니다. 읽기 `borrow()` 는 여러 개 OK.
// ─────────────────────────────────────────────
pub fn leaf_paths(root: &NodeRef) -> Vec<String> {
    todo!("TODO(7): leaf_paths")
}

// ─────────────────────────────────────────────
// TODO(8): visit: 노드의 visits 를 1 올리고, total_visits: 서브트리 전체 합을 구하세요.
//   🐍 Python: node.visits += 1                                            (python/main.py: visit)
//   ⚙️ C++  : node->visits += 1;  (shared_ptr<Node> 는 const 여도 안의 Node 는 수정 가능) (cpp/main.cpp: visit)
//   🦀 Rust : 인자가 `&NodeRef` (공유 참조) 인데도 `node.borrow_mut().visits += 1` 로 수정합니다. = 내부 가변성
//   ❓ 왜?: Rc 는 공유 소유이므로 `&mut` 를 줄 수 없어요. 그래서 RefCell 이 "런타임 빌림 검사" 로 대신합니다.
// ─────────────────────────────────────────────
pub fn visit(node: &NodeRef) {
    todo!("TODO(8): visit")
}

pub fn total_visits(root: &NodeRef) -> u32 {
    todo!("TODO(8): total_visits")
}

// ─────────────────────────────────────────────
// TODO(9): RefCell 은 빌림 규칙을 "런타임"에 검사합니다.
//   읽기용 `borrow()` 를 쥐고 있는 동안 `try_borrow_mut()` 가 성공하는지(true/false) 돌려주세요.
//   (`borrow_mut()` 는 실패 시 panic, `try_borrow_mut()` 는 Result 를 돌려줍니다.)
//   🐍 Python: (해당 없음 - 아무 때나 수정 가능)
//   ⚙️ C++  : (해당 없음 - 규칙 위반은 그냥 UB)
//   🦀 Rust : `let _reader = cell.borrow();  cell.try_borrow_mut().is_ok()`
//   💡 힌트: 컴파일은 되지만 실행 중에 걸립니다. C++ 에서 컴파일러가 안 잡는 오류를 Rust 는 "런타임에 확실히" 잡는 셈.
//
//   🔬 실험 2 (런타임 panic): 아래 함수의 주석을 풀고 main() 에서 호출해 보세요.
//   fn lab_refcell_panic() {
//       let cell = RefCell::new(vec![1, 2, 3]);
//       let first = cell.borrow_mut();
//       let second = cell.borrow_mut();   // <- 컴파일은 통과! 실행하면 panic
//       println!("{first:?} {second:?}");
//   }
//   기대 결과: thread 'main' panicked ... `RefCell already borrowed`
//   ❓ 같은 코드를 일반 `let mut v = vec![..]; let a = &mut v; let b = &mut v;` 로 쓰면 어떤 에러(E0499)가
//      나는지 비교해 보세요. RefCell 은 "컴파일 타임 검사를 런타임으로 미룬 것" 입니다.
// ─────────────────────────────────────────────
pub fn try_borrow_while_reading() -> bool {
    todo!("TODO(9): try_borrow_while_reading")
}

// 🔬 실험 3 (E0596): Rc 안의 값은 RefCell 없이는 못 바꿉니다. 주석을 풀어 보세요.
//   fn lab_rc_mutation() {
//       let shared = Rc::new(vec![1, 2, 3]);
//       shared.push(4);
//   }
//   기대 에러: error[E0596]: cannot borrow data in an `Rc` as mutable
//   ❓ 왜 `Rc<Vec<i32>>` 는 읽기 전용일까요? (힌트: 같은 Vec 을 여러 소유자가 동시에 쥐고 있어요)

// ───────────── 순환 참조 실험 ─────────────

pub struct Link {
    pub name: &'static str,
    pub other: RefCell<Option<Rc<Link>>>,
}

pub struct WeakLink {
    pub name: &'static str,
    pub strong_other: RefCell<Option<Rc<WeakLink>>>,
    pub weak_other: RefCell<Weak<WeakLink>>,
}

// ─────────────────────────────────────────────
// TODO(10): 강한 참조 순환(a -> b -> a)을 만들고, 변수를 버린 뒤에도 살아 있는지(= 누수) 돌려주세요.
//   절차:
//     1) a, b 를 `Rc::new(Link { name: "a", other: RefCell::new(None) })` 로 만든다.
//     2) `*a.other.borrow_mut() = Some(Rc::clone(&b));` 와 반대 방향도.
//     3) 다음 형식으로 출력:  println!("cycle: {} strong={} {} strong={}", a.name, Rc::strong_count(&a), b.name, Rc::strong_count(&b));
//        (기대: `cycle: a strong=2 b strong=2`)
//     4) `let probe = Rc::downgrade(&a);` 로 약한 감시자를 만든 뒤 `drop(a); drop(b);`
//     5) `probe.upgrade().is_some()` 를 돌려준다. (true = 아직 살아 있음 = 누수!)
//   🐍 Python: weakref.ref + del a, b  (gc.disable() 상태)                  (python/main.py: demo_cycle_leak)
//   ⚙️ C++  : std::weak_ptr probe = a;  a.reset(); b.reset();  !probe.expired() (cpp/main.cpp: demo_cycle_leak)
//   🦀 Rust : drop(a) 후에도 b.other 가 a 를 쥐고 있어서 strong 이 0 이 되지 않습니다.
//   ❓ 왜?: Rc 는 참조 카운트만 쓰는 방식이라 순환을 못 찾습니다. (Python 은 별도 순환 GC 가 있어요)
//           메모리 "누수"는 unsafe 가 아니라서 Rust 가 막아 주지 않습니다. 안전 ≠ 누수 없음.
// ─────────────────────────────────────────────
pub fn demo_cycle_leak() -> bool {
    todo!("TODO(10): demo_cycle_leak")
}

// ─────────────────────────────────────────────
// TODO(11): 같은 구조인데 b -> a 방향만 Weak 로 만들어 순환을 끊으세요. 누수가 없어야 합니다(false).
//   절차: `*a.strong_other.borrow_mut() = Some(Rc::clone(&b));`  `*b.weak_other.borrow_mut() = Rc::downgrade(&a);`
//   출력 형식: `fixed: a strong=1 b strong=2`  (a 는 강한 참조가 변수 하나뿐, b 는 변수 + a.strong_other)
//   그 뒤 `eprintln!("(stderr) a weak_count={}", Rc::weak_count(&a));` 로 약한 참조 개수도 구경해 보세요.
//   마지막에 `probe.upgrade().is_some()` 를 돌려줍니다 (이번엔 false 여야 해요).
//   💡 힌트: WeakLink 는 필드가 `RefCell<Weak<WeakLink>>` 라서 `Weak::new()` 로 빈 값으로 시작합니다.
//   ❓ 왜?: 소유 관계는 "한 방향(부모 -> 자식)", 되돌아가는 방향은 Weak 라는 규칙이 트리/그래프의 표준 해법입니다.
// ─────────────────────────────────────────────
pub fn demo_weak_fixed() -> bool {
    todo!("TODO(11): demo_weak_fixed")
}

fn main() {
    println!("== Part A: Expr ==");
    let expr = mul(add(num(2.0), num(3.0)), neg(add(num(4.0), num(1.5))));
    println!("expr: {}", show(&expr));
    println!("value: {:.2}", eval(&expr));
    println!("depth: {}", depth(&expr));
    let messy = add(mul(num(1.0), neg(neg(num(7.0)))), num(0.0));
    println!("messy: {}", show(&messy));
    let simple = simplify(messy); // messy 의 소유권이 simplify 로 넘어갑니다.
    println!("simplified: {} (depth {})", show(&simple), depth(&simple));

    println!("== Part B: dialogue tree ==");
    let root = new_node("Hello");
    let order = add_child(&root, "Order status");
    let track = add_child(&order, "Track package");
    add_child(&order, "Cancel order");
    let refund = add_child(&root, "Refund");
    add_child(&refund, "Refund policy");
    add_child(&root, "Chat");
    println!("paths:");
    for p in leaf_paths(&root) {
        println!("  {p}");
    }
    println!("path_to_root(track) = {}", path_to_root(&track));
    println!(
        "strong_count(root)={} strong_count(order)={}",
        Rc::strong_count(&root),
        Rc::strong_count(&order)
    );
    eprintln!("(stderr) weak_count(root)={}", Rc::weak_count(&root));
    visit(&track);
    visit(&track);
    visit(&refund);
    visit(&root);
    println!("total visits: {}", total_visits(&root));
    eprintln!("(stderr) try_borrow_mut while reading ok? {}", try_borrow_while_reading());
    // lab_refcell_panic(); // 🔬 실험 2: 주석을 풀면 여기서 panic

    println!("== Cycle experiment ==");
    let leaked = demo_cycle_leak();
    println!("cycle leaked: {}", if leaked { "yes" } else { "no" });
    let leaked = demo_weak_fixed();
    println!("fixed leaked: {}", if leaked { "yes" } else { "no" });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Expr {
        mul(add(num(2.0), num(3.0)), neg(add(num(4.0), num(1.5))))
    }

    #[test]
    fn eval_and_show() {
        assert_eq!(eval(&sample()), -27.5);
        assert_eq!(show(&sample()), "((2.0 + 3.0) * -(4.0 + 1.5))");
    }

    #[test]
    fn depth_counts_levels() {
        assert_eq!(depth(&num(1.0)), 1);
        assert_eq!(depth(&sample()), 4);
    }

    #[test]
    fn simplify_identities() {
        assert_eq!(simplify(add(num(0.0), num(5.0))), num(5.0));
        assert_eq!(simplify(mul(num(1.0), neg(neg(num(7.0))))), num(7.0));
        assert_eq!(simplify(mul(num(9.0), num(0.0))), num(0.0));
        // 정리할 것이 없으면 그대로
        assert_eq!(simplify(sample()), sample());
    }

    fn small_tree() -> (NodeRef, NodeRef, NodeRef) {
        let root = new_node("root");
        let a = add_child(&root, "a");
        let b = add_child(&a, "b");
        add_child(&root, "c");
        (root, a, b)
    }

    #[test]
    fn paths() {
        let (root, _a, b) = small_tree();
        assert_eq!(path_to_root(&b), "root > a > b");
        assert_eq!(leaf_paths(&root), vec!["root > a > b", "root > c"]);
    }

    #[test]
    fn visits_through_shared_handles() {
        let (root, a, b) = small_tree();
        visit(&a);
        visit(&b);
        visit(&b);
        assert_eq!(total_visits(&root), 3);
    }

    #[test]
    fn rc_counts() {
        let (root, a, _b) = small_tree();
        // a: 변수 `a` + root.children 안의 핸들 = 2
        assert_eq!(Rc::strong_count(&a), 2);
        // root: 변수 하나뿐 (자식들은 Weak 로 부모를 가리킴)
        assert_eq!(Rc::strong_count(&root), 1);
        assert_eq!(Rc::weak_count(&root), 2);
    }

    #[test]
    fn parent_is_weak_and_dies_with_tree() {
        let child;
        {
            let (root, a, _b) = small_tree();
            child = a;
            assert!(child.borrow().parent.upgrade().is_some());
            drop(root);
        }
        // root 가 해제되었으므로 upgrade 는 None
        assert!(child.borrow().parent.upgrade().is_none());
    }

    #[test]
    fn refcell_checks_at_runtime() {
        assert!(!try_borrow_while_reading());
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn double_borrow_panics() {
        let cell = RefCell::new(vec![1]);
        let _first = cell.borrow_mut();
        let _second = cell.borrow_mut(); // 런타임 panic
    }

    #[test]
    fn cycle_leaks_but_weak_does_not() {
        assert!(demo_cycle_leak());
        assert!(!demo_weak_fixed());
    }
}
