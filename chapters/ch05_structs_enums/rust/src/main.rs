//! # Ch05. 구조체·열거형·패턴 매칭 — 자판기 상태 머신
//!
//! ## 목표
//! - `struct` + `impl` 로 데이터와 메서드(`&self`, `&mut self`, `self`)를 묶는다.
//! - 데이터를 품은 `enum`(sum type)으로 "상태"와 "사건"을 표현한다.
//! - `match` 의 exhaustive(빠짐없음) 검사, `if let`, `let else`, `Option` 을 쓴다.
//!
//! ## 대응 파일
//! - Python : `../python/main.py`  (dataclass + match)
//! - C++    : `../cpp/main.cpp`    (std::variant + std::visit)
//!
//! ## 진행 방법
//! 1. 아래 `TODO(1)` 부터 번호 순서대로 `todo!()` 를 지우고 구현합니다.
//! 2. 중간중간 `cargo test -p ch05_structs_enums` 로 확인합니다.
//! 3. 막히면 `examples/solution.rs` 를 열어 보되, 먼저 힌트(💡)를 읽고 5분 이상 고민해 보세요.
//!
//! ## 완료 기준
//! `cargo test -p ch05_structs_enums` 가 모두 초록불이고,
//! `../../scripts/compare.sh ch05` 에서 Python / C++ / Rust 출력이 같으면 끝입니다.

// 다 풀고 나면 이 줄을 지워 보세요 (아직 쓰지 않은 코드에 대한 경고를 잠시 꺼 둡니다).
#![allow(dead_code, unused_variables, unused_mut, unused_imports)]

use std::fmt;

// ── 데이터 정의 (제공됨) ───────────────────────────────────

/// 상품 종류. 데이터가 없는 enum 은 `Copy` 까지 붙여 가볍게 다룹니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Cola,
    Chips,
    Candy,
}

/// 자판기에 들어오는 사건. variant 마다 담는 데이터가 다릅니다.
#[derive(Debug, Clone, PartialEq)]
enum Event {
    InsertCoin(u32),
    Select(Item),
    Cancel,
    Take,
    Kick,
    Repair,
}

/// 자판기의 상태. 상태마다 "그 상태에서만 의미 있는 데이터"를 갖습니다.
#[derive(Debug, Clone, PartialEq)]
enum State {
    Idle,
    Collecting { cents: u32 },
    Dispensing { item: Item, change: u32 },
    Refunding { cents: u32 },
    OutOfService { reason: String },
}

impl Item {
    fn name(self) -> &'static str {
        match self {
            Item::Cola => "Cola",
            Item::Chips => "Chips",
            Item::Candy => "Candy",
        }
    }

    // ─────────────────────────────────────────────
    // TODO(1): 상품 가격(센트)을 돌려주세요. Cola=150, Chips=100, Candy=70.
    //   🐍 Python: {Item.COLA: 150, ...}[self]            (python/main.py: Item.price)
    //   ⚙️ C++  : switch (i) { case Item::Cola: ... }      (cpp/main.cpp: price)
    //   🦀 Rust : `match self { ... }` — variant 를 하나라도 빼먹으면 컴파일이 안 됩니다.
    //   💡 힌트: 위의 `name()` 이 같은 모양입니다. 반환 타입은 u32.
    //   ❓ 왜?: Item 에 variant 를 추가하면 컴파일러가 고쳐야 할 모든 match 를 찾아 줍니다.
    // ─────────────────────────────────────────────
    fn price(self) -> u32 {
        todo!("TODO(1): match 로 가격 돌려주기")
    }

    // ─────────────────────────────────────────────
    // TODO(2): "cola"/"Cola"/"COLA" 같은 문자열을 Item 으로 바꿉니다. 모르는 이름이면 None.
    //   🐍 Python: Item.from_name 이 None 반환              (python/main.py: Item.from_name)
    //   ⚙️ C++  : std::optional<Item>                       (cpp/main.cpp: item_from_name)
    //   🦀 Rust : `Option<Item>` — null 대신 "없을 수 있음"이 타입에 드러납니다.
    //   💡 힌트: s.to_lowercase().as_str() 를 match 하고, 마지막 팔에 `_ => None`.
    //   ❓ 왜?: 문자열은 무한히 많은 값이라 `_` (나머지 전부) 팔이 꼭 필요합니다.
    // ─────────────────────────────────────────────
    fn from_name(s: &str) -> Option<Item> {
        todo!("TODO(2): 이름 -> Option<Item>")
    }
}

// ── 제공되는 코드 ───────────────────────────────────────────

fn event_str(event: &Event) -> String {
    match event {
        Event::InsertCoin(c) => format!("InsertCoin({c})"),
        Event::Select(item) => format!("Select({})", item.name()),
        Event::Cancel => "Cancel".to_string(),
        Event::Take => "Take".to_string(),
        Event::Kick => "Kick".to_string(),
        Event::Repair => "Repair".to_string(),
    }
}

fn is_valid_coin(c: u32) -> bool {
    matches!(c, 10 | 50 | 100 | 500)
}

const SCRIPT: [&str; 19] = [
    "coin 100",
    "coin 7",
    "coin 100",
    "select cola",
    "take",
    "select chips",
    "coin 500",
    "select candy",
    "take",
    "coin 50",
    "cancel",
    "take",
    "dance",
    "coin 100",
    "kick",
    "coin 100",
    "repair",
    "coin 10",
    "select chips",
];

// ── 구현할 부분 ─────────────────────────────────────────────

// ─────────────────────────────────────────────
// TODO(3): 한 줄("coin 100", "select cola", "take" ...)을 Event 로 바꿉니다. 이해 못 하면 None.
//   🐍 Python: match (cmd, args): case ("coin", [a]): ...    (python/main.py: parse_event)
//   ⚙️ C++  : if (cmd == "coin" && parts.size() == 2) ...     (cpp/main.cpp: parse_event)
//   🦀 Rust : (명령어, 인자) 튜플을 한 번에 match 합니다.
//   💡 힌트: line.split_whitespace() 로 쪼개 `parts.next()`; 인자 변환은
//           `a.parse().ok().map(Event::InsertCoin)` — enum variant 는 함수처럼 쓸 수 있어요.
//           "cancel" 처럼 인자가 없어야 하는 명령은 `("cancel", None)` 패턴.
//   ❓ 왜?: 튜플 패턴 덕분에 if/else 사슬 없이 "모양"으로 입력을 분류합니다.
// ─────────────────────────────────────────────
fn parse_event(line: &str) -> Option<Event> {
    todo!("TODO(3): 문자열 -> Option<Event>")
}

// ─────────────────────────────────────────────
// TODO(4): State 를 사람이 읽는 문자열로 출력하는 Display 를 구현하세요.
//   형식: "Idle" / "Collecting(250)" / "Dispensing(Cola, change=50)" / "Refunding(50)" / "OutOfService(kicked)"
//   🐍 Python: match state: case Collecting(cents): ...       (python/main.py: state_str)
//   ⚙️ C++  : std::visit(overloaded{...}, s)                   (cpp/main.cpp: state_str)
//   🦀 Rust : `match self` 로 5개 variant 를 모두 다루고 `write!(f, ...)` 합니다.
//   💡 힌트: 필드가 있는 variant 는 `State::Collecting { cents } => ...` 로 값을 꺼냅니다.
//   ❓ 왜?: `_ =>` 를 쓰지 않으면, 나중에 State 에 variant 가 늘 때 여기서 컴파일 에러가 나서 안전합니다.
// ─────────────────────────────────────────────
impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!("TODO(4): variant 별로 write!")
    }
}

// ─────────────────────────────────────────────
// TODO(5): 상태 전이 함수: (현재 상태, 사건) -> 다음 상태.
//   규칙 (위에서 아래로 먼저 맞는 것이 적용됩니다):
//   1) OutOfService + Repair => Idle           2) OutOfService + 그 밖의 사건 => 그대로
//   3) 어떤 상태든 Kick => OutOfService { reason: "kicked" }
//   4) Idle + InsertCoin(유효한 동전) => Collecting { cents: 그 동전 }
//   5) Collecting + InsertCoin(유효한 동전) => cents 누적 (유효하지 않은 동전은 무시)
//   6) Collecting + Select(item): cents >= item.price() 이면 Dispensing { item, change: cents - price }, 모자라면 그대로
//   7) Collecting + Cancel => Refunding { cents }
//   8) Dispensing 또는 Refunding + Take => Idle
//   9) 그 밖의 모든 조합 => 그대로 (state.clone())
//   🐍 Python: match (state, event): case (Collecting(cents), Select(item)) if ...   (python/main.py: transition)
//   ⚙️ C++  : std::visit(overloaded{...}, state, event)                              (cpp/main.cpp: transition)
//   🦀 Rust : `match (state, event) { ... }` — 튜플 패턴 + `if` 가드 + `|` (또는) 패턴.
//   💡 힌트: 가드 예: `(State::Idle, Event::InsertCoin(c)) if is_valid_coin(*c) => ...`
//           참조를 match 하면 c 는 `&u32` 입니다. 값이 필요하면 `*c`.
//           여러 variant 를 한 팔로: `State::Dispensing { .. } | State::Refunding { .. }` (`..` = 나머지 필드 무시)
//   ❓ 왜?: 상태 머신 전체가 "표 하나"처럼 읽힙니다. 순수 함수라 테스트도 쉽습니다.
// ─────────────────────────────────────────────
fn transition(state: &State, event: &Event) -> State {
    todo!("TODO(5): (state, event) 튜플 match")
}

// ─────────────────────────────────────────────
// TODO(6): 이 사건 때문에 상품이 실제로 손님에게 넘어가면(= Dispensing 상태에서 Take) 그 Item 을 돌려주세요.
//   🐍 Python: isinstance(state, Dispensing) and isinstance(event, Take)   (python/main.py: sale_of)
//   ⚙️ C++  : std::get_if<Dispensing>(&state)                               (cpp/main.cpp: sale_of)
//   🦀 Rust : `let State::Dispensing { item, .. } = state else { return None; };`  (let-else)
//   💡 힌트: 이어서 `*event == Event::Take` 를 확인합니다 (Event 에 PartialEq 가 derive 되어 있어요).
//           `bool::then_some(값)` 이 `Some(값)`/`None` 을 만들어 줍니다.
//   ❓ 왜?: let-else 는 "맞지 않으면 일찍 탈출" 패턴이라 들여쓰기가 깊어지지 않습니다.
// ─────────────────────────────────────────────
fn sale_of(state: &State, event: &Event) -> Option<Item> {
    todo!("TODO(6): let-else")
}

/// 상태와 판매 기록을 가진 자판기.
struct Machine {
    state: State,
    sold: Vec<Item>,
}

impl Machine {
    // ─────────────────────────────────────────────
    // TODO(7): 연관 함수(associated function) — 처음 상태는 Idle, 판매 기록은 빈 Vec.
    //   🐍 Python: __init__                                 (python/main.py: Machine.__init__)
    //   ⚙️ C++  : 멤버 기본값 / 생성자                       (cpp/main.cpp: Machine)
    //   🦀 Rust : self 를 받지 않으므로 `Machine::new()` 로 부릅니다. 구조체 리터럴 `Machine { state: ..., sold: ... }`
    //   💡 힌트: `Vec::new()`
    //   ❓ 왜?: Rust 에는 생성자 문법이 없고 관례적으로 `new` 라는 이름의 연관 함수를 씁니다.
    // ─────────────────────────────────────────────
    fn new() -> Self {
        todo!("TODO(7): Machine 만들기")
    }

    // ─────────────────────────────────────────────
    // TODO(8): `&mut self` 메서드 — 사건 하나를 처리합니다.
    //   (a) sale_of(&self.state, event) 가 Some(item) 이면 self.sold 에 push (`if let`)
    //   (b) self.state = transition(&self.state, event)
    //   🐍 Python: self.state = transition(...)               (python/main.py: Machine.handle)
    //   ⚙️ C++  : state = transition(state, event);            (cpp/main.cpp: Machine::handle)
    //   🦀 Rust : (a) 를 먼저 하세요. 상태가 바뀌고 나면 판매 여부를 알 수 없습니다.
    //   💡 힌트: `if let Some(item) = sale_of(...) { ... }`
    //   ❓ 왜?: `&mut self` 는 "이 메서드가 자기 자신을 바꾼다"는 것을 호출하는 쪽에 알립니다.
    // ─────────────────────────────────────────────
    fn handle(&mut self, event: &Event) {
        todo!("TODO(8): 판매 기록 + 상태 전이")
    }

    // ─────────────────────────────────────────────
    // TODO(9): `&self` 메서드 — 지금까지 판매 금액 합계. (이터레이터는 ch07 에서 자세히 배웁니다.)
    //   🐍 Python: sum(i.price for i in self.sold)            (python/main.py: Machine.revenue)
    //   ⚙️ C++  : for 루프로 합산                              (cpp/main.cpp: Machine::revenue)
    //   🦀 Rust : `self.sold.iter().map(|i| i.price()).sum()` 처럼 한 줄로도, for 루프로도 괜찮아요.
    //   💡 힌트: Item 은 Copy 라서 `i.price()` 를 빌린 참조에서도 바로 부를 수 있어요.
    //   ❓ 왜?: `&self` 는 읽기 전용 → 여러 곳에서 동시에 빌려도 안전합니다.
    // ─────────────────────────────────────────────
    fn revenue(&self) -> u32 {
        todo!("TODO(9): 합계")
    }

    // 제공됨: `self` 를 값으로 받으면 자판기가 "소비(move)"됩니다. 호출한 뒤에는 machine 을 쓸 수 없어요.
    fn into_sold(self) -> Vec<Item> {
        self.sold
    }
}

fn main() {
    let mut machine = Machine::new();
    for (i, line) in SCRIPT.iter().enumerate() {
        let label = match parse_event(line) {
            Some(event) => {
                machine.handle(&event);
                event_str(&event)
            }
            None => format!("? {line}"),
        };
        println!("step {:02}: {:<16} -> {}", i + 1, label, machine.state);
    }
    println!("revenue: {}", machine.revenue());
    let sold = machine.into_sold();
    // 🔬 실험 1: 아래 줄의 주석을 풀어 보세요. (기대: E0382 borrow of moved value: `machine`)
    //   into_sold(self) 가 machine 을 소비했으므로 더 이상 쓸 수 없습니다.
    //   ❓ C++ 의 std::move 후 사용과 무엇이 다른가요? (힌트: C++ 은 컴파일되고 값은 "유효하지만 불명")
    // println!("{}", machine.revenue());
    let names: Vec<&str> = sold.iter().map(|i| i.name()).collect();
    println!("sold: {}", names.join(", "));
}

// 🔬 실험 2: `State` 에 `Broken` variant 를 추가해 보세요 (E0004 non-exhaustive patterns).
//   TODO(4) 의 Display 에서 컴파일 에러가 납니다. 하지만 `transition` 의 마지막 `_ =>` 팔 때문에
//   거기서는 에러가 나지 않아요. 이 차이가 "와일드카드를 언제 쓰는가"의 설계 포인트입니다.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prices() {
        assert_eq!(Item::Cola.price(), 150);
        assert_eq!(Item::Chips.price(), 100);
        assert_eq!(Item::Candy.price(), 70);
    }

    #[test]
    fn from_name_is_case_insensitive() {
        assert_eq!(Item::from_name("Cola"), Some(Item::Cola));
        assert_eq!(Item::from_name("CHIPS"), Some(Item::Chips));
        assert_eq!(Item::from_name("water"), None);
    }

    #[test]
    fn parse_event_variants() {
        assert_eq!(parse_event("coin 100"), Some(Event::InsertCoin(100)));
        assert_eq!(
            parse_event("select candy"),
            Some(Event::Select(Item::Candy))
        );
        assert_eq!(parse_event("cancel"), Some(Event::Cancel));
        assert_eq!(parse_event("take"), Some(Event::Take));
        assert_eq!(parse_event("repair"), Some(Event::Repair));
    }

    #[test]
    fn parse_event_rejects_garbage() {
        assert_eq!(parse_event(""), None);
        assert_eq!(parse_event("dance"), None);
        assert_eq!(parse_event("coin"), None);
        assert_eq!(parse_event("coin abc"), None);
        assert_eq!(parse_event("select water"), None);
        assert_eq!(parse_event("take now"), None);
    }

    #[test]
    fn state_display() {
        assert_eq!(State::Idle.to_string(), "Idle");
        assert_eq!(
            State::Collecting { cents: 250 }.to_string(),
            "Collecting(250)"
        );
        let d = State::Dispensing {
            item: Item::Cola,
            change: 50,
        };
        assert_eq!(d.to_string(), "Dispensing(Cola, change=50)");
        let o = State::OutOfService {
            reason: "kicked".into(),
        };
        assert_eq!(o.to_string(), "OutOfService(kicked)");
    }

    #[test]
    fn coins_accumulate_and_invalid_coin_is_ignored() {
        let s = transition(&State::Idle, &Event::InsertCoin(100));
        assert_eq!(s, State::Collecting { cents: 100 });
        let s = transition(&s, &Event::InsertCoin(7));
        assert_eq!(s, State::Collecting { cents: 100 });
        let s = transition(&s, &Event::InsertCoin(50));
        assert_eq!(s, State::Collecting { cents: 150 });
    }

    #[test]
    fn select_needs_enough_money() {
        let s = State::Collecting { cents: 100 };
        assert_eq!(transition(&s, &Event::Select(Item::Cola)), s);
        let s = State::Collecting { cents: 200 };
        assert_eq!(
            transition(&s, &Event::Select(Item::Cola)),
            State::Dispensing {
                item: Item::Cola,
                change: 50
            }
        );
    }

    #[test]
    fn cancel_refunds_then_take_goes_idle() {
        let s = transition(&State::Collecting { cents: 60 }, &Event::Cancel);
        assert_eq!(s, State::Refunding { cents: 60 });
        assert_eq!(transition(&s, &Event::Take), State::Idle);
    }

    #[test]
    fn kick_breaks_and_repair_fixes() {
        let broken = transition(&State::Collecting { cents: 10 }, &Event::Kick);
        assert_eq!(
            broken,
            State::OutOfService {
                reason: "kicked".into()
            }
        );
        assert_eq!(transition(&broken, &Event::InsertCoin(100)), broken);
        assert_eq!(transition(&broken, &Event::Repair), State::Idle);
    }

    #[test]
    fn machine_tracks_sales() {
        let mut m = Machine::new();
        for line in [
            "coin 100",
            "coin 100",
            "select cola",
            "take",
            "coin 100",
            "select chips",
            "take",
        ] {
            m.handle(&parse_event(line).unwrap());
        }
        assert_eq!(m.revenue(), 250);
        assert_eq!(m.into_sold(), vec![Item::Cola, Item::Chips]);
    }
}
