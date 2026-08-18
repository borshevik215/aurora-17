# AURORA-17 // v0.6.2

A terminal-driven sci-fi operations demo about keeping a damaged autonomous vessel alive long enough to reach WAYPOINT-01.

## Core loop

1. Boot the terminal.
2. Calibrate the Star Tracker and lock the WAYPOINT-01 course.
3. `depart` starts the five-minute transit.
4. Only after departure can autonomous incidents occur.
5. Read the concrete failure report.
6. Use `diagnose <system>` and `scan <system>` to identify the failed component and understand the repair.
7. `request <system>` obtains the current rotating access token.
8. `auth <system> <code>` authorizes one repair action.
9. Execute the exact action shown by diagnostics. The token does not need to remain valid after the action has started.
10. Reach WAYPOINT-01 and issue `dock request`.
11. If clearance is denied, repair the vessel and request docking again. Clearance granted completes Mission 01.

## Useful commands

- `help`
- `status`
- `diagnose`
- `diagnose navigation`
- `scan nav`
- `request navigation`
- `auth navigation <code>`
- `codes`
- `tracker`
- `calibrate <ra> <dec>`
- `lock`
- `depart`
- `nav`
- `engine`
- `comms`
- `logs`
- `inspect`
- `dock request`
- `seed`

## Reproducible runs

```bash
cargo run --release -- 123456
```

The same seed reproduces the incident RNG sequence.
