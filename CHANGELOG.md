# Changelog

All notable changes to the protocol. Versions follow semver; during 0.x any minor bump may break compatibility.

## [0.2.0] - 2026-11-03

Faults, energy and run metrics, plus a normative semantics document. Not compatible with 0.1.x: several fields are newly required.

### Added

- `docs/semantics.md`: the normative behaviour of the interface. It covers the loop and decision points, time, every command's preconditions, effects, events and failure reasons, faults and repair, energy, comms and coverage, authority enforcement, and the run metrics.
- `service` command (`depot`): drive to a depot and be serviced, clearing every fault and resetting wear.
- `PowerSystem` (`mode`, `fission_w`, `solar_peak_w`, `storage_capacity_wh`, `essential_load_w`) with `PowerMode` (`hybrid`, `solar_only`, `nuclear_heavy`), and `BaseStation` (`position`, `range_m`).
- `ActiveFault` (`component`, `severity`, `cause`, `since_t`, `field_repairable`).
- Robot status `lost`: permanently gone (cannibalized) and never commandable again.
- Reject reasons `approval_required` and `vetoed`.
- Event log: `SiteTruth`, and `RunMetrics` with its `quality`, `energy`, `fleet`, `autonomy` and `conditions` parts.
- Rust: `COMPONENTS`, the fault component names ArcadiaBench v1 uses.

### Changed (breaking)

- `Scenario` requires `power` and `base_station`.
- `RobotState` requires `faults` and `telemetry_t`.
- `robot_fault` requires `field_repairable`, and its `component` must match `^[a-z][a-z0-9_]{0,31}$`.
- The `truth` log record requires `site`, and `run_end` requires `metrics`.
- Every schema `$id` embeds `v0.2.0`.

### Changed

- The event enums (`RejectReason`, `FailReason`, `FaultCause`, `Severity`, `WeatherCondition`, `Link`, `CommsCause`) and the fault `Component` are now named `$defs` in `event.schema.json`, so other schemas can reference them. Apart from the new reject reasons, they validate exactly as before.
- Clearer descriptions for `Depot`, `Repair`, `Cannibalize`, `Charge`, `Observation` and `ActionName`, which now mentions `service`.

## [0.1.0] - 2026-11-02

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
