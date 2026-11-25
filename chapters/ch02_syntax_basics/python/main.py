"""Ch02. 콜라츠 통계 - Python 참조 구현.

Rust 버전(rust/src/main.rs)과 함수 이름을 맞춰 두었어요.
Python 의 int 는 크기 제한이 없어서 "오버플로"가 아예 없습니다.
그래서 u8 / u32 의 한계는 max_value 인자로 흉내 냅니다. (Rust 에서는 타입 자체가 한계를 가집니다)
"""
import math
from typing import Optional

U8_MAX = 255
U32_MAX = 2**32 - 1


# ---------- 콜라츠 기본 ----------
def collatz_next(n: int) -> int:
    # Rust: if 는 "식"이라서  if n % 2 == 0 { n / 2 } else { 3 * n + 1 }  이 그대로 반환값이 됩니다.
    return n // 2 if n % 2 == 0 else 3 * n + 1


def collatz_steps(n: int) -> int:
    steps = 0
    while n != 1:
        n = collatz_next(n)
        steps += 1
    return steps


def collatz_peak(n: int) -> int:
    peak = n
    while n != 1:
        n = collatz_next(n)
        peak = max(peak, n)
    return peak


def classify_steps(steps: int) -> str:
    # Rust: match steps { 0 => ..., 1..=10 => ..., 11..=50 => ..., _ => ... }
    if steps == 0:
        return "start"
    if steps <= 10:
        return "short"
    if steps <= 50:
        return "medium"
    return "long"


def longest_chain(limit: int) -> tuple[int, int]:
    best = (1, 0)
    for n in range(1, limit + 1):
        s = collatz_steps(n)
        if s > best[1]:  # 같은 길이면 먼저 나온 n 을 유지
            best = (n, s)
    return best


def max_peak(limit: int) -> tuple[int, int]:
    best = (1, 1)
    for n in range(1, limit + 1):
        p = collatz_peak(n)
        if p > best[1]:
            best = (n, p)
    return best


# ---------- 오버플로 ----------
def checked_next(n: int, max_value: int) -> Optional[int]:
    """Rust: n.checked_mul(3)?.checked_add(1) 처럼, 넘치면 None 을 돌려줍니다."""
    if n % 2 == 0:
        return n // 2
    v = 3 * n + 1
    return v if v <= max_value else None


def overflow_step(start: int, max_value: int) -> Optional[int]:
    """start 에서 1 까지 가는 동안 처음 넘치는 순간까지 "성공한" 단계 수. 안 넘치면 None."""
    n = start
    steps = 0
    while n != 1:
        nxt = checked_next(n, max_value)
        if nxt is None:
            return steps
        n = nxt
        steps += 1
    return None


def find_first_exceeding(limit: int, ceiling: int) -> Optional[tuple[int, int]]:
    """1..=limit 중, 사슬 도중 ceiling 보다 커지는 첫 (n, 그 값). Rust 는 labeled break 로 바깥 루프를 한 번에 탈출."""
    for n in range(1, limit + 1):
        x = n
        while True:
            if x > ceiling:
                return (n, x)
            if x == 1:
                break
            x = collatz_next(x)
    return None


# ---------- as 변환 ----------
def to_u8(x: int) -> int:
    # Rust: x as u8  -> 하위 8비트만 남김 (모듈로 256)
    return x & 0xFF


def trunc_to_i32(x: float) -> int:
    # Rust: x as i32  -> 소수부 버림 (0 쪽으로)
    return math.trunc(x)


# ---------- 출력 ----------
def fmt_opt(v: Optional[int]) -> str:
    return "None" if v is None else f"Some({v})"


def trunc_div(a: int, b: int) -> int:
    """Rust / C++ 의 정수 나눗셈은 0 쪽으로 버림. Python 의 // 는 -inf 쪽으로 버려서 다릅니다!"""
    return int(a / b)


def trunc_rem(a: int, b: int) -> int:
    """Rust / C++ 의 % 는 나머지(부호는 피제수를 따름). Python 의 % 는 항상 b 의 부호를 따릅니다."""
    return int(math.fmod(a, b))


def basics_demo() -> None:
    print("== basics ==")
    x = 5
    print(f"shadowing: {x}", end="")
    x = x * 2  # Python 은 그냥 재할당. Rust 의 shadowing 은 새 변수(타입도 바꿀 수 있음)
    print(f" -> {x}", end="")
    x = f"{x} apples"
    print(f" -> {x}")

    point = (3, 4)
    px, py = point
    print(f"tuple: ({point[0]}, {point[1]}) -> x={px} y={py}")
    arr = [10, 20, 30]
    print(f"array: [{', '.join(map(str, arr))}] len={len(arr)} sum={sum(arr)}")

    print(
        f"div: 7/2={trunc_div(7, 2)} -7/2={trunc_div(-7, 2)} "
        f"-7%3={trunc_rem(-7, 3)} rem_euclid={-7 % 3}"
    )
    print(
        f"casts: 300 as u8={to_u8(300)} -1 as u8={to_u8(-1)} "
        f"3.99 as i32={trunc_to_i32(3.99)} -3.99 as i32={trunc_to_i32(-3.99)}"
    )

    label = "big" if collatz_steps(27) > 100 else "small"
    block = (lambda a: a * a)(3)
    counter = 0
    while True:
        counter += 1
        if counter == 5:
            loop_value = counter * 2
            break
    print(f"exprs: if={label} block={block} loop={loop_value}")


def overflow_demo() -> None:
    print("== overflow: u8 add ==")
    for a, b in ((250, 5), (250, 10)):
        total = a + b
        checked = total if total <= U8_MAX else None
        wrapping = total & 0xFF
        saturating = min(total, U8_MAX)
        print(f"{a} + {b}: checked={fmt_opt(checked)} wrapping={wrapping} saturating={saturating}")

    print("== overflow: collatz ==")
    overflowing = [(s, overflow_step(s, U8_MAX)) for s in range(1, 256)]
    overflowing = [(s, st) for s, st in overflowing if st is not None]
    first_s, first_st = overflowing[0]
    print(f"u8 starts 1..=255 that overflow: {len(overflowing)} (first: {first_s} at step {first_st})")
    for start in (27, 113383, 159487):
        step = overflow_step(start, U32_MAX)
        if step is None:
            print(f"u32 start={start}: no overflow")
        else:
            print(f"u32 start={start}: overflow at step {step}")


def main() -> None:
    basics_demo()

    print("== collatz 1..=30 ==")
    print(f"{'n':>3} {'steps':>5} {'peak':>6} kind")
    for n in range(1, 31):
        s = collatz_steps(n)
        print(f"{n:>3} {s:>5} {collatz_peak(n):>6} {classify_steps(s)}")
    n, s = longest_chain(30)
    print(f"longest chain: n={n} steps={s}")
    n, p = max_peak(30)
    print(f"highest peak: n={n} peak={p}")

    overflow_demo()

    print("== labeled break ==")
    for limit, ceiling in ((100, 1000), (30, 10000)):
        found = find_first_exceeding(limit, ceiling)
        if found is None:
            print(f"first n<={limit} whose chain exceeds {ceiling}: none")
        else:
            print(f"first n<={limit} whose chain exceeds {ceiling}: n={found[0]} value={found[1]}")


if __name__ == "__main__":
    main()
