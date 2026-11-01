# Beads: the Rekkit port's progress

- Role subagent: you never run `bd`. Your bead and what it needs are in your prompt; end with your report in `target/fleet/inbox/<bead>.json`.
- Conductor: you alone write to Beads, with the commands in `.agents/agents/conductor/agent.md` (*Beads*). Never run `bd ... --help`.
- Taking over a mod in progress (the last session died, ran out of credits or went offline): `make fleet-resume MOD=<id>` shows what to file and which subagents to restart.
- One page per mod (state, spec, archives, files): `make fleet-context MOD=<id>`. Every mod: `make fleet-status`.
