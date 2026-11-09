# Changelog

All notable changes to the protocol. Versions follow semver. From 1.1.0, two versions are compatible when their MAJOR matches; 1.0 and the 0.x versions are compatible only with their own MAJOR.MINOR, and before 1.0.0 any minor bump could break compatibility.

## [1.1.0] - 2026-11-09

What a coordinator had to assume is now in the scenario, the Earth link can run in pass windows, and from here on a minor release no longer needs both sides to upgrade together.

Every message valid under 1.0.0 is still valid: everything added is optional, or a new enumeration value. 1.0 and 1.1 are not compatible with each other, because a 1.0 reader rejects what it does not know. That is the last such break (see Compatibility below).

### Added

- **`scenario.constants`** (`Constants`): 20 simulator constants a coordinator plans with.
  - Work: the grade window, the compaction settlement, the minimum work duration and the rock density.
  - Repair and recovery: repair work per fault, for a stuck robot and per unit of wear removed, the fraction of wear a repair removes, strip and install work, the towing speed and energy factors, and the depot's service time and capacity.
  - Drains and comms: the heater's threshold and power factor, the relay radio's power, the lost-link timeout, and mission control's mean response time.

  A simulator on 1.1 always sends it. `docs/semantics.md` names each constant where it applies, so the repair work formula and the depot's first-come-first-served queue are now normative.
- **`RobotState.tank_wh`:** the energy left in a mobile charger's tank, exact. Present only for robots with the `charge` capability.
- **Pass windows on the Earth link** (§8.1). A scenario may schedule the link in windows.
  - `CommsCause` gains `between_passes` for the gaps.
  - A `comms` event with the link up carries `window_end_t`, when the link is next scheduled to go down.
  - `expected_restore_t` is now defined as when the link is next scheduled to come up, so it is present in a gap between passes as well as in a blackout.
  - While the link is down its cause is the first that holds of `conjunction`, `outage`, `between_passes`.
- **Run metrics:** `RunMetrics.coverage` (`CoverageMetrics`), with `out_of_coverage_rejections` and `lost_link_returns`; and `ConditionMetrics.between_passes_s`. With `blackout_s` and `outage_s` it adds up to all the time the link was down.
- **`examples/later/`:** messages as a simulator on a later minor version might send them, with unknown fields, event types and enumeration values. The conformance suite checks that this version's schemas reject them and that the Rust types still read them.
- `examples/valid/SimToCoordinator/hello_1_0.json`, a 1.0 `hello`, which the suite checks is still valid.

### Changed

- **Compatibility** (§1.6, new). From 1.1, versions with the same MAJOR are compatible.
  - A coordinator ignores the fields, events and Earth messages it does not know in what the simulator sends, and takes an enumeration value it does not know as unknown.
  - A coordinator sends only what the simulator's version defines; the simulator stays strict about what it receives.
  - The schemas stay strict and belong to one version: validate a message against the schemas of the version that produced it.
  - §1.6 also states what a minor release may and may not do.
- **Rust crate.**
  - `is_compatible` implements the new rule. It also rejects versions whose MAJOR or MINOR is not a number.
  - Every enum a simulator sends has an `Unknown` variant (`#[serde(other)]`) that takes any value this version does not define: `EventKind`, `RejectReason`, `FailReason`, `FaultCause`, `Severity`, `WeatherCondition`, `Link`, `CommsCause`, `RobotStatus`, `EndReason`, `ErrorCode`, `CapabilityName`, `PowerMode`, `Surface`, `Fallback`, `Uplink`, `Objective` and `LogEntry`. A `match` on one of them needs an arm for it. `Action` and `Downlink`, which a coordinator sends, have none.
  - New fields: `Scenario.constants`, `RobotState.tank_wh`, `EventKind::Comms.window_end_t`, `RunMetrics.coverage` and `ConditionMetrics.between_passes_s`, all `Option`s. Code that builds these structs must set them.
- `PROTOCOL_VERSION` and the crate version are `1.1.0`, and every schema `$id` embeds `v1.1.0`. The fixtures carry version 1.1.0.
- A run starts at `scenario.start_t`, which need not be 0 (§2). The schemas already allowed it; the `hello` description and the README no longer say the first goal arrives at `t = 0`.
- The appendix of `docs/semantics.md` is now the list of scenario constants, with ArcadiaBench 1.0's values for reference, and a short list of the constants no message carries.
- README: the versioning rules, what 1.1 added, and candidates for 1.2 in place of the candidates for 1.1.

### Fixed

- **The coverage leak** (§4.1, §8.5; 1.0's "known limitation"). A check that looks at another robot (the target of `repair`, `tow` or `cannibalize`, the beneficiary of `cannibalize`, a mobile charger named as a `charge` source) now judges a robot that is out of coverage as it was at last contact. The command's estimate, and the cell the executor drives to, use that same state. So a rejection or an estimate no longer reveals what the coordinator has not seen. The true state counts when the executor arrives. This changes which commands a simulator accepts in that case.

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
