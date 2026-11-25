"""Ch05 자판기 상태 머신 (Python 참조 구현).

Rust 의 enum 은 Python 에서는 "dataclass 여러 개 + match" 로 흉내 냅니다.
차이점: Python 은 새 상태 클래스를 추가해도 match 가 빠뜨린 분기를 알려 주지 않습니다.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class Item(Enum):
    COLA = "Cola"
    CHIPS = "Chips"
    CANDY = "Candy"

    @property
    def price(self) -> int:
        return {Item.COLA: 150, Item.CHIPS: 100, Item.CANDY: 70}[self]

    @property
    def label(self) -> str:
        return self.value

    @staticmethod
    def from_name(s: str) -> Item | None:  # Rust: Option<Item>
        for item in Item:
            if item.value.lower() == s.lower():
                return item
        return None


# ── Event: Rust 의 enum Event 에 해당 ──
@dataclass(frozen=True)
class InsertCoin:
    cents: int


@dataclass(frozen=True)
class Select:
    item: Item


@dataclass(frozen=True)
class Cancel: ...


@dataclass(frozen=True)
class Take: ...


@dataclass(frozen=True)
class Kick: ...


@dataclass(frozen=True)
class Repair: ...


Event = InsertCoin | Select | Cancel | Take | Kick | Repair


# ── State ──
@dataclass(frozen=True)
class Idle: ...


@dataclass(frozen=True)
class Collecting:
    cents: int


@dataclass(frozen=True)
class Dispensing:
    item: Item
    change: int


@dataclass(frozen=True)
class Refunding:
    cents: int


@dataclass(frozen=True)
class OutOfService:
    reason: str


State = Idle | Collecting | Dispensing | Refunding | OutOfService

SCRIPT = [
    "coin 100", "coin 7", "coin 100", "select cola", "take", "select chips",
    "coin 500", "select candy", "take", "coin 50", "cancel", "take", "dance",
    "coin 100", "kick", "coin 100", "repair", "coin 10", "select chips",
]  # fmt: skip


def event_str(event: Event) -> str:
    match event:
        case InsertCoin(c):
            return f"InsertCoin({c})"
        case Select(item):
            return f"Select({item.label})"
        case Cancel():
            return "Cancel"
        case Take():
            return "Take"
        case Kick():
            return "Kick"
        case Repair():
            return "Repair"
    raise AssertionError("unreachable")


def is_valid_coin(c: int) -> bool:
    return c in (10, 50, 100, 500)


def parse_event(line: str) -> Event | None:
    parts = line.split()
    if not parts:
        return None
    cmd, args = parts[0], parts[1:]
    match (cmd, args):
        case ("coin", [a]) if a.isascii() and a.isdigit():
            return InsertCoin(int(a))
        case ("select", [a]):
            item = Item.from_name(a)
            return Select(item) if item else None
        case ("cancel", []):
            return Cancel()
        case ("take", []):
            return Take()
        case ("kick", []):
            return Kick()
        case ("repair", []):
            return Repair()
    return None


def state_str(state: State) -> str:
    match state:
        case Idle():
            return "Idle"
        case Collecting(cents):
            return f"Collecting({cents})"
        case Dispensing(item, change):
            return f"Dispensing({item.label}, change={change})"
        case Refunding(cents):
            return f"Refunding({cents})"
        case OutOfService(reason):
            return f"OutOfService({reason})"
    raise AssertionError("unreachable")


def transition(state: State, event: Event) -> State:
    match (state, event):
        case (OutOfService(), Repair()):
            return Idle()
        case (OutOfService(), _):
            return state
        case (_, Kick()):
            return OutOfService("kicked")
        case (Idle(), InsertCoin(c)) if is_valid_coin(c):
            return Collecting(c)
        case (Collecting(cents), InsertCoin(c)) if is_valid_coin(c):
            return Collecting(cents + c)
        case (Collecting(cents), Select(item)) if cents >= item.price:
            return Dispensing(item, cents - item.price)
        case (Collecting(cents), Cancel()):
            return Refunding(cents)
        case (Dispensing() | Refunding(), Take()):
            return Idle()
    return state  # 나머지 조합: 아무 일도 안 일어남


def sale_of(state: State, event: Event) -> Item | None:
    if isinstance(state, Dispensing) and isinstance(event, Take):
        return state.item
    return None


class Machine:
    def __init__(self) -> None:
        self.state: State = Idle()
        self.sold: list[Item] = []

    def handle(self, event: Event) -> None:
        item = sale_of(self.state, event)
        if item is not None:
            self.sold.append(item)
        self.state = transition(self.state, event)

    def revenue(self) -> int:
        return sum(i.price for i in self.sold)

    def into_sold(self) -> list[Item]:
        # Python 에는 "소비(move)" 개념이 없어서 그냥 돌려줍니다.
        return self.sold


def main() -> None:
    machine = Machine()
    for i, line in enumerate(SCRIPT, start=1):
        event = parse_event(line)
        if event is not None:
            machine.handle(event)
            label = event_str(event)
        else:
            label = f"? {line}"
        print(f"step {i:02d}: {label:<16} -> {state_str(machine.state)}")
    print(f"revenue: {machine.revenue()}")
    print("sold: " + ", ".join(i.label for i in machine.into_sold()))


if __name__ == "__main__":
    main()
