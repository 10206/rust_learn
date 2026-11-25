//! Ch11 정답: 스마트 포인터 (Box / Rc / Weak / RefCell)
//!
//! 실행: `cargo run -q -p ch11_smart_pointers --example solution`
//! 테스트: `cargo test -p ch11_smart_pointers --example solution`

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

// Box::new 를 매번 쓰기 번거로우니 생성 도우미를 둡니다.
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

/// 식을 계산합니다. `&Box<Expr>` 는 자동으로 `&Expr` 로 역참조(Deref) 됩니다.
pub fn eval(e: &Expr) -> f64 {
    match e {
        Expr::Num(v) => *v,
        Expr::Add(l, r) => eval(l) + eval(r),
        Expr::Mul(l, r) => eval(l) * eval(r),
        Expr::Neg(x) => -eval(x),
    }
}

/// 사람이 읽기 좋은 문자열로 만듭니다.
pub fn show(e: &Expr) -> String {
    match e {
        Expr::Num(v) => format!("{v:.1}"),
        Expr::Add(l, r) => format!("({} + {})", show(l), show(r)),
        Expr::Mul(l, r) => format!("({} * {})", show(l), show(r)),
        Expr::Neg(x) => format!("-{}", show(x)),
    }
}

/// 트리의 깊이 (잎 하나 = 1).
pub fn depth(e: &Expr) -> usize {
    match e {
        Expr::Num(_) => 1,
        Expr::Add(l, r) | Expr::Mul(l, r) => 1 + depth(l).max(depth(r)),
        Expr::Neg(x) => 1 + depth(x),
    }
}

/// 항등원/이중 부정을 정리합니다. 소유권을 받아서 새 트리를 돌려줍니다.
pub fn simplify(e: Expr) -> Expr {
    match e {
        Expr::Num(_) => e,
        Expr::Add(l, r) => match (simplify(*l), simplify(*r)) {
            // `*l` : Box 에서 값을 "꺼내" 옵니다 (Box 만 이렇게 move 가능).
            (Expr::Num(z), x) | (x, Expr::Num(z)) if z == 0.0 => x,
            (l, r) => add(l, r),
        },
        Expr::Mul(l, r) => match (simplify(*l), simplify(*r)) {
            (Expr::Num(z), _) | (_, Expr::Num(z)) if z == 0.0 => num(0.0),
            (Expr::Num(o), x) | (x, Expr::Num(o)) if o == 1.0 => x,
            (l, r) => mul(l, r),
        },
        Expr::Neg(x) => match simplify(*x) {
            Expr::Neg(inner) => *inner,
            other => neg(other),
        },
    }
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

pub fn new_node(text: &str) -> NodeRef {
    Rc::new(RefCell::new(Node {
        text: text.to_string(),
        parent: Weak::new(),
        children: Vec::new(),
        visits: 0,
    }))
}

/// 자식 노드를 만들어 부모에 연결하고, 자식 핸들(Rc)을 돌려줍니다.
pub fn add_child(parent: &NodeRef, text: &str) -> NodeRef {
    let child = new_node(text);
    child.borrow_mut().parent = Rc::downgrade(parent);
    parent.borrow_mut().children.push(Rc::clone(&child));
    child
}

/// "Hello > Order status > Track package" 처럼 루트부터 이 노드까지의 경로.
pub fn path_to_root(node: &NodeRef) -> String {
    let mut parts = vec![node.borrow().text.clone()];
    let mut cur = node.borrow().parent.upgrade();
    while let Some(p) = cur {
        parts.push(p.borrow().text.clone());
        cur = p.borrow().parent.upgrade();
    }
    parts.reverse();
    parts.join(" > ")
}

/// 루트에서 모든 잎(leaf)까지의 경로를 DFS 순서로 돌려줍니다.
pub fn leaf_paths(root: &NodeRef) -> Vec<String> {
    let node = root.borrow();
    if node.children.is_empty() {
        return vec![path_to_root(root)];
    }
    node.children.iter().flat_map(leaf_paths).collect()
}

/// `&NodeRef` (공유 참조) 인데도 내부를 바꿀 수 있습니다: 내부 가변성.
pub fn visit(node: &NodeRef) {
    node.borrow_mut().visits += 1;
}

pub fn total_visits(root: &NodeRef) -> u32 {
    let n = root.borrow();
    n.visits + n.children.iter().map(total_visits).sum::<u32>()
}

/// RefCell 은 빌림 규칙을 런타임에 검사합니다. 읽는 중에 쓰기를 시도하면 Err.
pub fn try_borrow_while_reading() -> bool {
    let cell = RefCell::new(5);
    let _reader = cell.borrow();
    cell.try_borrow_mut().is_ok()
}

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

/// a -> b -> a 를 모두 강한 참조로 잇고, 변수를 버린 뒤에도 살아 있는지(= 누수) 돌려줍니다.
pub fn demo_cycle_leak() -> bool {
    let a = Rc::new(Link { name: "a", other: RefCell::new(None) });
    let b = Rc::new(Link { name: "b", other: RefCell::new(None) });
    *a.other.borrow_mut() = Some(Rc::clone(&b));
    *b.other.borrow_mut() = Some(Rc::clone(&a));
    println!(
        "cycle: {} strong={} {} strong={}",
        a.name,
        Rc::strong_count(&a),
        b.name,
        Rc::strong_count(&b)
    );
    let probe = Rc::downgrade(&a);
    drop(a);
    drop(b);
    // 변수는 사라졌지만 서로가 서로를 붙잡고 있어서 카운트가 0 이 되지 않습니다.
    probe.upgrade().is_some()
}

/// b -> a 를 Weak 로 바꾸면 순환이 끊겨서 모두 해제됩니다.
pub fn demo_weak_fixed() -> bool {
    let a = Rc::new(WeakLink {
        name: "a",
        strong_other: RefCell::new(None),
        weak_other: RefCell::new(Weak::new()),
    });
    let b = Rc::new(WeakLink {
        name: "b",
        strong_other: RefCell::new(None),
        weak_other: RefCell::new(Weak::new()),
    });
    *a.strong_other.borrow_mut() = Some(Rc::clone(&b));
    *b.weak_other.borrow_mut() = Rc::downgrade(&a);
    println!(
        "fixed: {} strong={} {} strong={}",
        a.name,
        Rc::strong_count(&a),
        b.name,
        Rc::strong_count(&b)
    );
    // weak_count 는 Rust 에만 있는 정보라서 stderr 로만 보여 줍니다.
    eprintln!(
        "(stderr) a weak_count={}, b.weak_other -> {:?}",
        Rc::weak_count(&a),
        b.weak_other.borrow().upgrade().map(|p| p.name)
    );
    let linked = a.strong_other.borrow().is_some();
    let probe = Rc::downgrade(&a);
    drop(a);
    drop(b);
    linked && probe.upgrade().is_some()
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
