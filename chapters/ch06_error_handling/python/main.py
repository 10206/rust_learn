"""Ch06 설정 파일 파서 (Python 참조 구현).

Python 은 예외(exception)로 실패를 알립니다. 함수 시그니처만 봐서는 "어떤 에러가 날 수 있는지" 알 수 없어요.
Rust 는 반환 타입 Result<T, E> 에 그것이 드러납니다.
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class Config:
    name: str
    port: int
    debug: bool
    max_retries: int


# ── 에러 타입: Rust 의 enum ConfigError 에 해당 (예외 클래스 계층) ──
class ConfigError(Exception):
    """모든 설정 에러의 부모. str(e) 가 Rust 의 Display 와 같은 문장을 돌려줍니다."""


class MissingEquals(ConfigError):
    def __init__(self, line: int) -> None:
        super().__init__(f"line {line}: expected 'key = value'")


class UnknownKey(ConfigError):
    def __init__(self, line: int, key: str) -> None:
        super().__init__(f"line {line}: unknown key '{key}'")


class DuplicateKey(ConfigError):
    def __init__(self, line: int, key: str) -> None:
        super().__init__(f"line {line}: duplicate key '{key}'")


class InvalidValue(ConfigError):
    def __init__(self, line: int, key: str, reason: str) -> None:
        super().__init__(f"line {line}: invalid value for '{key}': {reason}")


class MissingKey(ConfigError):
    def __init__(self, key: str) -> None:
        super().__init__(f"missing required key '{key}'")


class ConfigIoError(ConfigError):
    def __init__(self) -> None:
        super().__init__("cannot read config file")


def show(cfg: Config) -> str:
    debug = "true" if cfg.debug else "false"
    return f"name={cfg.name} port={cfg.port} debug={debug} max_retries={cfg.max_retries}"


BAD_INPUTS = [
    ("no equals", "name = x\nport 80\n"),
    ("unknown key", "name = x\nport = 80\ncolour = red\n"),
    ("port too big", "name = x\nport = 70000\n"),
    ("port not number", "name = x\nport = eighty\n"),
    ("negative port", "name = x\nport = -1\n"),
    ("bad bool", "name = x\nport = 80\ndebug = maybe\n"),
    ("retries too big", "name = x\nport = 80\nmax_retries = 300\n"),
    ("missing name", "# only a port\nport = 80\n"),
    ("duplicate", "name = a\nname = b\n"),
    ("empty name", "name =\nport = 1\n"),
]


def parse_line(line_no: int, raw: str) -> tuple[str, str] | None:
    line = raw.strip()
    if not line or line.startswith("#"):
        return None
    if "=" not in line:
        raise MissingEquals(line_no)  # Rust: Err(ConfigError::MissingEquals{..})
    key, _, value = line.partition("=")
    return key.strip(), value.strip()


def parse_number(line: int, key: str, value: str, max_value: int) -> int:
    if not value or not (value.isascii() and value.isdigit()):
        raise InvalidValue(line, key, "not a valid number")
    n = int(value)
    if n > max_value:
        raise InvalidValue(line, key, f"out of range (0-{max_value})")
    return n


def parse_bool(line: int, key: str, value: str) -> bool:
    if value == "true":
        return True
    if value == "false":
        return False
    raise InvalidValue(line, key, "expected true or false")


def parse_config(text: str) -> Config:
    name: str | None = None
    port: int | None = None
    debug: bool | None = None
    max_retries: int | None = None

    for idx, raw in enumerate(text.splitlines()):
        line_no = idx + 1
        # 예외는 자동으로 위로 전파됩니다 (Rust 에서는 `?` 를 직접 적어야 전파).
        parsed = parse_line(line_no, raw)
        if parsed is None:
            continue
        key, value = parsed
        match key:
            case "name":
                if name is not None:
                    raise DuplicateKey(line_no, key)
                if not value:
                    raise InvalidValue(line_no, key, "must not be empty")
                name = value
            case "port":
                if port is not None:
                    raise DuplicateKey(line_no, key)
                port = parse_number(line_no, key, value, 65535)
            case "debug":
                if debug is not None:
                    raise DuplicateKey(line_no, key)
                debug = parse_bool(line_no, key, value)
            case "max_retries":
                if max_retries is not None:
                    raise DuplicateKey(line_no, key)
                max_retries = parse_number(line_no, key, value, 255)
            case _:
                raise UnknownKey(line_no, key)

    if name is None:
        raise MissingKey("name")
    if port is None:
        raise MissingKey("port")
    return Config(
        name=name,
        port=port,
        debug=debug if debug is not None else False,
        max_retries=max_retries if max_retries is not None else 3,
    )


def load_config(path: str) -> Config:
    try:
        with open(path, encoding="utf-8") as f:
            text = f.read()
    except OSError as e:
        # Rust 의 `impl From<io::Error> for ConfigError` + `?` 에 해당하는 수동 변환
        raise ConfigIoError() from e
    return parse_config(text)


def port_or_default(text: str) -> int:
    try:
        return parse_config(text).port
    except ConfigError:
        return 8080


def main() -> None:
    for path in ("data/app.conf", "data/missing.conf"):
        try:
            print(f"loaded: {show(load_config(path))}")
        except ConfigError as e:
            print(f"error: {e}")

    print("-- bad inputs --")
    for label, text in BAD_INPUTS:
        try:
            print(f"{label:<16} ok?! {show(parse_config(text))}")
        except ConfigError as e:
            print(f"{label:<16} error: {e}")

    print("-- defaults --")
    minimal = "name = tiny\nport = 9000\n"
    try:
        print(f"minimal: {show(parse_config(minimal))}")
    except ConfigError as e:
        print(f"error: {e}")
    print(f"port_or_default(good) = {port_or_default(minimal)}")
    print(f"port_or_default(bad)  = {port_or_default('port = x')}")


if __name__ == "__main__":
    main()
