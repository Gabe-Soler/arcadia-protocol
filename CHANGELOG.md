# Changelog

All notable changes to the protocol. Versions follow semver; during 0.x any minor bump may break compatibility.

## [0.1.0] - 2026-11-08

First release, scoped to the v1 landing-pad mission.

- Loop envelope: `hello` / `welcome` / `observation` / `decision` / `end` / `error`, over newline-delimited JSON on stdio.
- Site map with partial-knowledge cells (elevation, regolith, boulders, surface state, ice).
- Capability-based robot specs; charge stations and depots.
- Robot state with noisy health telemetry.
- Commands: move_to, scan, dig, grade, haul, compact, sinter, repair, tow, cannibalize, charge, relay, wait, cancel.
- Events: command lifecycle, scan results, faults (wear, random, environmental), weather, power, comms, Earth messages.
- Earth link: goal, approve, veto, reprioritize, policy_update; notify, approval_request, status_report.
- Authority policy with four levels and level-3 fallback parameters.
- JSONL event log with ground truth for scoring and replay.
