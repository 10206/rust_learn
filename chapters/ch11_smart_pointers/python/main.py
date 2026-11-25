"""Ch11. 스마트 포인터 - Python 참조 구현.

Python 에서는 모든 객체가 이미 "참조 카운트되는 포인터"입니다.
  - Box<T>          -> 그냥 객체 (필요 없음)
  - Rc<T>           -> 평범한 참조 (CPython 이 참조 카운트를 자동 관리)
  - RefCell<T>      -> 필요 없음 (아무 때나 수정 가능)
  - Weak<T>         -> weakref.ref
Rust 에서는 이것들을 "명시적으로" 고릅니다.
"""

from __future__ import annotations

import gc
import sys
import weakref
from dataclasses import dataclass, field


# ───────────── Part A: 재귀 enum  Expr ─────────────
@dataclass
class Num:
    value: float


@dataclass
class Add:
    left: "Expr"
    right: "Expr"


@dataclass
class Mul:
    left: "Expr"
    right: "Expr"


@dataclass
class Neg:
    inner: "Expr"


Expr = Num | Add | Mul | Neg


def eval_expr(e: Expr) -> float:
    match e:
        case Num(v):
            return v
        case Add(l, r):
            return eval_expr(l) + eval_expr(r)
        case Mul(l, r):
            return eval_expr(l) * eval_expr(r)
        case Neg(x):
            return -eval_expr(x)


def show(e: Expr) -> str:
    match e:
        case Num(v):
            return f"{v:.1f}"
        case Add(l, r):
            return f"({show(l)} + {show(r)})"
        case Mul(l, r):
            return f"({show(l)} * {show(r)})"
        case Neg(x):
            return f"-{show(x)}"


def depth(e: Expr) -> int:
    match e:
        case Num():
            return 1
        case Add(l, r) | Mul(l, r):
            return 1 + max(depth(l), depth(r))
        case Neg(x):
            return 1 + depth(x)


def simplify(e: Expr) -> Expr:
    # Rust 에서는 Box 를 "소유권째로" 받아서 분해합니다 (값을 꺼내 쓰고 새로 조립).
    match e:
        case Num():
            return e
        case Add(l, r):
            l, r = simplify(l), simplify(r)
            if l == Num(0.0):
                return r
            if r == Num(0.0):
                return l
            return Add(l, r)
        case Mul(l, r):
            l, r = simplify(l), simplify(r)
            if l == Num(0.0) or r == Num(0.0):
                return Num(0.0)
            if l == Num(1.0):
                return r
            if r == Num(1.0):
                return l
            return Mul(l, r)
        case Neg(x):
            x = simplify(x)
            if isinstance(x, Neg):
                return x.inner
            return Neg(x)


# ───────────── Part B: 대화 분기 트리 ─────────────
class Node:
    def __init__(self, text: str) -> None:
        self.text = text
        # Rust: Weak<RefCell<Node>>  (부모를 강하게 잡으면 순환 -> 누수)
        self.parent: weakref.ref[Node] | None = None
        self.children: list[Node] = []
        self.visits = 0


def new_node(text: str) -> Node:
    return Node(text)


def add_child(parent: Node, text: str) -> Node:
    child = Node(text)
    child.parent = weakref.ref(parent)
    parent.children.append(child)
    return child


def path_to_root(node: Node) -> str:
    parts = [node.text]
    cur = node.parent() if node.parent else None
    while cur is not None:
        parts.append(cur.text)
        cur = cur.parent() if cur.parent else None
    return " > ".join(reversed(parts))


def leaf_paths(root: Node) -> list[str]:
    if not root.children:
        return [path_to_root(root)]
    out: list[str] = []
    for c in root.children:
        out.extend(leaf_paths(c))
    return out


def visit(node: Node) -> None:
    node.visits += 1


def total_visits(root: Node) -> int:
    return root.visits + sum(total_visits(c) for c in root.children)


def strong_count(obj: object) -> int:
    # getrefcount 는 인자 자체(+1)와 이 함수의 매개변수(+1)를 포함하므로 2를 뺍니다.
    return sys.getrefcount(obj) - 2


# ───────────── 순환 참조 실험 ─────────────
class Link:
    def __init__(self, name: str) -> None:
        self.name = name
        self.other: Link | None = None  # 강한 참조


class WeakLink:
    def __init__(self, name: str) -> None:
        self.name = name
        self.strong_other: WeakLink | None = None
        self.weak_other: weakref.ref[WeakLink] | None = None


def demo_cycle_leak() -> bool:
    a = Link("a")
    b = Link("b")
    a.other = b
    b.other = a
    print(f"cycle: a strong={strong_count(a)} b strong={strong_count(b)}")
    probe = weakref.ref(a)
    del a, b
    # Python 은 순환 GC 가 따로 있어서 gc.collect() 를 하면 회수됩니다.
    # Rust 의 Rc 에는 그런 GC 가 없으므로 영원히 남습니다. (여기서는 gc 를 꺼 둔 상태)
    return probe() is not None


def demo_weak_fixed() -> bool:
    a = WeakLink("a")
    b = WeakLink("b")
    a.strong_other = b
    b.weak_other = weakref.ref(a)
    print(f"fixed: a strong={strong_count(a)} b strong={strong_count(b)}")
    probe = weakref.ref(a)
    del a, b
    return probe() is not None


def main() -> None:
    print("== Part A: Expr ==")
    expr = Mul(Add(Num(2.0), Num(3.0)), Neg(Add(Num(4.0), Num(1.5))))
    print(f"expr: {show(expr)}")
    print(f"value: {eval_expr(expr):.2f}")
    print(f"depth: {depth(expr)}")
    messy = Add(Mul(Num(1.0), Neg(Neg(Num(7.0)))), Num(0.0))
    print(f"messy: {show(messy)}")
    simple = simplify(messy)
    print(f"simplified: {show(simple)} (depth {depth(simple)})")

    print("== Part B: dialogue tree ==")
    root = new_node("Hello")
    order = add_child(root, "Order status")
    track = add_child(order, "Track package")
    add_child(order, "Cancel order")
    refund = add_child(root, "Refund")
    add_child(refund, "Refund policy")
    add_child(root, "Chat")
    print("paths:")
    for p in leaf_paths(root):
        print(f"  {p}")
    print(f"path_to_root(track) = {path_to_root(track)}")
    # root: 변수 root 하나 / order: 변수 + root.children 리스트 = 2
    print(f"strong_count(root)={strong_count(root)} strong_count(order)={strong_count(order)}")
    visit(track)
    visit(track)
    visit(refund)
    visit(root)
    print(f"total visits: {total_visits(root)}")

    print("== Cycle experiment ==")
    gc.disable()  # Rust 의 Rc 와 같은 조건을 만들기 위해 순환 GC 를 끕니다.
    print(f"cycle leaked: {'yes' if demo_cycle_leak() else 'no'}")
    print(f"fixed leaked: {'yes' if demo_weak_fixed() else 'no'}")


if __name__ == "__main__":
    main()
