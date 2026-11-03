#!/usr/bin/env python3
"""How many agents a local model can run at once on this machine: use all the memory there is.

Only for local models (opencode, or any tool, on Ollama, LM Studio or llama.cpp). One orchestrator
runs on the model opencode has selected; its subagents run on the same model, so the weights are
loaded once and every extra agent costs only its own context (KV cache). This script measures the
memory the GPU can use (NVIDIA or AMD VRAM, Apple Silicon unified memory, else system RAM for CPU
inference), the model's weights and its KV cache per token, and says how many agents fit:

  agents = (usable memory - weights - overhead) / (KV bytes per token x context per agent)

1 means no room for subagents: the orchestrator works alone and does each role itself, one at a
time. More means the orchestrator plus (agents - 1) subagents at once, and the model server must
serve that many requests in parallel (Ollama: OLLAMA_NUM_PARALLEL, with OLLAMA_CONTEXT_LENGTH for the
context). The answer is saved in target/fleet/local.json for the conductor. A cloud model is not
limited by this machine: the normal fleet rules apply.

Usage:
  make fleet-local [ARGS="--model qwen3-coder:30b --ctx 65536"]
  python3 scripts/fleet/local_capacity.py [--model NAME] [--ctx TOKENS] [--weights-gb G] [--kv-kb-per-token K]
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "target" / "fleet" / "local.json"
GB = 1024 ** 3
HEADROOM = 0.90  # of the memory the GPU can use: the OS, the tools and the game need some
OVERHEAD = 1 * GB  # the server's compute buffers
MAX_AGENTS = 10
LOCAL_PROVIDERS = {"ollama", "lmstudio", "lm-studio", "llama.cpp", "llamacpp", "local", "vllm", "localai"}
KV_BYTES = {"f16": 2.0, "q8_0": 1.0, "q4_0": 0.5}  # per KV element, by OLLAMA_KV_CACHE_TYPE


def run(*cmd: str) -> str:
    try:
        return subprocess.run(cmd, capture_output=True, text=True, timeout=20).stdout
    except (OSError, subprocess.TimeoutExpired):
        return ""


def opencode_model() -> tuple[str | None, bool]:
    """The model opencode has selected ("provider/model") and whether its provider is local."""
    paths = [ROOT / "opencode.json", ROOT / "opencode.jsonc", Path.home() / ".config" / "opencode" / "opencode.json",
             Path.home() / ".config" / "opencode" / "opencode.jsonc"]
    for path in paths:
        try:
            text = re.sub(r"^\s*//.*$", "", path.read_text(encoding="utf-8"), flags=re.M)
            cfg = json.loads(text)
        except (OSError, ValueError):
            continue
        model = cfg.get("model")
        if not model:
            continue
        provider = model.split("/", 1)[0]
        base = json.dumps(cfg.get("provider", {}).get(provider, {}))
        local = provider.lower() in LOCAL_PROVIDERS or bool(re.search(r"localhost|127\.0\.0\.1|0\.0\.0\.0", base))
        return model, local
    return None, False


def ollama(path: str, body: dict | None = None) -> dict:
    host = os.environ.get("OLLAMA_HOST", "http://127.0.0.1:11434")
    if not host.startswith("http"):
        host = "http://" + host
    try:
        req = urllib.request.Request(host.rstrip("/") + path, data=json.dumps(body).encode() if body else None,
                                     headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(req, timeout=5) as r:
            return json.loads(r.read())
    except (OSError, ValueError):
        return {}


def gpu_memory(loaded: int) -> tuple[int, int, str]:
    """Bytes the GPU (or, without one, the CPU) can give models now, counting what models already use; the
    most it could with other programs closed; and what kind of memory it is."""
    nvidia = run("nvidia-smi", "--query-gpu=memory.free", "--format=csv,noheader,nounits") if shutil.which("nvidia-smi") else ""
    if nvidia.strip():
        free = sum(int(x) for x in re.findall(r"\d+", nvidia)) * 1024 ** 2
        total = sum(int(x) for x in re.findall(r"\d+", run("nvidia-smi", "--query-gpu=memory.total",
                                                             "--format=csv,noheader,nounits"))) * 1024 ** 2
        return free + loaded, total or free + loaded, "NVIDIA VRAM"
    if shutil.which("rocm-smi"):
        try:
            data = json.loads(run("rocm-smi", "--showmeminfo", "vram", "--json") or "{}")
            total = sum(int(v.get("VRAM Total Memory (B)", 0)) for v in data.values())
            used = sum(int(v.get("VRAM Total Used Memory (B)", 0)) for v in data.values())
            if total:
                return total - used + loaded, total, "AMD VRAM"
        except ValueError:
            pass
    if sys.platform == "darwin":
        total = int(run("sysctl", "-n", "hw.memsize") or 0)
        page = int(run("sysctl", "-n", "hw.pagesize") or 16384)
        stats = {k.strip(): int(v.strip(" .")) for k, v in
                 (line.split(":", 1) for line in run("vm_stat").splitlines()[1:] if ":" in line)}
        free = sum(stats.get(f"Pages {k}", 0) for k in ("free", "inactive", "speculative", "purgeable")) * page
        # Metal lets the GPU wire about 2/3 of unified memory (3/4 from 36 GB).
        cap = int(total * (0.75 if total >= 36 * GB else 0.67))
        kind = "Apple unified memory" if platform.machine() == "arm64" else "system RAM"
        return min(cap, free + loaded), cap, kind
    if Path("/proc/meminfo").is_file():
        text = Path("/proc/meminfo").read_text()
        m, t = re.search(r"MemAvailable:\s+(\d+)", text), re.search(r"MemTotal:\s+(\d+)", text)
        avail = (int(m.group(1)) * 1024 if m else 0) + loaded
        return avail, int(t.group(1)) * 1024 * 3 // 4 if t else avail, "system RAM (CPU inference)"
    if os.name == "nt":
        out = run("powershell", "-NoProfile", "-Command", "(Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory")
        avail = int(out.strip() or 0) * 1024 + loaded
        return avail, avail, "system RAM (CPU inference)"
    return 0, 0, "unknown"


def model_facts(name: str) -> dict:
    """Weights, KV bytes per token and the model's own context length, from Ollama."""
    tags = ollama("/api/tags").get("models", [])
    weights = next((m.get("size", 0) for m in tags if m.get("name") == name or m.get("model") == name), 0)
    info = ollama("/api/show", {"model": name}).get("model_info", {})
    arch = info.get("general.architecture", "")

    def get(key: str) -> int:
        return int(info.get(f"{arch}.{key}") or 0)
    layers, kv_heads = get("block_count"), get("attention.head_count_kv") or get("attention.head_count")
    key_len = get("attention.key_length") or (get("embedding_length") // max(get("attention.head_count"), 1))
    value_len = get("attention.value_length") or key_len
    per_element = KV_BYTES.get(os.environ.get("OLLAMA_KV_CACHE_TYPE", "f16"), 2.0)
    return {"weights": weights, "kv_per_token": layers * kv_heads * (key_len + value_len) * per_element,
            "context": get("context_length")}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--model", help="The local model (default: opencode's selected one, else the one Ollama has loaded)")
    parser.add_argument("--ctx", type=int, help="Context per agent in tokens (default: OLLAMA_CONTEXT_LENGTH, "
                                                "else the model's, at most 131072)")
    parser.add_argument("--weights-gb", type=float, help="The model's weights, when the server cannot say")
    parser.add_argument("--kv-kb-per-token", type=float, help="Its KV cache per token in KiB, when the server cannot say")
    args = parser.parse_args()

    selected, local = opencode_model()
    if selected and not local and not args.model:
        print(f"opencode's model {selected} is not local: this machine's memory does not limit it; the normal "
              "fleet rules apply (small tasks, many subagents).")
        OUT.parent.mkdir(parents=True, exist_ok=True)
        OUT.write_text(json.dumps({"model": selected, "local": False}, indent=2) + "\n", encoding="utf-8")
        return 0
    running = ollama("/api/ps").get("models", [])
    name = args.model or (selected.split("/", 1)[1] if selected else None) or (running[0]["name"] if running else None)
    if not name:
        print("No local model: select one in opencode (\"model\": \"ollama/<name>\" in opencode.json), load one "
              "in Ollama, or pass --model.", file=sys.stderr)
        return 1
    facts = model_facts(name)
    if args.weights_gb:
        facts["weights"] = int(args.weights_gb * GB)
    if args.kv_kb_per_token:
        facts["kv_per_token"] = args.kv_kb_per_token * 1024
    if not facts["weights"] or not facts["kv_per_token"]:
        print(f"Cannot size {name}: start Ollama with it pulled, or pass --weights-gb and --kv-kb-per-token.",
              file=sys.stderr)
        return 1
    loaded = sum(int(m.get("size_vram") or m.get("size") or 0) for m in running)
    memory, most, kind = gpu_memory(loaded)
    ctx = args.ctx or int(os.environ.get("OLLAMA_CONTEXT_LENGTH") or 0) or min(facts["context"] or 32768, 131072)
    per_agent = facts["kv_per_token"] * ctx
    room = memory * HEADROOM - facts["weights"] - OVERHEAD
    agents = max(0, min(MAX_AGENTS, int(room // per_agent))) if per_agent else 0
    best = max(0, min(MAX_AGENTS, int((most * HEADROOM - facts["weights"] - OVERHEAD) // per_agent))) if per_agent else 0
    print(f"{kind}: {memory / GB:.1f} GB the GPU can use now ({HEADROOM:.0%} of it planned); {name}: "
          f"{facts['weights'] / GB:.1f} GB of weights, {per_agent / GB:.2f} GB of KV cache per agent at {ctx:,} tokens.")
    if agents < 1:
        fit = int(max(room, 0) // facts["kv_per_token"]) if facts["kv_per_token"] else 0
        print(f"It does not fit with a {ctx:,}-token context. At most {fit:,} tokens fit for one agent: use a "
              "smaller context (--ctx) or a smaller model.")
        agents = 0
    elif agents == 1:
        print("Room for one agent: the orchestrator works alone, doing each role itself one at a time (no subagents).")
    else:
        print(f"Room for {agents} agents, but local models never run subagents: the orchestrator works alone.")
    agents = min(agents, 1)  # policy: local models never start subagents
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"model": selected or name, "local": True, "agents": agents, "subagents": max(agents - 1, 0),
                               "context": ctx, "memory_gb": round(memory / GB, 1), "kind": kind}, indent=2) + "\n",
                   encoding="utf-8")
    return 0 if agents else 1


if __name__ == "__main__":
    raise SystemExit(main())
