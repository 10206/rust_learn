#!/usr/bin/env bash
# 관리자용 전체 검증: 빌드, 정답 테스트, clippy, compare --solution
set -u
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO" || exit 2

RESULTS=""
ANY_FAIL=0
LOGDIR="$(mktemp -d "${TMPDIR:-/tmp}/verify.XXXXXX")"
trap 'rm -rf "$LOGDIR"' EXIT

run_step() { # $1 라벨, 나머지 명령
  local label="$1"; shift
  if "$@" >"$LOGDIR/step.log" 2>&1; then
    echo "PASS"
  else
    echo "FAIL"
    echo "---- 실패: $label ----" >&2
    tail -30 "$LOGDIR/step.log" >&2
  fi
}

echo "== cargo build --workspace =="
if cargo build --workspace; then B="PASS"; else B="FAIL"; ANY_FAIL=1; fi

for d in chapters/ch[0-9][0-9]_*/; do
  [ -d "$d" ] || continue
  name="$(basename "$d")"
  n="$(echo "$name" | cut -c1-4)"
  pkg=""; spkg=""; example=0
  [ -f "$d/rust/Cargo.toml" ] && pkg="$(sed -n 's/^name *= *"\([^"]*\)".*/\1/p' "$d/rust/Cargo.toml" | head -1)"
  [ -f "$d/rust_solution/Cargo.toml" ] && spkg="$(sed -n 's/^name *= *"\([^"]*\)".*/\1/p' "$d/rust_solution/Cargo.toml" | head -1)"
  [ -f "$d/rust/examples/solution.rs" ] && example=1

  echo "== $name =="
  if [ "$example" = 1 ]; then
    T="$(run_step "$name test" cargo test -q -p "$pkg" --example solution)"
    C="$(run_step "$name clippy" cargo clippy -q -p "$pkg" --examples -- -D warnings)"
  elif [ -n "$spkg" ]; then
    T="$(run_step "$name test" cargo test -q -p "$spkg")"
    C="$(run_step "$name clippy" cargo clippy -q -p "$spkg" --all-targets -- -D warnings)"
  else
    T="FAIL"; C="FAIL"; echo "정답을 찾을 수 없어요: $name" >&2
  fi
  if "$SCRIPT_DIR/compare.sh" "$n" --solution >"$LOGDIR/cmp.log" 2>&1; then
    P="PASS"
  else
    P="FAIL"; cat "$LOGDIR/cmp.log" >&2
  fi
  [ "$T" = PASS ] && [ "$C" = PASS ] && [ "$P" = PASS ] || ANY_FAIL=1
  RESULTS="$RESULTS$(printf '%-34s %-6s %-6s %-6s' "$name" "$T" "$C" "$P")
"
done

echo
echo "workspace build: $B"
printf '%-34s %-6s %-6s %-6s\n' CHAPTER TEST CLIPPY COMPARE
printf '%s' "$RESULTS"
if [ "$ANY_FAIL" = 0 ]; then echo "모두 통과했어요."; exit 0; fi
echo "실패한 항목이 있어요."; exit 1
