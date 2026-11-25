#!/usr/bin/env bash
# 사용법: scripts/compare.sh <chNN | all> [--solution]
# 한 챕터의 Python / C++ / Rust 실행 결과(stdout)를 비교합니다.
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
PORT=8787

if [ -t 1 ]; then
  RED=$'\033[31m'; GRN=$'\033[32m'; YEL=$'\033[33m'; BLD=$'\033[1m'; RST=$'\033[0m'
else
  RED=""; GRN=""; YEL=""; BLD=""; RST=""
fi

CH=""; SOL=0
for a in "$@"; do
  case "$a" in
    --solution) SOL=1 ;;
    -h|--help) echo "사용법: scripts/compare.sh <chNN|all> [--solution]"; exit 0 ;;
    *) CH="$a" ;;
  esac
done
if [ -z "$CH" ]; then
  echo "사용법: scripts/compare.sh <chNN|all> [--solution]" >&2
  exit 2
fi

# 모든 챕터 반복
if [ "$CH" = "all" ]; then
  rc=0
  for d in "$REPO"/chapters/ch[0-9][0-9]_*/; do
    [ -d "$d" ] || continue
    n="$(basename "$d" | cut -c1-4)"
    if [ "$SOL" = 1 ]; then "$0" "$n" --solution || rc=1; else "$0" "$n" || rc=1; fi
    echo
  done
  exit $rc
fi

# 챕터 디렉터리 찾기 (ch03 / ch03_ownership 모두 허용)
CHDIR=""
for d in "$REPO"/chapters/"$CH"_* "$REPO"/chapters/"$CH"; do
  if [ -d "$d" ]; then CHDIR="$d"; break; fi
done
if [ -z "$CHDIR" ]; then
  echo "${RED}챕터를 찾을 수 없어요: $CH${RST}" >&2
  exit 2
fi
CHNAME="$(basename "$CHDIR")"
cd "$CHDIR" || exit 2

TMP="$(mktemp -d "${TMPDIR:-/tmp}/compare.XXXXXX")"
MOCK_PID=""
stop_mock() {
  if [ -n "$MOCK_PID" ]; then
    kill "$MOCK_PID" 2>/dev/null
    wait "$MOCK_PID" 2>/dev/null
    MOCK_PID=""
  fi
}
cleanup() { stop_mock; rm -rf "$TMP"; }
trap cleanup EXIT
trap 'exit 130' INT TERM

start_mock() {
  [ -f USES_MOCK_SERVER ] || return 0
  stop_mock
  if [ ! -f "$REPO/tools/mock_server.py" ]; then
    echo "${RED}tools/mock_server.py 가 없어요.${RST}" >&2
    return 1
  fi
  python3 "$REPO/tools/mock_server.py" --port "$PORT" >"$TMP/mock.out" 2>"$TMP/mock.err" &
  MOCK_PID=$!
  local i=0
  while [ $i -lt 50 ]; do
    if python3 - "$PORT" <<'PY' 2>/dev/null
import sys, urllib.request
urllib.request.urlopen("http://127.0.0.1:%s/health" % sys.argv[1], timeout=1).read()
PY
    then return 0; fi
    if ! kill -0 "$MOCK_PID" 2>/dev/null; then break; fi
    sleep 0.2; i=$((i+1))
  done
  echo "${RED}목 서버가 시작되지 않았어요 (포트 $PORT 사용 중일 수 있어요).${RST}" >&2
  cat "$TMP/mock.err" >&2
  stop_mock
  return 1
}

# Cargo 패키지 이름 읽기
PKG=""
if [ -f rust/Cargo.toml ]; then
  PKG="$(sed -n 's/^name *= *"\([^"]*\)".*/\1/p' rust/Cargo.toml | head -1)"
fi

FAILED=0
STEP_FAIL=""
note_fail() { FAILED=1; STEP_FAIL="$STEP_FAIL $1"; }

show_err() { # $1 = stderr 파일
  sed 's/^/    /' "$1" | head -40
}

todo_hint() { # $1 = stderr 파일 ; todo!() 패닉이면 안내 출력 후 0
  if grep -q 'not yet implemented' "$1"; then
    local msg
    msg="$(grep -o 'not yet implemented: .*' "$1" | head -1 | sed 's/^not yet implemented: //')"
    echo "${YEL}  아직 TODO가 남아 있어요: ${msg}${RST}"
    echo "  rust/src/main.rs 의 TODO 를 순서대로 채워 보세요."
    return 0
  fi
  return 1
}

echo "${BLD}== $CHNAME ($([ "$SOL" = 1 ] && echo 정답 || echo 실습)) ==${RST}"

# --- Python ---
if [ -f python/main.py ]; then
  start_mock || note_fail python
  if python3 python/main.py >"$TMP/py.out" 2>"$TMP/py.err"; then
    echo "  Python  : 실행 완료"
  else
    echo "${RED}  Python 실행 실패${RST}"; show_err "$TMP/py.err"; note_fail python
  fi
  stop_mock
else
  echo "${RED}  python/main.py 가 없어요${RST}"; note_fail python
fi

# --- C++ ---
if ls cpp/*.cpp >/dev/null 2>&1; then
  mkdir -p "$REPO/target/cpp/$CHNAME"
  CXXBIN="$REPO/target/cpp/$CHNAME/main"
  # shellcheck disable=SC2046
  if g++ -std=c++20 -O2 -Wall -Wextra cpp/*.cpp -o "$CXXBIN" $(cat cpp/BUILD_FLAGS 2>/dev/null) 2>"$TMP/cpp.build.err"; then
    start_mock || note_fail cpp
    if "$CXXBIN" >"$TMP/cpp.out" 2>"$TMP/cpp.err"; then
      echo "  C++     : 실행 완료"
    else
      echo "${RED}  C++ 실행 실패${RST}"; show_err "$TMP/cpp.err"; note_fail cpp
    fi
    stop_mock
  else
    echo "${RED}  C++ 컴파일 실패${RST}"; show_err "$TMP/cpp.build.err"; note_fail cpp
  fi
else
  echo "${RED}  cpp/*.cpp 가 없어요${RST}"; note_fail cpp
fi

# --- Rust ---
RUST_OK=0
if [ -z "$PKG" ]; then
  echo "${RED}  rust/Cargo.toml 이 없어요${RST}"; note_fail rust
else
  RARGS=""
  if [ "$SOL" = 1 ]; then
    if [ -f rust/examples/solution.rs ]; then
      RARGS="-p $PKG --example solution"
    elif [ -f rust_solution/Cargo.toml ]; then
      SPKG="$(sed -n 's/^name *= *"\([^"]*\)".*/\1/p' rust_solution/Cargo.toml | head -1)"
      RARGS="-p $SPKG"
    else
      echo "${RED}  정답(examples/solution.rs 또는 rust_solution/)이 없어요${RST}"; note_fail rust
    fi
  else
    RARGS="-p $PKG"
  fi
  if [ -n "$RARGS" ]; then
    start_mock || note_fail rust
    # shellcheck disable=SC2086
    if cargo run -q $RARGS >"$TMP/rs.out" 2>"$TMP/rs.err"; then
      echo "  Rust    : 실행 완료"; RUST_OK=1
    else
      if todo_hint "$TMP/rs.err"; then :; else
        echo "${RED}  Rust 실행 실패${RST}"; show_err "$TMP/rs.err"
      fi
      note_fail rust
    fi
    stop_mock
  fi
fi

# --- 비교 ---
cmp_pair() { # $1 이름, $2 파일A, $3 파일B
  if [ -f "$2" ] && [ -f "$3" ]; then
    if diff -u "$2" "$3" >"$TMP/diff.txt" 2>&1; then
      echo "${GRN}  ✅ $1 : 출력이 똑같아요${RST}"
    else
      echo "${RED}  ❌ $1 : 출력이 달라요${RST}"
      sed 's/^/    /' "$TMP/diff.txt" | head -60
      note_fail "diff($1)"
    fi
  fi
}
cmp_pair "python vs cpp " "$TMP/py.out" "$TMP/cpp.out"
if [ "$RUST_OK" = 1 ]; then
  cmp_pair "python vs rust" "$TMP/py.out" "$TMP/rs.out"
else
  echo "${YEL}  (Rust 결과가 없어 python vs rust 비교는 건너뛰었어요)${RST}"
fi

if [ "$FAILED" = 0 ]; then
  echo "${GRN}${BLD}통과: $CHNAME${RST}"
  exit 0
fi
echo "${RED}${BLD}실패: $CHNAME  (단계:${STEP_FAIL})${RST}"
exit 1
