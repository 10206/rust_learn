"""Ch16. HTTP 클라이언트 & 실전 에러 처리 (Python 참조 구현)

표준 라이브러리 urllib 만 사용합니다. 목 서버(tools/mock_server.py)에 말을 겁니다.
환경 변수 API_BASE_URL 로 주소를 바꿀 수 있어요 (기본 http://127.0.0.1:8787).
"""

import json
import os
import socket
import time
import urllib.error
import urllib.request
from dataclasses import dataclass

DEFAULT_BASE_URL = "http://127.0.0.1:8787"


# ───────────────────────── 에러 타입 (Rust: thiserror enum ApiError) ─────────────────────────

class ApiError(Exception):
    """라이브러리가 던지는 에러의 공통 부모. Rust 에서는 enum 하나로 표현합니다."""

    def retryable(self) -> bool:
        return False


class NotFound(ApiError):
    def __init__(self, id: int):
        super().__init__(f"todo {id} not found")
        self.id = id


class HttpStatus(ApiError):
    def __init__(self, status: int):
        super().__init__(f"HTTP {status}")
        self.status = status

    def retryable(self) -> bool:
        return self.status >= 500


class Timeout(ApiError):
    def __init__(self):
        super().__init__("request timed out")

    def retryable(self) -> bool:
        return True


class NetworkError(ApiError):
    def __init__(self, cause: Exception):
        super().__init__(f"network error: {cause}")


class DecodeError(ApiError):
    def __init__(self, cause: Exception):
        super().__init__(f"decode error: {cause}")


# ───────────────────────── 데이터 ─────────────────────────

@dataclass
class Todo:
    id: int
    title: str
    done: bool


def backoff_delay(attempt: int, base: float = 0.05) -> float:
    """attempt=1 -> base, 2 -> 2*base, 3 -> 4*base ... (지수 백오프)"""
    return base * (2 ** (attempt - 1))


# ───────────────────────── 클라이언트 ─────────────────────────

class TodoClient:
    def __init__(self, base_url: str, timeout: float = 2.0):
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout

    def _request(self, method: str, path: str, body: dict | None = None):
        data = json.dumps(body).encode() if body is not None else None
        req = urllib.request.Request(self.base_url + path, data=data, method=method,
                                     headers={"Content-Type": "application/json"})
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                raw = resp.read().decode("utf-8")
        except urllib.error.HTTPError as e:      # 4xx / 5xx 는 urllib 에서 '예외'로 옵니다.
            raise HttpStatus(e.code) from e
        except (socket.timeout, TimeoutError) as e:
            raise Timeout() from e
        except urllib.error.URLError as e:
            if isinstance(e.reason, (socket.timeout, TimeoutError)):
                raise Timeout() from e
            raise NetworkError(e) from e
        try:
            return json.loads(raw)
        except ValueError as e:
            raise DecodeError(e) from e

    @staticmethod
    def _todo(obj: dict) -> Todo:
        try:
            return Todo(int(obj["id"]), str(obj["title"]), bool(obj["done"]))
        except (KeyError, TypeError, ValueError) as e:
            raise DecodeError(e) from e

    def reset(self) -> None:
        self._request("POST", "/reset", {})

    def list_todos(self) -> list[Todo]:
        return [self._todo(o) for o in self._request("GET", "/todos")]

    def create_todo(self, title: str) -> Todo:
        return self._todo(self._request("POST", "/todos", {"title": title}))

    def get_todo(self, id: int) -> Todo:
        try:
            return self._todo(self._request("GET", f"/todos/{id}"))
        except HttpStatus as e:
            if e.status == 404:
                raise NotFound(id) from e   # 404 는 '의미 있는' 에러로 번역
            raise

    def get_flaky(self) -> dict:
        return self._request("GET", "/flaky")

    def get_slow(self, ms: int) -> dict:
        return self._request("GET", f"/slow?ms={ms}")


def retry_with_backoff(max_attempts: int, base: float, op):
    """op(attempt) 를 최대 max_attempts 번 시도. 재시도 가능한 에러일 때만 쉬었다가 다시 합니다."""
    attempt = 1
    while True:
        try:
            return op(attempt)
        except ApiError as e:
            if not e.retryable() or attempt >= max_attempts:
                raise
            time.sleep(backoff_delay(attempt, base))
            attempt += 1


# ───────────────────────── main ─────────────────────────

def main() -> None:
    base_url = os.environ.get("API_BASE_URL", DEFAULT_BASE_URL)
    client = TodoClient(base_url, timeout=2.0)

    print("== reset ==")
    client.reset()
    print("reset ok")

    print("== create ==")
    for title in ("Learn Rust", "Write tests", "Ship it"):
        t = client.create_todo(title)
        print(f'created #{t.id} "{t.title}" done={str(t.done).lower()}')

    print("== list ==")
    for t in client.list_todos():
        print(f"#{t.id} [{'x' if t.done else ' '}] {t.title}")

    print("== get ==")
    t = client.get_todo(2)
    print(f'#{t.id} "{t.title}"')

    print("== not found ==")
    try:
        client.get_todo(999)
    except ApiError as e:
        # Rust: anyhow 의 .context("failed to fetch todo 999") 와 같은 효과
        print(f"error: failed to fetch todo 999: {e}")
        print(f"  is NotFound? {'yes' if isinstance(e, NotFound) else 'no'}")

    print("== invalid body ==")
    try:
        client.create_todo("")
    except ApiError as e:
        print(f"error: {e}")

    print("== flaky ==")

    def flaky_op(attempt: int):
        try:
            r = client.get_flaky()
        except ApiError as e:
            print(f"attempt {attempt}: {e}")
            raise
        print(f"attempt {attempt}: ok (server saw {r['attempts']} calls)")
        return r

    retry_with_backoff(5, 0.05, flaky_op)

    print("== timeout ==")
    try:
        TodoClient(base_url, timeout=0.2).get_slow(1000)
    except ApiError as e:
        print(f"error: {e}")


if __name__ == "__main__":
    main()
