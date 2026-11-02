#!/usr/bin/env python3
"""Look at the port's progress in Beads: in the terminal, or as a page in the browser.

  tasks [<mod>]   the mod in progress (or <mod>) on one screen: each step's progress, what is being
                  worked, what is open in the current step, open bugs, subagents running or
                  interrupted, and the whole board in one line
  watch [<mod>]   tasks, redrawn every 30 seconds (Ctrl+C stops)
  html            target/fleet/board.html: every mod, and the mod in progress as one column per step
  serve           that page live at http://127.0.0.1:8765, rebuilt from Beads on every refresh (the page
                  reloads itself every 30 seconds); opens your browser

Reads only: safe while agents work.

Usage:
  make tasks [MOD=<id>]   make watch [MOD=<id>]   make board-html   make board-live
"""

from __future__ import annotations

import html
import http.server
import os
import sys
import threading
import time
import webbrowser

import beads
import resume

OUT = beads.STATE_DIR / "board.html"
PORT = int(os.environ.get("BOARD_PORT", "8765"))  # BOARD_PORT=<n> make board-live for another port
DONE = ("closed",)
ORDER = {"in_progress": 0, "blocked": 1, "open": 2, "deferred": 3, "closed": 4}


class Snapshot:
    """Every issue once, grouped the way the views need them."""

    def __init__(self) -> None:
        self.all = beads.issues()
        self.by_id = {i["id"]: i for i in self.all}
        self.epics = {beads.label_value(i, "mod"): i for i in self.all
                      if i["issue_type"] == "epic" and beads.label_value(i, "mod")}

    def children(self, parent: str) -> list[dict]:
        # The parent field wins over the id: a bead moved to another parent keeps its old id.
        return [i for i in self.all if (i.get("parent") or i["id"].rsplit(".", 1)[0]) == parent]

    def stories(self, mod: str) -> dict[int, dict]:
        out = {}
        for s in self.children(self.epics[mod]["id"]):
            step = beads.label_value(s, "step")
            if s["issue_type"] == "story" and step and step.isdigit():
                out[int(step)] = s
        return out

    def active(self) -> str | None:
        return next((m for m, e in self.epics.items() if e["status"] == "in_progress"), None)


def bar(done: int, total: int, width: int = 12) -> str:
    if not total:
        return " " * width
    fill = round(width * done / total)
    return "█" * fill + "░" * (width - fill)


def step_line(snap: Snapshot, n: int, story: dict) -> tuple[str, int, int]:
    kids = snap.children(story["id"])
    done = sum(k["status"] in DONE for k in kids)
    return story["status"], done, len(kids)


def tasks(mod: str | None) -> str:
    snap = Snapshot()
    mod = mod or snap.active()
    counts = {s: sum(e["status"] == s for e in snap.epics.values()) for s in ("closed", "in_progress", "blocked", "open")}
    ready = beads.next_mod()
    board = (f"Board: {len(snap.epics)} mods: {counts['closed']} done, {counts['in_progress']} in progress, "
             f"{counts['blocked']} blocked, {counts['open']} queued" + (f"; next ready: {ready}" if ready else ""))
    if not mod or mod not in snap.epics:
        return board + ("" if not mod else f"\nNo epic for {mod}.")
    epic = snap.epics[mod]
    st = snap.stories(mod)
    ms = beads.mod_status(mod)  # the gate's view: a step is done only when nothing under it is open
    core = ms["core"] if mod != beads.CORE else []
    pending = [n for n in beads.AGENT_STEPS if ms["next"].startswith(f"step {n} ")]
    out = [f"{mod}  ({epic['id']}, {epic['status']})   next: {ms['next']}", ""]
    for n, story in sorted(st.items()):
        status, done, total = step_line(snap, n, story)
        if n == 3:
            total += len(core)
        open_under = ms["unsettled"].get(n, 0) + (len(core) if n == 3 else 0)
        mark = ("done" if not open_under else "open") if status == "closed" else status
        out.append(f"  {n:>2} {beads.STEPS[n]:<15} {bar(done, total) if total else bar(status == 'closed', 1)} "
                   f"{f'{done}/{total}' if total else '':>9}  {mark}")
    family = [i for i in snap.all if i["id"].startswith(epic["id"] + ".")]
    working = [i for i in family if i["status"] == "in_progress" and i["issue_type"] != "story"]
    if working:
        out += ["", "Being worked:"] + [f"  {i['id']:<14} {i['title'][:90]}" for i in working[:12]]
    if pending:
        cur = st[pending[0]]
        open_now = sorted((k for k in snap.children(cur["id"]) if k["status"] not in DONE),
                          key=lambda k: (ORDER.get(k["status"], 9), id_key(k["id"])))
        if open_now:
            out += ["", f"Open in step {pending[0]} {beads.STEPS[pending[0]]} ({len(open_now)}):"]
            out += [f"  {k['id']:<14} {k['status']:<11} {k['title'][:80]}" for k in open_now[:15]]
            if len(open_now) > 15:
                out.append(f"  … {len(open_now) - 15} more (bd children {cur['id']})")
    if core:
        out += ["", f"Core work for {mod} (under rekkit_core; part of step 3):"]
        out += [f"  {i['id']:<14} {i['title'][:90]}" for i in core]
    bugs = [i for i in family if i["issue_type"] == "bug" and i["status"] not in DONE
            and i.get("parent") not in {e["id"] for e in snap.epics.values()}]
    if bugs:
        out += ["", f"Open bugs ({len(bugs)}):"] + [f"  {b['id']:<14} {b['title'][:90]}" for b in bugs[:10]]
    closed_epics = {e["id"] for e in snap.epics.values() if e["status"] == "closed"}
    orphans = [i for i in snap.all if i["status"] not in DONE and i.get("parent") in closed_epics]
    if orphans:
        out += ["", "Open under a closed epic (no run will pick these up; reopen the epic or move them):"]
        out += [f"  {i['id']:<14} {i['title'][:90]}" for i in orphans[:10]]
    flight = [line for line in resume.digest(mod).splitlines()
              if line.startswith(("Running now", "Interrupted", "  rk-", "Unfiled"))]
    if flight:
        out += [""] + flight
    out += ["", board]
    return "\n".join(out)


# ---------------------------------------------------------------- HTML

CSS = """
:root{--bg:#f6f7f9;--card:#fff;--ink:#1d2330;--muted:#667085;--line:#e3e6ec;--done:#2e9e6b;--work:#d98c1a;
--block:#d0453b;--open:#8a94a6;--accent:#3c64d8}
@media (prefers-color-scheme:dark){:root{--bg:#12151b;--card:#1b2029;--ink:#e6e9ef;--muted:#98a2b3;--line:#2a3140;
--done:#4cc38a;--work:#f0a83a;--block:#ef6a60;--open:#6b7588;--accent:#7b9cff}}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--ink);font:14px/1.45 system-ui,-apple-system,
"Segoe UI",sans-serif;padding:24px 16px}main{max-width:1400px;margin:0 auto}h1{font-size:20px;margin:0 0 4px}
h2{font-size:15px;margin:28px 0 10px}.muted{color:var(--muted)}table{border-collapse:collapse;width:100%;
background:var(--card);border:1px solid var(--line);border-radius:8px;overflow:hidden}th,td{padding:6px 10px;
border-bottom:1px solid var(--line);text-align:left;vertical-align:middle}th{font-weight:600;color:var(--muted);
font-size:12px}.pill{display:inline-block;padding:1px 8px;border-radius:99px;font-size:12px;color:#fff}
.closed{background:var(--done)}.in_progress{background:var(--work)}.blocked{background:var(--block)}
.open,.deferred{background:var(--open)}.steps{display:flex;gap:3px}.steps i{width:14px;height:10px;border-radius:2px;
background:var(--line)}.steps i.closed{background:var(--done)}.steps i.in_progress{background:var(--work)}
.steps i.blocked{background:var(--block)}.cols{display:grid;grid-template-columns:repeat(auto-fill,minmax(230px,1fr));
gap:12px}.col{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:10px}
.col h3{font-size:13px;margin:0 0 6px;display:flex;justify-content:space-between;gap:6px}.meter{height:6px;
background:var(--line);border-radius:3px;margin-bottom:8px;overflow:hidden}.meter b{display:block;height:100%;
background:var(--done)}.card{border-left:3px solid var(--open);padding:3px 6px;margin:3px 0;font-size:12px;
background:var(--bg);border-radius:3px;overflow-wrap:anywhere}.card.closed{border-color:var(--done);opacity:.6}.card.in_progress{
border-color:var(--work)}.card.blocked{border-color:var(--block)}.card code{color:var(--muted);font-size:11px}
details summary{cursor:pointer;color:var(--accent);font-size:12px;margin-top:4px}pre{background:var(--card);
border:1px solid var(--line);border-radius:8px;padding:10px;overflow:auto;font-size:12px}
@media (max-width:640px){th:nth-child(3),td:nth-child(3){display:none}}
"""


def id_key(bead: str) -> tuple:
    """rk-qst.6.2 before rk-qst.6.10."""
    return tuple(int(p) if p.isdigit() else p for p in bead.split("."))


def esc(text: object) -> str:
    return html.escape(str(text))


def card(i: dict) -> str:
    return f'<div class="card {esc(i["status"])}"><code>{esc(i["id"])}</code> {esc(i["title"])}</div>'


def page(live: bool) -> str:
    snap = Snapshot()
    mod = snap.active()
    rows = []
    for m, e in sorted(snap.epics.items(), key=lambda kv: (ORDER.get(kv[1]["status"], 9) if kv[1]["status"] != "closed"
                                                          else 5, kv[1].get("priority", 9), kv[0])):
        st = snap.stories(m)
        cells = "".join(f'<i class="{esc(st[n]["status"]) if n in st else ""}" title="{n} {esc(beads.STEPS[n])}"></i>'
                        for n in beads.AGENT_STEPS)
        rows.append(f'<tr><td><b>{esc(m)}</b></td><td><span class="pill {esc(e["status"])}">'
                    f'{esc(e["status"].replace("_", " "))}</span></td><td class="muted">'
                    f'{esc(beads.label_value(e, "wave") or "")}</td><td><div class="steps">{cells}</div></td></tr>')
    detail = ""
    if mod:
        epic = snap.epics[mod]
        cols = []
        for n, story in sorted(snap.stories(mod).items()):
            kids = sorted(snap.children(story["id"]), key=lambda k: (ORDER.get(k["status"], 9), id_key(k["id"])))
            done = sum(k["status"] in DONE for k in kids)
            pct = 100 * done // len(kids) if kids else (100 if story["status"] == "closed" else 0)
            todo, closed = [k for k in kids if k["status"] not in DONE], [k for k in kids if k["status"] in DONE]
            shown, rest = todo[:15], todo[15:]
            more = (f"<details><summary>{len(rest)} more open</summary>{''.join(card(k) for k in rest)}</details>"
                    if rest else "")
            if closed:
                more += f"<details><summary>{len(closed)} done</summary>{''.join(card(k) for k in closed)}</details>"
            cols.append(f'<div class="col"><h3><span>{n} {esc(beads.STEPS[n])}</span><span class="pill '
                        f'{esc(story["status"])}">{done}/{len(kids)}</span></h3><div class="meter"><b style="width:'
                        f'{pct}%"></b></div>{"".join(card(k) for k in shown)}{more}</div>')
        detail = (f'<h2>{esc(mod)} <span class="muted">({esc(epic["id"])})</span></h2>'
                  f'<pre>{esc(chr(10).join(l for l in resume.digest(mod).splitlines()[2:]))}</pre>'
                  f'<div class="cols">{"".join(cols)}</div>')
    horizon = [i for i in snap.all if "horizon" in i.get("labels", []) and i["status"] not in DONE]
    hz = ("<h2>Horizon bugs</h2>" + "".join(card(i) for i in horizon)) if horizon else ""
    refresh = '<meta http-equiv="refresh" content="30">' if live else ""
    return (f'<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" '
            f'content="width=device-width,initial-scale=1">{refresh}<title>Rekkit board</title><style>{CSS}</style>'
            f'</head><body><main><h1>Rekkit board</h1><div class="muted">From Beads, '
            f'{time.strftime("%a %H:%M:%S")}{" · refreshes every 30 s" if live else ""}</div>{detail}'
            f'<h2>Every mod</h2><table><tr><th>Mod</th><th>Status</th><th>Wave</th><th>Steps 1–9</th></tr>'
            f'{"".join(rows)}</table>{hz}</main></body></html>')


REBUILD = 15  # seconds between rebuilds of the live page (one takes a few seconds of bd calls)


def serve() -> None:
    """The page live: a background thread rebuilds it from Beads every REBUILD seconds and requests get
    the latest copy at once, so a slow rebuild never keeps the browser waiting (or hanging up)."""
    latest = {"body": b"", "code": 200}
    ready = threading.Event()

    def rebuild() -> None:
        while True:
            try:
                latest["body"], latest["code"] = page(live=True).encode(), 200
            except beads.BeadsError as exc:
                latest["body"], latest["code"] = f"<pre>[beads] {esc(exc)}</pre>".encode(), 500
            except Exception as exc:  # keep serving the last good page; show what broke
                latest["body"], latest["code"] = f"<pre>{esc(type(exc).__name__)}: {esc(exc)}</pre>".encode(), 500
            ready.set()
            time.sleep(REBUILD)

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 (http.server's name)
            if self.path != "/" and not self.path.startswith("/?"):
                self.send_response(204)  # favicon.ico and the like: nothing to build
                self.end_headers()
                return
            ready.wait(timeout=60)
            body = latest["body"] or b"<meta http-equiv='refresh' content='2'>Reading Beads..."
            try:
                self.send_response(latest["code"])
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.send_header("Cache-Control", "no-store")
                self.end_headers()
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass  # the browser left (reload, closed tab) before the page arrived

        def log_message(self, *args) -> None:
            pass

    try:
        server = http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler)
    except OSError:
        sys.exit(f"Port {PORT} is busy: another `make board-live` is running (open http://127.0.0.1:{PORT}), "
                 "or stop whatever uses it.")
    server.daemon_threads = True
    threading.Thread(target=rebuild, daemon=True).start()
    url = f"http://127.0.0.1:{PORT}"
    print(f"The board is live at {url}, rebuilt every {REBUILD} s (Ctrl+C stops).", flush=True)
    webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


def main() -> int:
    cmd = sys.argv[1] if len(sys.argv) > 1 else "tasks"
    mod = sys.argv[2] if len(sys.argv) > 2 and sys.argv[2] else None
    try:
        if cmd == "tasks":
            print(tasks(mod))
        elif cmd == "watch":
            while True:
                text = tasks(mod)
                print("\033[2J\033[H" if os.name != "nt" else "\n" * 3, end="")
                print(text + f"\n\n{time.strftime('%H:%M:%S')}, every 30 s; Ctrl+C stops.", flush=True)
                time.sleep(30)
        elif cmd == "html":
            OUT.parent.mkdir(parents=True, exist_ok=True)
            OUT.write_text(page(live=False), encoding="utf-8")
            print(f"Wrote {OUT.relative_to(beads.MAIN).as_posix()}")
            webbrowser.open(OUT.as_uri())
        elif cmd == "serve":
            serve()
        else:
            print(__doc__, file=sys.stderr)
            return 1
    except KeyboardInterrupt:
        return 0
    except beads.BeadsError as exc:
        print(f"[beads] {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
