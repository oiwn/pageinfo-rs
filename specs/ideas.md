# Future Ideas

## `pginf install-skill` subcommand

Add a CLI subcommand that installs the LLM skill file to popular agent configs:

```
pginf install-skill --agent claude    # ~/.claude/skills/pginf/SKILL.md
pginf install-skill --agent opencode  # .opencode/skills/pginf.md
pginf install-skill --agent cursor    # .cursor/skills/pginf.md
```

Copies `skills/pginf.md` to the right location for the chosen agent.
