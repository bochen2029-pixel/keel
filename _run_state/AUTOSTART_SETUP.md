# KEEL — AUTOSTART setup (the ONE-TIME operator wiring that turns on the self-perpetuating loop)

After this one setup, KEEL builds itself toward completion across sessions with **no further handoff**.
Three parts: **(A)** the auto-bootstrap hook (kills the paste for interactive sessions + auto-rehydrates
after compaction), **(B)** the unattended permission policy, **(C)** run the supervisor. The hook JSON +
headless flags below are **verified against the Claude Code docs (June 2026)** — not guessed.

## A · Auto-bootstrap + auto-rehydrate: the SessionStart hook
*(Optional for the autonomous loop — the supervisor passes `AUTOSTART.md` to `claude -p` directly — but
recommended: it also auto-bootstraps your INTERACTIVE check-ins AND re-injects after a compaction.)*

Add to your Claude Code `settings.json` (user-level, or project `.claude/settings.json`):
```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          { "type": "command", "command": "type C:\\KEEL\\_run_state\\AUTOSTART.md", "timeout": 30 }
        ]
      }
    ],
    "PreCompact": [
      {
        "matcher": "",
        "hooks": [
          { "type": "command", "command": "powershell -NoProfile -Command \"[DateTime]::Now.ToString('o') | Add-Content C:\\KEEL\\.keelstate\\compaction.log\"", "timeout": 30 }
        ]
      }
    ]
  }
}
```
- **`matcher: ""` on SessionStart fires on every source — `startup` · `resume` · `clear` · `compact`.**
  The `compact` source is the key: it **re-injects `AUTOSTART.md` after a compaction**, so the
  post-compact session auto-rehydrates (re-reads WAKE_UP → STATE → ROADMAP, re-verifies by artifact).
  That is "the moment it realizes it compacted, rehydrate completely." The command's **stdout is
  auto-injected** as additionalContext — plain `type <file>` is enough (no JSON wrapper needed).
- **PreCompact** can only snapshot/log before compaction — it **cannot** inject context into the
  post-compact session (the SessionStart `compact` matcher above does that). The entry above just logs a
  timestamp; harmless, optional.

## B · Unattended permission policy (your TRUSTED box only)
The supervisor runs `claude -p "<AUTOSTART>" --dangerously-skip-permissions` so a headless session can
run cargo/git/commit/push without interactive prompts. This is **full autonomy** — appropriate ONLY on
your own trusted machine. It stays inside the governance because the **inviolable guards live in
`AUTOSTART.md` + the build discipline, not the permission layer**: no frozen-contract/golden/seal or
toolchain edits, secret-scan every diff, the reversibility gate, state confined to `.keelstate/`, every
slice committed + pushed (fully auditable). For a tighter policy, run the supervisor with an allowlist
instead: `pwsh -File tools\keel-autoloop.ps1 -ClaudeFlag '--allowedTools "Bash,Read,Edit,Write"'`
*(caveat: a tool that isn't allowed will HANG a headless run — full-skip avoids that on a trusted box).*

## C · Run it
```
cd C:\KEEL
pwsh -File tools\keel-autoloop.ps1                          # up to 50 sessions; stop after 2 no-progress in a row
pwsh -File tools\keel-autoloop.ps1 -MaxSessions 200 -StallLimit 3   # longer unattended run
```
**The loop:** spawn a fresh `claude -p` session → it reconstitutes from `AUTOSTART.md` → executes
`ROADMAP.md` slices to ~90% context → banks + pushes each → exits → the supervisor respawns → … until
`.keelstate\DONE` (KEEL complete → perpetual-polish), `.keelstate\STALLED` (only operator-gated items
left — see ROADMAP §5 ISSUES), a no-progress stall (HEAD unchanged `-StallLimit` times), or the
`-MaxSessions` safety cap. Full log: `.keelstate\autoloop.log`; every action is also in `git`.

## What you do after this: nothing required
Glance — whenever you feel like it — at `_run_state/WORKLOG.md` (the trail) and `_run_state/ROADMAP.md`
§5 ISSUES (the operator-only queue: pin `keel.lock` hashes, the embedder design-review, the memory
narrative register, etc.). Resolving an ISSUE unblocks its slices on the next respawn. Until then the
loop works everything else. **You never do a manual handoff again** — until quota or power runs out.
