#!/usr/bin/env python3
"""Which model a headless fleet run uses, and which models are out of quota.

The chain is .agents/models.json: tools in order, each with its models best first. A candidate is
usable when its CLI is installed, the CLI offers the model (agy: `agy models`) and it is not resting.
A run that ends on a quota, usage or rate limit puts its model to rest until the reset time the
error names (else 30 minutes, doubling on each hit in a row, at most 6 hours); a model the CLI
rejects rests a day. The rest times live in target/fleet/models.json, shared by every run.

Usage:
  python3 scripts/fleet/models.py            # the chain, and what is resting (make fleet-models)
  python3 scripts/fleet/models.py --clear    # forget every rest (after buying more quota, say)
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import socket
import subprocess
import time
from dataclasses import dataclass
from datetime import datetime, timedelta
from pathlib import Path

import tracker

ROOT = tracker.main_checkout()
CONFIG = ROOT / ".agents" / "models.json"
STATE = ROOT / "target" / "fleet" / "models.json"
DEFAULT_REST = 30 * 60
OVERLOADED_REST = 5 * 60  # the provider is busy, not our quota
MAX_REST = 6 * 3600
UNAVAILABLE_REST = 24 * 3600
TAIL = 40  # only the end of a run's output is searched: the agent's own text may mention limits

LIMIT = re.compile(
    r"RESOURCE_EXHAUSTED|quota (?:reached|exceeded|exhausted)|insufficient[_ ]quota|usage limit|"
    r"rate[ _-]?limit(?:ed| reached| exceeded|_error)|hit your (?:usage )?limit|limit reached|"
    r"too many requests|(?:code|status|http|error)\W{0,3}429\b|overloaded|credit balance is too low", re.I)
UNAVAILABLE = re.compile(
    r"\bmodel\b[^\n]{0,60}(?:not found|not available|unavailable|does not exist|not supported|invalid)|"
    r"invalid model|unknown model|not_found_error", re.I)
DURATION = re.compile(r"(?:resets?|try again|retry)(?: after| in)?\s+((?:\d+\s*(?:d|h|m|s)\w*[\s,]*)+)", re.I)
UNIT = re.compile(r"(\d+)\s*([dhms])", re.I)
EPOCH = re.compile(r"limit reached\|(\d{10})")
CLOCK = re.compile(r"resets?(?: at)?\s+(\d{1,2})(?::(\d\d))?\s*(am|pm)\b", re.I)


@dataclass(frozen=True)
class Candidate:
    tool: str
    model: str | None

    @property
    def key(self) -> str:
        return f"{self.tool}:{self.model or 'default'}"


def online() -> bool:
    """Whether the internet is reachable (DNS and a TCP connection), which every agent CLI needs."""
    for host in ("api.anthropic.com", "generativelanguage.googleapis.com", "1.1.1.1"):
        try:
            with socket.create_connection((host, 443), timeout=5):
                return True
        except OSError:
            continue
    return False


def config() -> dict:
    return json.loads(CONFIG.read_text(encoding="utf-8"))


def offered(tool: str) -> list[str] | None:
    """The models the CLI says it can use, or None when it cannot say."""
    if tool != "agy":
        return None
    try:
        out = subprocess.run(["agy", "models"], capture_output=True, text=True, timeout=60).stdout
    except (OSError, subprocess.TimeoutExpired):
        return None
    names = [line.split("\t", 1)[0].strip() for line in out.splitlines() if "\t" in line]
    return names or None


def candidates(prefer: str | None = None) -> list[Candidate]:
    cfg = config()
    tools = list(dict.fromkeys([*([prefer] if prefer else []), *cfg["tools"]]))
    out: list[Candidate] = []
    for tool in tools:
        if not shutil.which(tool):
            continue
        chain = cfg.get(tool, {}).get("chain") or [None]
        listed = offered(tool)
        if listed is not None:
            chain = [m for m in chain if m in listed] + [m for m in listed if m not in chain]
        out += [Candidate(tool, m) for m in chain]
    return out


def load() -> dict:
    try:
        return json.loads(STATE.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return {}


def save(state: dict) -> None:
    STATE.parent.mkdir(parents=True, exist_ok=True)
    STATE.write_text(json.dumps(state, indent=2) + "\n", encoding="utf-8")


def resting_until(c: Candidate, now: float | None = None) -> float | None:
    until = load().get(c.key, {}).get("until", 0)
    return until if until > (now or time.time()) else None


def first_ready(cands: list[Candidate]) -> Candidate | None:
    now = time.time()
    return next((c for c in cands if not resting_until(c, now)), None)


def earliest_reset(cands: list[Candidate]) -> float | None:
    times = [t for c in cands if (t := resting_until(c))]
    return min(times) if times else None


def reset_seconds(text: str) -> float | None:
    """Seconds until the limit lifts, from what the error says."""
    if m := EPOCH.search(text):
        return max(int(m.group(1)) - time.time(), 60)
    if m := DURATION.search(text):
        scale = {"d": 86400, "h": 3600, "m": 60, "s": 1}
        secs = sum(int(n) * scale[u.lower()] for n, u in UNIT.findall(m.group(1)))
        if secs:
            return secs
    if m := CLOCK.search(text):
        hour = int(m.group(1)) % 12 + (12 if m.group(3).lower() == "pm" else 0)
        now = datetime.now()
        at = now.replace(hour=hour, minute=int(m.group(2) or 0), second=0, microsecond=0)
        if at <= now:
            at += timedelta(days=1)
        return (at - now).total_seconds()
    return None


def limit_in(output: str, tool_log: str = "") -> tuple[str, float | None] | None:
    """(reason, rest seconds) when a run ended on a limit or an unusable model, else None."""
    lines = output.splitlines()[-TAIL:] + [ln for ln in tool_log.splitlines() if ln.startswith("E")]
    for pattern, kind in ((LIMIT, "limit"), (UNAVAILABLE, "unavailable")):
        hits = [ln.strip() for ln in lines if pattern.search(ln)]
        if hits:
            if kind == "unavailable":
                return hits[-1][:300], UNAVAILABLE_REST
            text = "\n".join(hits)
            seconds = reset_seconds(text)
            if seconds is None and re.search(r"overloaded", text, re.I) and not re.search(r"quota|usage", text, re.I):
                seconds = OVERLOADED_REST
            return hits[-1][:300], seconds
    return None


def rest(c: Candidate, reason: str, seconds: float | None) -> float:
    """Put a model to rest; returns when it may run again."""
    state = load()
    entry = state.get(c.key, {})
    hits = entry.get("hits", 0) + 1
    if seconds is None:
        seconds = min(DEFAULT_REST * 2 ** (hits - 1), MAX_REST)
    until = time.time() + seconds
    state[c.key] = {"until": until, "hits": hits, "reason": reason,
                    "since": datetime.now().isoformat(timespec="seconds")}
    save(state)
    return until


def ran_ok(c: Candidate) -> None:
    state = load()
    if state.pop(c.key, None) is not None:
        save(state)


def when(ts: float) -> str:
    return datetime.fromtimestamp(ts).strftime("%a %H:%M")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--clear", action="store_true", help="Forget every rest")
    parser.add_argument("--prefer", help="Tool to try first (as AGENT=<tool> does)")
    args = parser.parse_args()
    if args.clear:
        STATE.unlink(missing_ok=True)
        print("Every model is available again.")
        return 0
    cands = candidates(args.prefer)
    if not cands:
        print("No agent CLI from .agents/models.json is installed (agy, claude, codex).")
        return 1
    state, ready = load(), first_ready(cands)
    for c in cands:
        until = resting_until(c)
        mark = "resting until " + when(until) if until else "next" if c == ready else "ready"
        print(f"  {c.key:40} {mark}")
        if until:
            print(f"  {'':40}   {state[c.key]['reason'][:110]}")
    if not ready:
        print(f"Every model is resting; the first is back {when(earliest_reset(cands))}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
