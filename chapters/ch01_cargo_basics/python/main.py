"""Ch01. 온도 변환표 - Python 참조 구현.

Rust 버전(rust/src/main.rs)과 함수 이름을 맞춰 두었어요.
Python은 인터프리터가 한 줄씩 실행하고, 타입은 실행 중에 결정됩니다.
"""


def greeting(name: str) -> str:
    # Rust: format!("Hello, {}!", name)  -> 화면에 찍지 않고 String 을 "만들어" 돌려줍니다.
    return f"Hello, {name}!"


def c_to_f(c: float) -> float:
    return c * 9.0 / 5.0 + 32.0


def f_to_c(f: float) -> float:
    return (f - 32.0) * 5.0 / 9.0


def table_line(c: float) -> str:
    # 폭 7, 소수점 1자리, 오른쪽 정렬
    return f"{c:>7.1f} {c_to_f(c):>7.1f}"


def print_table() -> None:
    print(f"{'C':>7} {'F':>7}")
    # Rust 의 (-10..=40).step_by(10) 에 해당: range 는 끝을 포함하지 않으므로 41
    for c in range(-10, 41, 10):
        print(table_line(float(c)))


def min_max_avg(values: list[float]) -> tuple[float, float, float]:
    # Python 은 min / max / sum 이 내장 함수라 한 줄이면 끝나요.
    return min(values), max(values), sum(values) / len(values)


def main() -> None:
    print(greeting("Cargo"))
    print("=== Celsius to Fahrenheit ===")
    print_table()

    print("=== round trip (F -> C) ===")
    for f in (14.0, 32.0, 98.6, 212.0):
        print(f"{f:>6.1f} F = {f_to_c(f):>6.1f} C")

    print("=== sample readings ===")
    readings = [22.5, 18.0, 30.1, 15.7, 25.3]
    print("readings: " + ", ".join(f"{r:.1f}" for r in readings))
    lo, hi, avg = min_max_avg(readings)
    print(f"min: {lo:.1f}  max: {hi:.1f}  avg: {avg:.2f}")


if __name__ == "__main__":
    main()
