# Changelog

All notable changes to the protocol. Versions follow semver, and two versions are compatible when MAJOR.MINOR match. Before 1.0.0, any minor bump could break compatibility.

## [1.0.0] - 2026-11-04

The first stable release: 0.2.0's messages, unchanged, with the semantics brought in line with what ArcadiaBench 1.0 implements. From 1.0.0 the protocol follows semver, and compatibility stays MAJOR.MINOR, because the strict schemas make a field added in a minor release unreadable to the previous minor (see the README).

No schema shape changed. Every message that validated under 0.2.0 still validates, and every one that failed still fails; only the `$id`s and some descriptions differ. 0.2 and 1.0 are still incompatible under the MAJOR.MINOR rule, so a coordinator must announce `1.0.0`.

### Changed

- `PROTOCOL_VERSION` and the crate version are `1.0.0`, and every schema `$id` embeds `v1.0.0`.
- Schema descriptions now match the semantics: `approval_id` is the `message_id` of the cited `approval_request`; a level-3 request's deadline is the earlier of `decide_by_t` and the send time plus `level3.max_wait_s`; the `out_of_coverage` fail reason is reserved and never produced; grading moves no material; `observed` and `truth` log records include the run's final instant; the compatibility note in `messages.schema.json` gives the 1.0 rule.
- The fixtures carry version 1.0.0, for the protocol and for the example simulator and coordinator.

### Fixed

`docs/semantics.md` now matches ArcadiaBench 1.0 wherever it stated a behaviour differently or not at all:

- Event log (§8.6, §10): every `observed` record is logged at the time of the observation that delivered its event, while the event keeps its own `t`. When the run ends, the log carries the events never delivered, then a final `truth` record, then `run_end`, all at the end time. A truth record's `storage_wh` is a preview: the grid settled up to the record's `t`.
- Coverage (§1.3, §6.4, §8.4, §8.5): a robot out of coverage creates no decision points, only its return does; lost robots follow coverage too; each command of a decision sees the coverage left by the ones before it. A robot engaged by another never starts a lost-link return, and the return stops at the first covered cell on the way to the base station.
- The Earth link (§8.1): the conjunction blackout starts at a seeded time inside the run, not on a seeded sol. Unplanned outages never start inside it; one that runs into it gives way, and the change of cause while the link stays down is a `comms` event.
- Mission control (§8.3): it judges `commit_energy_reserve` on site storage when the request was sent, and hears of incidents through telemetry even from robots out of coverage. The trust script's two rules are spelled out.
- Authority (§9.2): a level-3 deadline has passed once `t ≥ deadline`. An approval request that repeats an earlier request's `message_id` is ignored: Earth never answers it, and no command can cite it.
- Repair options (§4.1, §4.3): `tow` and `cannibalize` targets must have no running command, else `invalid_target`. Stripping and installing are work steps of a fixed work content at the executor's repair rate. A repair's work counts a stuck robot, and its estimate ignores hidden wear.
- Failures (§6.4): hazards are evaluated when each exposed step starts; compacted and stabilized ground is firm.
- Dust storms (§2): a storm's build-up is no longer described as its first fifth, which the `dust_storm` variant's storm front does not follow.
- §8.5 states a known limitation: acceptance checks on another robot read its true state, even out of coverage.
- Simulator tunables are described rather than quoted, since ArcadiaBench's `docs/simulator.md` owns the numbers, and the notes on development history are gone: ArcadiaBench 1.0 implements every section.

### Added

- `docs/semantics.md`, appendix: the reference constants a coordinator has to assume because no message carries them (the grade window, the lost-link timeout, the relay radio's power and more), with ArcadiaBench 1.0's Standard track values.
- README: the stability statement, and the candidates for 1.1.

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
