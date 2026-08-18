# Aurora-17 Changelog

## 0.6.2 — Mission 01 / Operational Demo

- Centralized incident and repair gameplay loop.
- Transit incidents begin only after `DEPART`.
- Seven seed-driven incident families with explicit physical failure modes.
- Each incident identifies the failed component, consequence, diagnostic domain and exact repair sequence.
- Every repair action requires a rotating security authorization; tokens rotate every 60 seconds of game time.
- Navigation now supports targeted `scan nav` and `diagnose navigation` output.
- `diagnose` explains what the current repair step does and what to request next.
- Reactor cooling failure can escalate into thermal runaway and vessel loss if ignored.
- Life-support, power, fuel, navigation and hull failures now produce system-specific consequences.
- Critical vessel condition triggers a restrained low-frequency emergency alarm and terminal safety reboot/red state.
- Mission loss is reserved for a vessel that has actually lost safe operating capability.
- Rejected docking is recoverable: repair, then request docking clearance again.
- Preflight-only commands are blocked after departure.
- WAYPOINT-01 approach and docking clearance complete Mission 01.
- Faster terminal output and refreshed startup / relay / docking sound design.
- New staged terminal boot audio and softer emergency siren.
