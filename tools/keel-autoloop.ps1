# KEEL — autonomous build supervisor (the external Driver: it does the handoffs so the operator never
# has to). Respawns fresh headless Claude Code sessions; each self-bootstraps from AUTOSTART.md,
# executes ROADMAP slices to ~90% context, banks + pushes, and exits; this loop respawns the next —
# until DONE/STALLED, a no-progress stall, or the safety cap. KEEL is "the loop"; this is the temporary
# external loop that builds it, until KEEL can host its own (the daemon, ROADMAP A2).
#
# ONE-TIME operator setup (see _run_state/AUTOSTART_SETUP.md): set the unattended permission policy,
# (optionally) wire the SessionStart/PreCompact hooks, then run this on the trusted box.
#
#   pwsh -File tools\keel-autoloop.ps1 [-MaxSessions 50] [-StallLimit 2] [-PermissionMode bypassPermissions]
#
# Stops when: .keelstate\DONE or .keelstate\STALLED appears; HEAD unchanged for -StallLimit sessions
# in a row (no progress -> operator input likely needed); or -MaxSessions reached (safety cap).
# Everything is logged to .keelstate\autoloop.log and every slice is committed + pushed (fully auditable).

param(
    [int]$MaxSessions = 50,
    [int]$StallLimit = 2,
    # The unattended permission flag passed to `claude -p`. Full autonomy (Bash/cargo/git/commit/push
    # without prompts) on a TRUSTED box = `--dangerously-skip-permissions` (verified June 2026; the old
    # `--permission-mode bypassPermissions` is NOT a valid value). For a tighter policy, swap to an
    # `--allowedTools "..."` allowlist or a settings.json `permissions.allow` set (see AUTOSTART_SETUP.md).
    [string]$ClaudeFlag = "--dangerously-skip-permissions"
)

$ErrorActionPreference = "Stop"
$repo = "C:\KEEL"
$state = Join-Path $repo ".keelstate"
New-Item -ItemType Directory -Force -Path $state | Out-Null
$log = Join-Path $state "autoloop.log"
$autostartPath = Join-Path $repo "_run_state\AUTOSTART.md"

function Log($m) {
    $line = "[{0}] {1}" -f (Get-Date -Format o), $m
    Add-Content -Path $log -Value $line
    Write-Host $line
}

if (-not (Test-Path $autostartPath)) { Log "FATAL: AUTOSTART.md not found at $autostartPath"; exit 1 }
$autostart = Get-Content $autostartPath -Raw

Log "=== KEEL autoloop START (MaxSessions=$MaxSessions, StallLimit=$StallLimit, ClaudeFlag=$ClaudeFlag) ==="
$stall = 0
$i = 0
while ($i -lt $MaxSessions) {
    if (Test-Path (Join-Path $state "DONE"))    { Log "DONE sentinel present -> KEEL complete; winding down (stop)."; break }
    if (Test-Path (Join-Path $state "STALLED")) { Log "STALLED sentinel present -> operator input needed (stop)."; break }
    $i++
    $before = (& git -C $repo rev-parse HEAD).Trim()
    Log "--- session $i/$MaxSessions starting (HEAD $before) ---"
    try {
        # Headless, unattended: one `claude -p` runs a full multi-step agentic session (the model works
        # through ROADMAP slices until ~90% context, then exits per AUTOSTART). Then we respawn a fresh one.
        & claude -p $autostart $ClaudeFlag 2>&1 | Tee-Object -FilePath $log -Append
    } catch {
        Log "session $i errored: $($_.Exception.Message)"
    }
    $after = (& git -C $repo rev-parse HEAD).Trim()
    if ($after -eq $before) {
        $stall++
        Log "session $i made NO commit (HEAD unchanged): stall $stall/$StallLimit."
        if ($stall -ge $StallLimit) { Log "stall limit reached -> stop (no progress; check ROADMAP ISSUES / STALLED)."; break }
    } else {
        $stall = 0
        Log "session $i advanced HEAD: $before -> $after."
    }
}
Log "=== KEEL autoloop EXIT after $i session(s) ==="
