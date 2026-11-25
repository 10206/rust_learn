//! Ch05 정답: 자판기 상태 머신
//!
//! 실행: `cargo run -q -p ch05_structs_enums --example solution`
//! 테스트: `cargo test -p ch05_structs_enums --example solution`

use std::fmt;

// ── 데이터 정의 ─────────────────────────────────────────────

/// 상품 종류. 데이터가 없는 enum 은 `Copy` 까지 붙여 가볍게 다룹니다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Cola,
    Chips,
    Candy,
}

impl Item {
    /// match 는 모든 variant 를 다뤄야 컴파일됩니다 (exhaustive).
    fn price(self) -> u32 {
        match self {
            Item::Cola => 150,
            Item::Chips => 100,
            Item::Candy => 70,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Item::Cola => "Cola",
            Item::Chips => "Chips",
            Item::Candy => "Candy",
        }
    }

    /// 문자열 -> Option<Item>. "없음"은 null 이 아니라 `None` 입니다.
    fn from_name(s: &str) -> Option<Item> {
        match s.to_lowercase().as_str() {
            "cola" => Some(Item::Cola),
            "chips" => Some(Item::Chips),
            "candy" => Some(Item::Candy),
            _ => None,
        }
    }
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

// ── 구현 ────────────────────────────────────────────────────

/// 한 줄("coin 100", "select cola", "take" ...)을 Event 로 바꿉니다.
fn parse_event(line: &str) -> Option<Event> {
    let mut parts = line.split_whitespace();
    let cmd = parts.next()?;
    let arg = parts.next();
    match (cmd, arg) {
        ("coin", Some(a)) => a.parse().ok().map(Event::InsertCoin),
        ("select", Some(a)) => Item::from_name(a).map(Event::Select),
        ("cancel", None) => Some(Event::Cancel),
        ("take", None) => Some(Event::Take),
        ("kick", None) => Some(Event::Kick),
        ("repair", None) => Some(Event::Repair),
        _ => None,
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `_ =>` 를 쓰지 않았기 때문에 State 에 variant 를 추가하면 여기서 컴파일 에러가 납니다.
        match self {
            State::Idle => write!(f, "Idle"),
            State::Collecting { cents } => write!(f, "Collecting({cents})"),
            State::Dispensing { item, change } => {
                write!(f, "Dispensing({}, change={change})", item.name())
            }
            State::Refunding { cents } => write!(f, "Refunding({cents})"),
            State::OutOfService { reason } => write!(f, "OutOfService({reason})"),
        }
    }
}

/// 상태 전이 함수: (현재 상태, 사건) -> 다음 상태. 순수 함수라 테스트하기 쉽습니다.
fn transition(state: &State, event: &Event) -> State {
    match (state, event) {
        (State::OutOfService { .. }, Event::Repair) => State::Idle,
        (State::OutOfService { .. }, _) => state.clone(),
        (_, Event::Kick) => State::OutOfService {
            reason: "kicked".to_string(),
        },
        (State::Idle, Event::InsertCoin(c)) if is_valid_coin(*c) => State::Collecting { cents: *c },
        (State::Collecting { cents }, Event::InsertCoin(c)) if is_valid_coin(*c) => {
            State::Collecting { cents: cents + c }
        }
        (State::Collecting { cents }, Event::Select(item)) if *cents >= item.price() => {
            State::Dispensing {
                item: *item,
                change: cents - item.price(),
            }
        }
        (State::Collecting { cents }, Event::Cancel) => State::Refunding { cents: *cents },
        (State::Dispensing { .. } | State::Refunding { .. }, Event::Take) => State::Idle,
        // 나머지 조합은 "아무 일도 안 일어남"
        _ => state.clone(),
    }
}

/// 이 사건으로 상품이 실제로 손님에게 넘어가면 그 상품을 돌려줍니다.
fn sale_of(state: &State, event: &Event) -> Option<Item> {
    // let-else: 패턴이 맞지 않으면 else 블록에서 반드시 탈출해야 합니다.
    let State::Dispensing { item, .. } = state else {
        return None;
    };
    (*event == Event::Take).then_some(*item)
}

/// 상태와 판매 기록을 가진 자판기.
struct Machine {
    state: State,
    sold: Vec<Item>,
}

impl Machine {
    /// 연관 함수(associated function): self 가 없고 `Machine::new()` 로 부릅니다.
    fn new() -> Self {
        Machine {
            state: State::Idle,
            sold: Vec::new(),
        }
    }

    /// `&mut self`: 자기 자신을 수정합니다.
    fn handle(&mut self, event: &Event) {
        if let Some(item) = sale_of(&self.state, event) {
            self.sold.push(item);
        }
        self.state = transition(&self.state, event);
    }

    /// `&self`: 읽기만 합니다.
    fn revenue(&self) -> u32 {
        self.sold.iter().map(|i| i.price()).sum()
    }

    /// `self`: 자판기를 소비(move)하고 판매 목록만 꺼내 줍니다.
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
    let names: Vec<&str> = sold.iter().map(|i| i.name()).collect();
    println!("sold: {}", names.join(", "));
}

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
