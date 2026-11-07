# Token audit: process map

Every step an agent takes to port one mod (`/conductor` or `make agy`), with the files and commands that put tokens into its context. Red = predicted heavy, amber = medium, green = light. Predicted ranges are in [`predictions.md`](predictions.md); measured ones in `performance.md` once sampled.

```mermaid
flowchart TD
  classDef heavy fill:#f8d7da,stroke:#b02a37,color:#000
  classDef mid fill:#fff3cd,stroke:#b58105,color:#000
  classDef light fill:#d1e7dd,stroke:#146c43,color:#000

  S0["S0 Session start<br/>system prompt + tools + AGENTS.md + hooks<br/>bd prime, start.md, make doctor"]:::mid
  C1["C1 Conductor boot<br/>reads conductor/agent.md 14KB<br/>make fleet-start / fleet-resume"]:::mid

  subgraph STEP1["Step 1: Spec and tickets"]
    D1["S1a make decompile + fleet-parts<br/>(shell, output only)"]:::light
    W1["S1b spec-writer: outline per part<br/>mod-spec skill 12KB + java-outline/show<br/>writes .md + features.json"]:::heavy
    W2["S1c spec-writer: tickets per part<br/>reads spec + Java again, writes tickets JSON"]:::heavy
    W3["S1d make fleet-tickets<br/>(validate + file into Beads)"]:::light
  end

  S2["S2 Scaffold<br/>scaffolder copies example_mod"]:::light

  subgraph STEP3["Step 3: Code, one subagent per ticket"]
    L0["S3a fleet-task + role file + skills<br/>(fixed overhead per ticket)"]:::mid
    L1["S3b contract file per block<br/>from the Java, a method at a time"]:::heavy
    L2["S3c content-author<br/>block/item/recipe/lang JSON"]:::heavy
    L3["S3d domain-builder / -pro<br/>Rust crate + cargo build/test output"]:::heavy
    L4["S3e hook-engineer<br/>mixin bundle + horizon-hooks skill"]:::heavy
    L5["S3f test-smith<br/>tests + e2e_actions.json"]:::mid
    L6["S3g report write<br/>target/fleet/inbox/&lt;bead&gt;.json"]:::light
  end

  CL["C2 Conductor loop<br/>fleet-ready, spawn, wait, fleet-inbox, board<br/>context grows with every report"]:::heavy

  subgraph STEP4["Steps 4 to 7: Check, assets, recheck"]
    K1["S4 agent-gate<br/>rustfmt, clippy, tests output"]:::mid
    K2["S4b parity-critic<br/>spec + diff + Java"]:::heavy
    K3["S5 asset-smith<br/>extract + map sprites"]:::mid
    K4["S6 new assets<br/>fleet-gaps, make art"]:::light
    K5["S7 recheck<br/>second parity-critic pass"]:::heavy
  end

  H8["S8 live harness<br/>e2e-mod, e2e-all-mods logs"]:::heavy
  P9["S9 package<br/>release-clerk, scribe, pack-mods"]:::light
  X["X Cross-cutting<br/>stop-gate retries, hook output,<br/>context-limit wrap-ups (200k), resume"]:::heavy

  S0 --> C1 --> D1 --> W1 --> W2 --> W3 --> S2 --> CL
  CL --> L0 --> L1 --> L2 --> L6
  L1 --> L3 --> L6
  L1 --> L4 --> L6
  L1 --> L5 --> L6
  L6 --> CL
  CL --> K1 --> K2 --> K3 --> K4 --> K5 --> H8 --> P9
  X -.adds to every step.-> CL
```

## Where context is paid

Every turn re-sends the whole context, so a step costs roughly the sum of the context size over its turns, not its final context size.

```mermaid
flowchart LR
  A["fixed overhead per agent<br/>system prompt + tools + AGENTS.md<br/>+ role file + skills<br/>(about 12k to 25k, paid every turn)"] --> T["cost = sum over turns of context size"]
  B["growth per turn<br/>file reads, command output,<br/>cargo/clippy/e2e logs, own writes"] --> T
  C["fan-out<br/>agents in flight x turns each"] --> T
```
