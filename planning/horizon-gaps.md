# Horizon engine gaps

Moved to the issue tracker: every engine gap is a bug labelled `horizon` (`issuectl ls "label:horizon" --all`).

    bd list --label horizon        # all of them
    bd show <id>                   # one, with its notes

Agents file them through their reports (`.agents/skills/horizon-hooks/SKILL.md`); a person triages each one
(upstream a Horizon PR, work around it, or defer). The entry that was here is `rk-yls` (vanilla loot-pool
`conditions` drop nothing).
