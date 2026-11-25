#!/usr/bin/env python3
"""rust_learn 용 로컬 목(mock) HTTP 서버 (표준 라이브러리만 사용).

실행:  python3 tools/mock_server.py --port 8787

ch16 (Todo REST API)
  GET  /health            -> 200 {"status":"ok"}
  POST /reset             -> 200 {"status":"reset"}   (todo, flaky 카운터, 메시지 카운터 초기화)
  GET  /todos             -> 200 [ {id,title,done}, ... ]
  POST /todos             -> 201 {id,title,done}      (본문: {"title": "..."}; 잘못되면 400)
  GET  /todos/<id>        -> 200 {...} | 404 {"error": "todo <id> not found"}
  GET  /flaky             -> 처음 2번은 500, 3번째부터 200 (reset 이후 다시 셈)
  GET  /slow?ms=N         -> N 밀리초 뒤에 200 (타임아웃 실험용)

ch17 (Claude Messages API 흉내)
  POST /v1/messages       -> 공식 Python `anthropic` SDK 가 말을 걸 수 있을 만큼 비슷하게 응답.
                             "모델"은 대본(script)대로 움직이는 결정적(deterministic) 가짜입니다.

특별 트리거 (첫 user 메시지 텍스트에 포함):
  MOCK_REFUSAL     -> stop_reason "refusal" 로 응답
  MOCK_MAX_TOKENS  -> stop_reason "max_tokens" 로 응답
  MOCK_LOOP        -> 끝없이 tool_use 만 요청 (max-iterations 방어 테스트용)
"""

import argparse
import json
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

LOCK = threading.Lock()
STATE = {}


def reset_state():
    STATE.clear()
    STATE.update(todos=[], next_id=1, flaky_calls=0, msg_counter=0)


reset_state()

MIN_NO_SAMPLING_VERSION = "2026-10-21"
EFFORTS = {"low", "medium", "high", "xhigh", "max"}


class ApiProblem(Exception):
    """Anthropic 스타일 에러로 변환되는 예외."""

    def __init__(self, status, etype, message):
        super().__init__(message)
        self.status, self.etype, self.message = status, etype, message


# ───────────────────────── 가짜 "모델" (ch17) ─────────────────────────

def block_text(content):
    """tool_result.content (문자열 또는 블록 리스트) -> 문자열"""
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        return "".join(b.get("text", "") for b in content if isinstance(b, dict))
    return ""


def collect_tool_results(messages):
    """tool_use_id -> (tool 이름, 결과 텍스트, is_error)"""
    names = {}
    out = {}
    for m in messages:
        if not isinstance(m.get("content"), list):
            continue
        for b in m["content"]:
            if b.get("type") == "tool_use":
                names[b["id"]] = b["name"]
            elif b.get("type") == "tool_result":
                out[b["tool_use_id"]] = (
                    names.get(b["tool_use_id"], "?"),
                    block_text(b.get("content")),
                    bool(b.get("is_error")),
                )
    return out


def first_user_text(messages):
    c = messages[0].get("content")
    return c if isinstance(c, str) else block_text(c)


def count_tool_uses(messages):
    n = 0
    for m in messages:
        if m["role"] == "assistant" and isinstance(m["content"], list):
            n += sum(1 for b in m["content"] if b.get("type") == "tool_use")
    return n


def script_reply(req, counter):
    """대화 상태(assistant 메시지 개수)만 보고 다음 응답 content 를 정한다."""
    messages = req["messages"]
    declared = {t.get("name") for t in req.get("tools", []) if isinstance(t, dict)}
    n_assistant = sum(1 for m in messages if m["role"] == "assistant")
    first = first_user_text(messages)
    sig = f"mock_signature_{counter}"
    thinking = {"type": "thinking", "thinking": "", "signature": sig}
    next_id = count_tool_uses(messages)

    def tool(name, inp):
        nonlocal next_id
        next_id += 1
        return {"type": "tool_use", "id": f"toolu_mock_{next_id}", "name": name, "input": inp}

    if "MOCK_REFUSAL" in first:
        return [], "refusal"
    if "MOCK_MAX_TOKENS" in first:
        return [thinking, {"type": "text", "text": "This answer is cut off because"}], "max_tokens"
    if not declared:
        return [thinking, {"type": "text", "text": "I have no tools, so I can only answer directly."}], "end_turn"
    if "MOCK_LOOP" in first:
        return [thinking, {"type": "text", "text": "Let me calculate once more."},
                tool("calculator", {"expression": "1+1"})], "tool_use"

    results = collect_tool_results(messages)

    def by_name(name):
        for tid in sorted(results):
            if results[tid][0] == name:
                return results[tid]
        return None

    if n_assistant == 0:
        # 병렬 tool_use 2개
        return [thinking,
                {"type": "text", "text": "I will read the note and run the calculation in parallel."},
                tool("read_note", {"name": "notes.txt"}),
                tool("calculator", {"expression": "(12+30)*2"})], "tool_use"
    if n_assistant == 1:
        note = by_name("read_note")
        note_text = note[1] if note else ""
        # word_count + 일부러 data/ 밖의 파일 읽기 시도 (-> is_error)
        return [thinking,
                {"type": "text", "text": "Now I will count the words and also try a file outside data/."},
                tool("word_count", {"text": note_text}),
                tool("read_note", {"name": "../secret.txt"})], "tool_use"
    if n_assistant == 2:
        return [thinking,
                {"type": "text", "text": "One read was rejected, so I will not retry it. Checking the weather instead."},
                tool("get_weather", {"city": "Seoul"})], "tool_use"

    wc = by_name("word_count")
    calc = by_name("calculator")
    weather = by_name("get_weather")
    rejected = None
    for tid in sorted(results):
        name, text, is_err = results[tid]
        if name == "read_note" and is_err:
            rejected = text
    lines = [
        "Summary:",
        f"- notes.txt has {wc[1] if wc else '?'} words.",
        f"- (12+30)*2 = {calc[1] if calc else '?'}.",
        f"- Reading ../secret.txt failed: {rejected if rejected else 'n/a'}",
        f"- Weather in Seoul: {weather[1] if weather else '?'}.",
    ]
    return [thinking, {"type": "text", "text": "\n".join(lines)}], "end_turn"


def validate_messages_request(headers, req):
    """실제 API 가 하는 검사 중 학습에 도움이 되는 것들."""
    # 실제 API 는 x-api-key 헤더를 씁니다. (최신 Python SDK 는 Authorization: Bearer 로 보내므로 둘 다 허용)
    auth = headers.get("authorization", "")
    if not headers.get("x-api-key") and not auth.lower().startswith("bearer "):
        raise ApiProblem(401, "authentication_error", "x-api-key header is required")
    version = headers.get("anthropic-version")
    if not version:
        raise ApiProblem(400, "invalid_request_error", "anthropic-version: header is required")
    if not isinstance(req, dict):
        raise ApiProblem(400, "invalid_request_error", "Request body must be a JSON object")
    for field in ("model", "max_tokens", "messages"):
        if field not in req:
            raise ApiProblem(400, "invalid_request_error", f"{field}: Field required")
    if not isinstance(req["model"], str) or not req["model"]:
        raise ApiProblem(400, "invalid_request_error", "model: Input should be a non-empty string")
    if not isinstance(req["max_tokens"], int) or isinstance(req["max_tokens"], bool) or req["max_tokens"] < 1:
        raise ApiProblem(400, "invalid_request_error", "max_tokens: Input should be a positive integer")
    if version >= MIN_NO_SAMPLING_VERSION:
        for p in ("temperature", "top_p", "top_k"):
            if p in req:
                raise ApiProblem(400, "invalid_request_error",
                                 f"`{p}` may not be set on this model/API version. Remove it from the request.")
    cfg = req.get("output_config")
    if cfg is not None:
        if not isinstance(cfg, dict) or cfg.get("effort", "medium") not in EFFORTS:
            raise ApiProblem(400, "invalid_request_error", "output_config.effort: invalid value")
    msgs = req["messages"]
    if not isinstance(msgs, list) or not msgs:
        raise ApiProblem(400, "invalid_request_error", "messages: at least one message is required")
    if "tools" in req and not isinstance(req["tools"], list):
        raise ApiProblem(400, "invalid_request_error", "tools: Input should be a valid list")
    for i, t in enumerate(req.get("tools", [])):
        if not isinstance(t, dict) or "name" not in t or "input_schema" not in t:
            raise ApiProblem(400, "invalid_request_error",
                             f"tools.{i}: `name` and `input_schema` are required")

    expect = "user"
    for i, m in enumerate(msgs):
        if not isinstance(m, dict) or m.get("role") not in ("user", "assistant"):
            raise ApiProblem(400, "invalid_request_error", f"messages.{i}.role: Input should be 'user' or 'assistant'")
        if m["role"] != expect:
            raise ApiProblem(400, "invalid_request_error",
                             f"messages.{i}.role: roles must alternate between 'user' and 'assistant', starting with 'user'")
        expect = "assistant" if expect == "user" else "user"
        c = m.get("content")
        if not isinstance(c, (str, list)) or c == "" or c == []:
            raise ApiProblem(400, "invalid_request_error", f"messages.{i}.content: must be non-empty")
    if msgs[-1]["role"] != "user":
        raise ApiProblem(400, "invalid_request_error",
                         "messages: the last message must have role 'user' (assistant prefill is not supported)")

    # tool_use <-> tool_result 짝 맞추기 (이 검사가 가장 흔한 실수를 잡아 줍니다)
    for i, m in enumerate(msgs):
        c = m["content"]
        if not isinstance(c, list):
            continue
        if m["role"] == "assistant":
            for j, b in enumerate(c):
                if b.get("type") == "thinking" and not b.get("signature"):
                    raise ApiProblem(400, "invalid_request_error",
                                     f"messages.{i}.content.{j}.thinking.signature: Field required")
                if b.get("type") == "thinking" and not str(b["signature"]).startswith("mock_signature_"):
                    raise ApiProblem(400, "invalid_request_error",
                                     f"messages.{i}.content.{j}: Invalid `signature` in `thinking` block")
            ids = [b["id"] for b in c if b.get("type") == "tool_use"]
            if ids and c[0].get("type") not in ("thinking", "redacted_thinking"):
                raise ApiProblem(400, "invalid_request_error",
                                 f"messages.{i}.content.0.type: Expected `thinking` or `redacted_thinking`, but found "
                                 f"`{c[0].get('type')}`. When thinking is enabled, an assistant turn that contains "
                                 f"`tool_use` must be passed back unchanged, starting with its thinking block.")
            if ids:
                if i + 1 >= len(msgs):
                    raise ApiProblem(400, "invalid_request_error",
                                     f"messages.{i}: `tool_use` ids were found without `tool_result` blocks "
                                     f"immediately after: {', '.join(ids)}.")
                nxt = msgs[i + 1]["content"]
                got = {b.get("tool_use_id") for b in nxt if isinstance(b, dict) and b.get("type") == "tool_result"} \
                    if isinstance(nxt, list) else set()
                missing = [x for x in ids if x not in got]
                if missing:
                    raise ApiProblem(400, "invalid_request_error",
                                     f"messages.{i + 1}: `tool_use` ids were found without `tool_result` blocks "
                                     f"immediately after: {', '.join(missing)}. Each `tool_use` block must have a "
                                     f"corresponding `tool_result` block in the next message.")
        else:
            prev_ids = set()
            if i > 0 and isinstance(msgs[i - 1]["content"], list):
                prev_ids = {b["id"] for b in msgs[i - 1]["content"] if b.get("type") == "tool_use"}
            seen_non_result = False
            for j, b in enumerate(c):
                if b.get("type") == "tool_result":
                    if seen_non_result:
                        raise ApiProblem(400, "invalid_request_error",
                                         f"messages.{i}.content.{j}: `tool_result` blocks must come first in the content")
                    if b.get("tool_use_id") not in prev_ids:
                        raise ApiProblem(400, "invalid_request_error",
                                         f"messages.{i}.content.{j}: unexpected `tool_use_id` found in `tool_result` "
                                         f"blocks: {b.get('tool_use_id')}. Each `tool_result` block must have a "
                                         f"corresponding `tool_use` block in the previous message.")
                else:
                    seen_non_result = True


def handle_messages(headers, raw):
    try:
        req = json.loads(raw.decode("utf-8")) if raw else None
    except (ValueError, UnicodeDecodeError):
        raise ApiProblem(400, "invalid_request_error", "Request body is not valid JSON")
    validate_messages_request(headers, req)
    with LOCK:
        STATE["msg_counter"] += 1
        counter = STATE["msg_counter"]
    content, stop = script_reply(req, counter)
    n_out = sum(len(json.dumps(b)) for b in content) // 4
    return {
        "id": f"msg_mock_{counter}",
        "type": "message",
        "role": "assistant",
        "model": req["model"],
        "content": content,
        "stop_reason": stop,
        "stop_sequence": None,
        "usage": {"input_tokens": 100 * len(req["messages"]), "output_tokens": n_out},
    }


# ───────────────────────── HTTP 핸들러 ─────────────────────────

class Handler(BaseHTTPRequestHandler):
    server_version = "rust-learn-mock/1.0"

    def log_message(self, fmt, *args):  # 조용히
        pass

    def send_json(self, status, obj, extra_headers=None):
        body = json.dumps(obj).encode("utf-8")
        try:
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            for k, v in (extra_headers or {}).items():
                self.send_header(k, v)
            self.end_headers()
            self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            pass  # 클라이언트가 타임아웃으로 먼저 끊은 경우

    def anthropic_error(self, status, etype, message):
        self.send_json(status, {"type": "error", "error": {"type": etype, "message": message}},
                       {"request-id": "req_mock"})

    def read_body(self):
        n = int(self.headers.get("Content-Length") or 0)
        return self.rfile.read(n) if n else b""

    # --- GET ---
    def do_GET(self):
        url = urlparse(self.path)
        path = url.path.rstrip("/") or "/"
        if path == "/health":
            return self.send_json(200, {"status": "ok"})
        if path == "/todos":
            with LOCK:
                return self.send_json(200, list(STATE["todos"]))
        if path.startswith("/todos/"):
            tail = path[len("/todos/"):]
            try:
                tid = int(tail)
            except ValueError:
                return self.send_json(400, {"error": f"invalid id: {tail}"})
            with LOCK:
                for t in STATE["todos"]:
                    if t["id"] == tid:
                        return self.send_json(200, t)
            return self.send_json(404, {"error": f"todo {tid} not found"})
        if path == "/flaky":
            with LOCK:
                STATE["flaky_calls"] += 1
                n = STATE["flaky_calls"]
            if n <= 2:
                return self.send_json(500, {"error": f"flaky failure {n}"})
            return self.send_json(200, {"status": "ok", "attempts": n})
        if path == "/slow":
            try:
                ms = int(parse_qs(url.query).get("ms", ["1000"])[0])
            except ValueError:
                ms = 1000
            time.sleep(min(ms, 5000) / 1000)
            return self.send_json(200, {"status": "slow ok", "ms": ms})
        if path.startswith("/v1/"):
            return self.anthropic_error(404, "not_found_error", f"GET {path} is not supported by the mock")
        return self.send_json(404, {"error": f"no route: GET {path}"})

    # --- POST ---
    def do_POST(self):
        url = urlparse(self.path)
        path = url.path.rstrip("/") or "/"
        raw = self.read_body()
        if path == "/reset":
            with LOCK:
                reset_state()
            return self.send_json(200, {"status": "reset"})
        if path == "/todos":
            try:
                body = json.loads(raw.decode("utf-8"))
            except (ValueError, UnicodeDecodeError):
                return self.send_json(400, {"error": "body must be valid JSON"})
            title = body.get("title") if isinstance(body, dict) else None
            if not isinstance(title, str) or not title.strip():
                return self.send_json(400, {"error": "title must be a non-empty string"})
            with LOCK:
                todo = {"id": STATE["next_id"], "title": title, "done": bool(body.get("done", False))}
                STATE["next_id"] += 1
                STATE["todos"].append(todo)
            return self.send_json(201, todo)
        if path == "/v1/messages":
            try:
                resp = handle_messages(self.headers, raw)
            except ApiProblem as e:
                return self.anthropic_error(e.status, e.etype, e.message)
            return self.send_json(200, resp, {"request-id": "req_mock"})
        if path.startswith("/v1/"):
            return self.anthropic_error(404, "not_found_error", f"POST {path} is not supported by the mock")
        return self.send_json(404, {"error": f"no route: POST {path}"})


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--port", type=int, default=8787)
    ap.add_argument("--host", default="127.0.0.1")
    args = ap.parse_args()
    srv = ThreadingHTTPServer((args.host, args.port), Handler)
    srv.daemon_threads = True
    try:
        srv.serve_forever()
    except KeyboardInterrupt:
        pass


if __name__ == "__main__":
    main()
