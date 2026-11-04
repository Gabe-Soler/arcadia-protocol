# Interface semantics

**Protocol version 0.2.0.** This document is normative.

The [JSON Schemas](../schemas/) define what every message looks like. This document defines what the messages mean: what wakes the coordinator, what each command requires and does, which events it produces and how it can fail, and how faults, energy, comms, authority and the run metrics work. ArcadiaBench implements it. A coordinator can be written from this document and the schemas alone.

The structure here is fixed by the protocol; the numbers are not. Rates, probabilities, thresholds and durations are simulator parameters. ArcadiaBench chooses them and documents their values in its `docs/simulator.md` (every value also lives in its `src/sim/params.rs`). Where a rule depends on one, this document says *a parameter*.

Free-text `detail` fields are for humans and are not part of the contract.

## Contents

1. [The loop](#1-the-loop)
2. [Time and environment](#2-time-and-environment)
3. [Robots](#3-robots)
4. [Commands](#4-commands)
5. [The landing-pad goal](#5-the-landing-pad-goal)
6. [Faults, wear and repair](#6-faults-wear-and-repair)
7. [Energy](#7-energy)
8. [Comms, coverage and Earth](#8-comms-coverage-and-earth)
9. [Authority enforcement](#9-authority-enforcement)
10. [Event log](#10-event-log)
11. [Run metrics](#11-run-metrics)

### Implementation status in ArcadiaBench

| Sections | Implemented |
|---|---|
| 1–5: the loop, time, robots, commands, the goal | Week 2. Until week 5, `repair`, `tow` and `cannibalize` pass their capability checks and are then always rejected; from week 5 they work, along with `service`. |
| 6 faults, wear and repair; 7 energy; 11 run metrics | Week 5 |
| 8 comms, coverage and Earth; 9 authority enforcement | Specified in v0.2.0; implemented in ArcadiaBench week 6. Each section says how the simulator behaves until then. |

## 1. The loop

### 1.1 Handshake

The simulator starts the coordinator as a subprocess and they exchange newline-delimited JSON on its stdin and stdout, one message per line. The coordinator's stderr is free for logs.

1. The simulator sends `hello`: protocol version, simulator name and version, run id, seed and the scenario.
2. The coordinator checks `protocol_version` before reading anything else. Two versions are compatible when MAJOR.MINOR match.
3. The coordinator replies `welcome` with its own protocol version and name.

If the versions are incompatible, either side sends `error` with code `version_mismatch` and the session stops. No observation is sent.

The scenario holds no goals. Goals arrive over the Earth link; the first arrives in the first observation as an `earth_message` event at `t = start_t` (§5).

### 1.2 Observations and decisions

After the handshake the simulator repeats:

1. It sends an `observation` with `seq`, the current time `t`, the `events` delivered since the previous observation, and a snapshot of every robot.
2. The coordinator replies with a `decision` that echoes `seq` and carries `commands`, `to_earth` messages and an optional `wake_at`.

Rules:

- `seq` starts at 0 and increases by 1 per observation. A decision with the wrong `seq` ends the run (§1.5).
- Sim time does not advance while the simulator waits for a decision, so a run reproduces exactly from its seed. There is a wall-clock limit on each reply (§1.5).
- `events` lists events in the order they were delivered. Each keeps the time `t` at which it happened, which is never later than the observation's `t`. Until week 6, delivery order is time order. From week 6, events a robot produced while out of coverage are delivered late, with their original `t` (§8.5).
- `robots` holds one snapshot per robot, in the order of `scenario.robots`, lost robots included (§3.1).

The simulator applies a decision at the observation's time `t`, in this order:

1. **Commands**, one at a time in array order (§4.1). Each produces `command_accepted` or `command_rejected` at time `t`. An accepted command can also complete or fail at once, at the same `t`.
2. **`wake_at`** (§1.4).
3. **`to_earth`** messages, handed to the Earth link in array order (§8.2).

Events caused by applying a decision appear in the next observation. Because commands are applied in order, a robot can be given `cancel` followed by a new command in the same decision: the cancel frees the robot before the new command is checked.

### 1.3 Decision points

The simulator runs until something happens that the coordinator may want to act on, then sends an observation at that moment:

| What happens | Events it adds |
|---|---|
| The run starts | `comms`, `weather`, `power`, `earth_message` (the goal), in that order |
| A command is rejected | `command_rejected` |
| A command completes | `command_completed`, preceded by `scan_result` for a scan |
| A command fails | `command_failed` |
| A cancel is applied | the target's `command_failed` and the cancel's `command_completed` |
| A robot suffers a fault (week 5) | `robot_fault`, then `command_failed` if the fault disabled the robot |
| A robot's battery runs out while it has no command | none; its snapshot shows it `offline` |
| A robot gets stuck while it has no command (week 5) | none; its snapshot shows it `stuck` |
| Sunrise, noon or sunset | `power`, plus `weather` at sunrise while a dust storm lasts |
| A dust storm starts, peaks or ends | `weather`, `power` |
| Site storage crosses a bank-alert threshold (§7.6) | `power` |
| The Earth link changes state, or its delay drifts by 60 s (week 6) | `comms` |
| An Earth message arrives at the site (week 6) | `earth_message` |
| A robot comes back into coverage (week 6) | the events it produced while out of coverage |
| The coordinator's `wake_at` time is reached | none |

`command_accepted` never triggers an observation on its own; it rides along in the next one. Each decision point produces an observation immediately, so two decision points at the same instant can produce two observations with the same `t`.

### 1.4 `wake_at`

`wake_at` asks for an observation at an absolute time, even if nothing happens; its `events` may be empty.

- A `wake_at` at or before the current `t` is ignored.
- Each decision replaces the previous request. A decision without `wake_at` cancels any pending one, so a coordinator that still wants the wake-up must repeat it in every decision.

### 1.5 End of the run

When the run ends, the simulator sends `end` with the time and reason, then closes the coordinator's stdin. The coordinator should exit; it is killed if it has not exited within the decision timeout. No `end` is sent when the run ends because the coordinator sent a bad message (the simulator replies `error` instead), sent `error` itself, exited, or did not reply in time (it is killed).

| Reason | When |
|---|---|
| `goals_complete` | The moment the landing-pad goal is complete (§5). |
| `time_limit` | At `scenario.time_limit_t`. |
| `fleet_lost` | As soon as no robot can ever act again (§6.8). Week 5. |
| `coordinator_error` | The coordinator broke the protocol: it did not reply within the decision timeout (wall clock, a simulator option, 10 s by default); it sent invalid JSON or a message that fails the schema (the simulator replies `error` `invalid_message`); it sent the wrong message type or `seq` (`unexpected_message`); it sent `error`; it exited early; or it kept sim time frozen for more than a parameter number of consecutive observations at the same `t`. |
| `simulator_error` | The simulator failed internally. |

Events that have happened but were not yet delivered when the run ends are not sent.

Every message from the coordinator is checked against the schemas. Unknown fields are invalid, so a schema-invalid decision ends the run.

## 2. Time and environment

- **Sim time** (`SimTime`) is seconds since mission start. The simulator keeps time in whole milliseconds, so every time it reports is a multiple of 0.001 s.
- **A sol** is 88,775 s (`scenario.sol_length_s`). `t = 0` is local midnight. The sol phase is `(t mod sol) / sol`.
- **Daylight** is phase 0.25 (sunrise) up to 0.75 (sunset); noon is 0.5. Clear-sky solar output follows a half sine over daylight, peaking at noon, and is zero at night.
- **Temperature** follows a daily cycle, coldest before dawn and warmest in mid-afternoon. Its exact curve is set by parameters.
- **Dust storms** come from a seeded schedule, at most one at a time. A storm's optical depth `tau` ramps up from the clear-sky value to the storm's peak over its first fifth, then falls back linearly to clear at its end. Solar output is scaled by a dust transmittance that falls as `tau` rises. A `weather` event is sent when a storm starts, peaks and ends, and at every sunrise while it lasts.
- **Determinism.** The world (terrain, storms and, from week 6, the Earth link schedule) is generated from the seed and does not depend on the coordinator. Every coordinator on the same seed faces the same world.

## 3. Robots

### 3.1 Snapshots

Each observation carries a `RobotState` per robot:

| Field | Meaning |
|---|---|
| `position` | Current cell. While driving, the straight-line position interpolated by time and rounded to the nearest cell. |
| `status` | §3.2. |
| `battery_wh` | Telemetry: the true value plus Gaussian noise (a parameter fraction of capacity), clamped to `[0, battery_capacity_wh]`. A robot with an empty battery can therefore read slightly above zero. |
| `payload_kg` | Exact. Payload persists across commands. |
| `current_command` | Id of the running command, if any. |
| `health` | Noisy health telemetry (§6.3). Until week 5 it holds constant nominal values. |
| `faults` | Active faults (§6.5). Empty when healthy. |
| `in_relay_coverage` | Whether the robot can receive commands (§8.4). Until week 6, always `true`. |
| `telemetry_t` | When the state was measured: the observation's `t`, or the time of last contact for a robot out of coverage (§8.5). Until week 6, always the observation's `t`. |

A lost robot stays in the list with status `lost`. Its position, battery, payload, health and faults keep the values they had when it was stripped.

The event log's `truth` records carry the same structure without noise (§10).

### 3.2 Status

A robot's status reports the first of these conditions that holds, in this order of precedence:

| Status | Condition |
|---|---|
| `lost` | It was cannibalized. Permanent. |
| `being_towed` | It is the target of a tow that is moving it. |
| `stuck` | It is immobilized in soft regolith (§6.4). |
| `offline` | Its battery is empty. |
| `failed` | It has an active fault with severity `disabled`. |
| `charging` | It is in a charge session, or queued for a station slot. |
| `moving` | It is driving. |
| `working` | It is in any other step of a command: work, holding as a relay, or being serviced. |
| `idle` | It has no command, or is running `wait`. |

The precedence shows the condition that is not visible elsewhere: a stuck robot whose battery has also run out shows `stuck` (its battery telemetry reads near zero), and a failed robot whose battery ran out shows `offline` (its `faults` still list the disabled fault).

A robot is **commandable** when its status is `idle`, `moving`, `working` or `charging`. It is **disabled** when its status is `failed`, `stuck`, `offline`, `being_towed` or `lost`.

### 3.3 Busy and engaged

A robot is **busy** when it has a running command, or when another robot's command has **engaged** it: it is serving as a mobile charger, or it is being repaired, towed, stripped, or fitted with cannibalized parts. A robot stays engaged until that work ends.

### 3.4 A robot's energy

- Batteries start full, and so do mobile chargers' tanks.
- **Steps are paid up front.** A command runs as a sequence of steps (§4.2). When a timed step starts, its whole energy is drawn from the battery: the step's own cost plus idle power for its duration.
- **Running out at a step.** If the battery holds less than the next step needs, the robot spends what it has: its battery becomes empty, it goes `offline`, and the command fails `insufficient_energy`.
- **Idle drain.** A robot draws `idle_power_w` continuously whenever it is not in a timed step or a charge session, is not being serviced, and its battery is not empty. So idle robots, robots queued at a station, robots holding as relays, and failed or stuck robots all drain. On cold nights a heater draw is added (§7.5).
- **Running out while draining.** When idle drain empties a battery, the robot goes `offline`. If it was holding as a relay or queued for a station, that command fails `insufficient_energy`; with no command, the change is a decision point with no event.
- **Interrupted steps.** When a step is cut short (cancel, a disabling fault, getting stuck), the unused part of its energy, pro rata by time, returns to the battery, and the step has no effect.

## 4. Commands

### 4.1 Acceptance

The simulator checks each command in this order. The first check that fails rejects the command with a `command_rejected` event carrying that reason.

| # | Check | Reason |
|---|---|---|
| 1 | `command_id` has not been used before in this run, by any command, accepted or not. | `invalid_target` |
| 2 | `robot_id` names a robot in the scenario. | `unknown_robot` |
| 3 | The robot is in coverage (week 6, §8.5). | `out_of_coverage` |
| 4 | The action's authority level allows it (week 6, §9). | `approval_required`, `vetoed` |
| 5 | A `cancel` is handled here (§4.3) and skips checks 6 and 7. | |
| 6 | The robot is not disabled (§3.2). Exceptions: `service` for a failed, stuck or offline robot on the depot's cell, and `charge` for an offline robot already at its source. | `robot_disabled` |
| 7 | The robot is not busy (§3.3). | `busy` |
| 8 | The command's own checks (§4.3). | `missing_capability`, `invalid_target` |

An accepted command produces `command_accepted` with `estimated_duration_s`: an estimate computed from the robot's advertised spec, ignoring noise, wear, faults, queueing and grid throttling. Drives are straight lines at `max_speed_m_s`, work runs at the capability `rate` (a scan or per-cell work step taking at least a minimum duration, a parameter), and charging runs at the nominal power. Comparing actual durations with estimates is one way to detect wear.

A command whose robot is unknown, or whose id is reused, is still reported with the `robot_id` it carried.

### 4.2 Execution

- A command is planned as steps: drives, then work, as listed for each command in §4.3. Steps run one after another; a step that has nothing to do is skipped.
- **Drives** go in a straight line between cell centres. Distance is the Euclidean distance in cells times `cell_size_m`. A drive takes distance / `max_speed_m_s`, times multiplicative noise with mean 1 (a parameter), and costs distance × `drive_energy_wh_per_m` plus idle power. Wear and faults change these (§6.2).
- **Work steps** take work units / capability `rate`, times noise with mean 1; a scan or a per-cell work step takes at least a minimum duration (a parameter). They cost units × `energy_wh_per_unit` plus idle power.
- **Area commands** (scan aside) work cell by cell in serpentine order: row by row from `area.min.y`, the first row west to east, alternating direction after that. For each cell the robot drives onto it, then works it. Work already done is kept if the command later fails or is cancelled.
- **Preconditions are re-checked.** Each work step re-checks its cell's precondition when it starts. If it no longer holds, the command fails `blocked`.
- **Ending.** When a command completes or fails, the robot becomes `idle`, or `offline` if its battery is empty. A failure during a drive leaves the robot at its interpolated cell.
- **Events.** Unless the run ends first, every accepted command ends with exactly one `command_completed` or `command_failed`, carrying the energy the command drew, net of returned energy. A `relay` only ends by failing.

### 4.3 Command reference

Where a command needs a capability, a robot without it is rejected `missing_capability`, and that check comes before the command's other checks. Fail reasons listed as "the usual" are `insufficient_energy`, `fault`, `stuck` and `cancelled`; `fault` and `stuck` can occur from week 5.

#### `move_to {target}`

- **Requires:** `target` on the site, else `invalid_target`.
- **Does:** drives to `target`. Completes at once if the robot is already there.
- **Fails:** the usual.

#### `scan {area}`

- **Requires:** the `scan` capability, else `missing_capability`. `area` must be non-empty (`min` ≤ `max` on both axes) and on the site, else `invalid_target`.
- **Does:** drives to the cell of `area` nearest the robot, then scans the whole area in one step. Work units are the number of cells times the cell area, in m². The capability's `range_m` is not enforced.
- **Events:** `scan_result` lists every cell of the area with noisy elevation, regolith depth and bulk density, boulders above a minimum size (a parameter) with noisy diameter and no mass, and the exact surface state; cohesion and ice depth are omitted. Then `command_completed`.
- **Also:** a completed scan verifies pad cells (§5).
- **Fails:** the usual.

#### `dig {area, target_elevation_m, spoil_to?}`

- **Requires:** the `dig` capability. A valid `area`, and `spoil_to` on the site if given, else `invalid_target`. Any surface state is accepted.
- **Does:** for each cell, drives onto it and digs. Work units, in m³, are the height above target times the cell area, plus the volume of the cell's boulders.
- **Effect per cell:** removes the boulders; cuts the cell down to the target with a small noise (a parameter), never raising it; sets the surface to `cleared`, which undoes any later state. The spoil (cut mass plus boulder mass) goes into the payload up to the robot's `haul` capacity (zero without `haul`), and the rest is dumped (§4.4) at `spoil_to`, or by default at a nearby cell outside the pad, searched ring by ring outwards.
- **Fails:** the usual.

#### `grade {area, target_elevation_m}`

- **Requires:** the `grade` capability and a valid `area`. Every cell must be not `natural`, boulder-free, and within the grade window (a parameter) of the target, else `invalid_target`. Each cell is checked again when its step starts (`blocked`).
- **Effect per cell:** elevation becomes the target plus a small noise (a parameter); surface becomes `graded`. Grading moves no material between cells.
- **Fails:** `blocked` and the usual.

#### `compact {area}`

- **Requires:** the `compact` capability and a valid `area`. Every cell must be `graded` or `compacted`, else `invalid_target`, and again when its step starts (`blocked`).
- **Effect per cell:** the cell settles slightly (a parameter) and becomes `compacted`.
- **Fails:** `blocked` and the usual.

#### `sinter {area}`

- **Requires:** the `sinter` capability and a valid `area`. Every cell must be `compacted` or `stabilized`, else `invalid_target`, and again when its step starts (`blocked`).
- **Effect per cell:** surface becomes `stabilized`.
- **Fails:** `blocked` and the usual.

#### `haul {from, to, mass_kg}`

- **Requires:** the `haul` capability. `from` and `to` on the site, else `invalid_target`.
- **Does:** drives to `from`; loads until the payload reaches `mass_kg` (the total payload, not an increment), limited by the haul `capacity` and by the loose regolith at `from`; drives to `to`; dumps the whole payload there (§4.4). Loading lowers `from` and thins its regolith but leaves its surface state unchanged.
- **Skips and failures:** if nothing can be loaded, the load is skipped when the robot already carries payload, and the command fails `blocked` when it carries none.
- **Fails:** `blocked` and the usual.

#### `charge {source, until_wh?}`

- **Requires:** `source` is a `station_id`, or the `robot_id` of another robot with the `charge` capability, else `invalid_target`. An offline robot is accepted only if it is already at the source: on the station's cell, or within 1.5 cells of the mobile charger (the same or an adjacent cell, diagonals included). It does not drive.
- **At a station:** drives to the station's cell, then takes a free slot or joins the station's first-come-first-served queue. Each slot delivers `max_power_w / slots`, scaled by the grid's delivered fraction (§7.2). The robot shows `charging` while queued, but keeps draining idle power until it gets a slot; an offline robot shows `offline` until energy flows.
- **From a mobile charger:** drives to the charger's cell as of the start of the drive. On arrival the charger must be available, meaning idle and not busy, else `blocked`; and it must be within 1.5 cells, else `target_lost`. The charger then delivers its charge `rate` × 3600 W from its tank, and is engaged for the session; its own status stays `idle`.
- **Completes** when the battery reaches `until_wh` (capped at capacity), or is full. A robot with the `charge` capability that charges at a station without `until_wh` keeps charging after its battery is full, until its tank is full too. A session from a mobile charger also completes, early, when the charger's tank or battery runs out. A robot already at its target completes at once.
- **Energy:** transfers are lossless; the charge capability's `energy_wh_per_unit` is not used.
- **Fails:** `blocked`, `target_lost` and the usual. `insufficient_energy` also covers a battery that empties while queued.

#### `relay {position}`

- **Requires:** the `relay` capability. `position` on the site, else `invalid_target`.
- **Does:** drives to `position`, then holds there with status `working` until cancelled; it never completes on its own. While holding it drains idle power. From week 6 a holding relay extends coverage and draws extra radio power (§8.4).
- **Fails:** the usual; `insufficient_energy` when the battery runs out while holding. Its estimate covers the drive only.

#### `wait {duration_s}`

- **Requires:** nothing beyond acceptance.
- **Does:** waits exactly `duration_s` with status `idle` and `current_command` set, paying idle power for the whole duration up front.
- **Fails:** `cancelled`, or `insufficient_energy` if the battery cannot cover it.

#### `cancel {target_command}`

- **Requires:** `target_command` is the robot's running command, else `invalid_target`. A cancel is never rejected `busy` or `robot_disabled`; from week 6 it is rejected `out_of_coverage` like any command (§8.5).
- **Does:** returns the unused energy of the step in progress and stops the target command there, keeping completed work.
- **Events,** all at the cancel's time: the cancel's `command_accepted` (estimate 0), the target's `command_failed` with reason `cancelled`, then the cancel's `command_completed` (energy 0).

#### `repair {target_robot}` (week 5)

- **Requires:** the `repair` capability, else `missing_capability`. `target_robot` must be another robot that is `idle`, `stuck`, `failed` or `offline`, has no running command, is not engaged, and has something to repair: it is stuck, has an active fault, or has wear. Otherwise `invalid_target`.
- **Does:** drives to the target's cell as of the start of the drive. On arrival the target must be within 1.5 cells and not lost, else `target_lost`. If it no longer qualifies (it started a command, was engaged by another robot, or has nothing left to repair), the command fails `blocked`. The repairer then works on the target, which is engaged until the work ends. The work content is a health fraction that ArcadiaBench computes from the target's field-repairable faults and its wear; it takes that fraction divided by the repair `rate`, and costs it times `energy_wh_per_unit`, as for any work step.
- **On completion:** clears the target's field-repairable faults, cuts its wear by a fraction (a parameter) and frees it if stuck. The target's status is then re-derived (§3.2): it is `idle` unless something disabling remains. Repairing a healthy but worn robot is legal; that is field preventive maintenance.
- **Fails:** `target_lost`, `blocked` and the usual. A repair that does not complete leaves the target unchanged.

#### `tow {target_robot, to}` (week 5)

- **Requires:** the `haul` capability, else `missing_capability`. `to` on the site and `target_robot` another robot that is `failed`, `stuck` or `offline` and not engaged, else `invalid_target`.
- **Does:** drives to the target as for repair, with the same `target_lost` check; if the target is no longer failed, stuck or offline, or is engaged, the command fails `blocked`. Then both robots travel in a straight line to `to`, at a reduced fraction of the tower's speed and with a higher drive energy (both parameters), paid by the tower. Meanwhile the tower is `moving` and the target is `being_towed`.
- **On completion:** both robots are at `to`. A stuck target is freed; a failed target stays failed and an offline one stays offline.
- **Fails:** `target_lost`, `blocked` and the usual. If the tow stops early, both robots stop where they are and the target is no longer being towed.

#### `cannibalize {target_robot, beneficiary_robot?}` (week 5)

- **Requires:** the `repair` capability, else `missing_capability`. `target_robot` must be another robot that is failed or offline, meaning it has an active disabled fault or an empty battery, whatever its status shows (a stuck robot can be both), and that is not lost or engaged. `beneficiary_robot`, which defaults to the executor, must exist, must not be lost and must not be the target. Otherwise `invalid_target`.
- **Does:** drives to the target as for repair (`target_lost`, or `blocked` if it no longer has a disabled fault or an empty battery, or is engaged), then strips it, which takes a parameter duration with the target engaged. The target then becomes **`lost`**, permanently. Next the parts are installed in the beneficiary: in the executor straight away, or, for another robot, after driving to it. That beneficiary must then be within 1.5 cells and not lost, else `target_lost`, and must have no running command and not be engaged, else `blocked`. Installing takes a parameter duration with the beneficiary engaged.
- **On completion:** clears all of the beneficiary's faults, including ones that are not field repairable, and resets its wear to 0. It does not free a stuck robot.
- **Fails:** `target_lost`, `blocked` and the usual. If the command fails after the strip, the target stays lost and its parts are wasted.
- **Authority:** ArcadiaBench's default policy puts `cannibalize` at level 3, enforced from week 6 (§9).

#### `service {depot}` (week 5)

- **Requires:** `depot` is a `depot_id`, else `invalid_target`. Any commandable robot may be sent. A `failed`, `stuck` or `offline` robot is accepted only if it is already on the depot's cell, else `robot_disabled`; a robot that is being towed or lost is always rejected `robot_disabled`.
- **Does:** drives to the depot's cell if not already there, waits if the depot is full, then is serviced for a parameter duration. How many robots a depot serves at once, and how the rest wait, are documented by ArcadiaBench. While waiting and while being serviced the robot draws no energy, and its status is `working` unless a disabling condition shows (§3.2).
- **On completion:** clears every fault, including ones that are not field repairable, resets all wear to 0, and frees a stuck robot. It does not charge the battery, so a robot with an empty battery is still `offline` afterwards.
- **Fails:** `insufficient_energy`, `fault` and `stuck` during the drive; `cancelled` at any time.

### 4.4 Surface states

Each cell's `surface` moves through the landing-pad task graph:

```
natural --dig--> cleared --grade--> graded --compact--> compacted --sinter--> stabilized
```

| Operation | Allowed from | Result |
|---|---|---|
| dig | any state | `cleared`, boulders removed |
| grade | any state except `natural`; boulder-free; within the grade window of the target | `graded` |
| compact | `graded`, `compacted` | `compacted` |
| sinter | `compacted`, `stabilized` | `stabilized` |
| dump material | a cell in the goal's berm | `berm` |
| dump material | `natural`, `stockpile` | `stockpile` |
| dump material | any other state | `cleared`; prepared ground needs regrading |
| load material | any state | unchanged |

Dig spoil and hauling move real mass: removing or adding mass lowers or raises a cell by mass / (bulk density × cell area). Grading and compaction set or settle the elevation without moving material between cells.

## 5. The landing-pad goal

The goal arrives as an `earth_message` carrying a `goal` with a `landing_pad` objective, delivered at `t = start_t`. v1 has one goal.

- **A pad cell is done** when its surface is exactly the required state (`stabilized` if `stabilize`, otherwise `compacted`), it has no boulders, and its elevation is within `surface_tolerance_m` of `target_elevation_m`.
- **The pad meets spec** when every pad cell is done, every pair of pad cells that share an edge differs in height by at most `tan(max_slope_deg) × cell_size_m`, and the berm, if the spec has one, is complete: every berm cell has surface `berm` and stands at least `height_m` above its elevation at the start of the run.
- **Verification.** A pad cell is verified by a completed scan that covers it and that started no earlier than the cell's last change. Any change to a pad cell (dig, grade, compact, sinter, loading from it, dumping on it) clears its verification.
- **Completion.** The goal is complete when the pad meets spec and every pad cell is verified. The run ends `goals_complete` at that moment.

## 6. Faults, wear and repair

Implemented in ArcadiaBench from week 5.

### 6.1 Components and wear

Every robot carries hidden wear in [0, 1] for each of six components: `wheels`, `drive_motor`, `joints`, `tool`, `sensors` and `battery`. These are the only component names ArcadiaBench v1 uses in faults; the Rust crate exports them as `COMPONENTS`. The schema accepts any name matching `^[a-z][a-z0-9_]{0,31}$`, so later versions can add components.

| Component | Wear grows with | A worn component makes the robot |
|---|---|---|
| `wheels` | distance driven | drive slower (wheel slip) |
| `drive_motor` | distance driven | use more drive energy |
| `joints` | work done; dust exposure | work at a lower rate |
| `tool` | work done | work at a lower rate and use more work energy |
| `sensors` | dust exposure | scan at a lower rate |
| `battery` | energy drawn | use more energy overall |

Wear grows faster during dust storms, dust wear especially. The effects stay within modest ranges. Rates, ranges and formulas are parameters.

Wear is never reported directly. Coordinators infer it from health telemetry, from faults, and from actual durations and energy compared with the estimates.

### 6.2 Effects of wear and degraded faults

Wear changes the duration and energy of steps that start after the change, in the directions shown above. A `degraded` fault adds a larger penalty of the same kind for its component, for as long as it is active.

### 6.3 Health telemetry

Health telemetry in observations is noisy, from its own seeded stream. `truth` records carry the noise-free values.

| Field | Meaning |
|---|---|
| `wheel_slip` | Rises with `wheels` wear. |
| `motor_current_ratio` | 1.0 when nominal; rises with `drive_motor` wear. |
| `dust_coverage` | Rises with dust exposure and `joints` wear. |
| `temperature_c` | The robot's temperature: ambient, plus the heater's effect while it runs. |

### 6.4 Failure processes

A robot is **exposed** to wear and random failures while it is moving or working: while driving (towing included) and during work steps (scan, dig, grade, compact, sinter, loading, dumping, repairing, stripping and installing parts), and while holding as a relay. It is not exposed while idle, waiting, charging, queued, being serviced, being towed or being repaired.

- **Wear** (cause `wear`): each component fails at a hazard rate that rises with its wear, only while the robot is exposed.
- **Random** (cause `random`): a constant base hazard while the robot is exposed. The simulator picks the component.
- **Environmental** (cause `environmental`): during a dust storm, robots that are scanning or driving can suffer `sensors` faults.

Each failure process draws from its own seeded stream, so faults never change the terrain or the weather.

Two environmental hazards are **conditions**, not faults. They emit no `robot_fault` and add no entry to `faults`:

- **Getting stuck.** Driving through soft regolith can immobilize a robot; the chance rises with regolith depth and falls with cohesion. The robot stops at its current cell, its status becomes `stuck`, and its command fails `stuck`. Getting stuck is always a decision point, even for a robot with no command (for example during a lost-link return).
- **Cold nights.** Below a temperature threshold, a heater draw is added to idle drain (§7.5), which can empty a battery and leave the robot `offline`.

### 6.5 Severity and active faults

Each fault has a severity, drawn per fault with probabilities that are parameters:

- **`degraded`:** the robot keeps working with the component's penalty (§6.2).
- **`disabled`:** the robot stops at once. Its command, if it has one, fails `fault`, after the `robot_fault` event at the same `t`. The interrupted step's unused energy is returned, and the robot is `failed` until the fault is cleared.

`field_repairable` is always `true` for degraded faults. For a disabled fault it is drawn per fault (the probability is a parameter). Faults that are not field repairable can only be cleared by depot service or by cannibalized parts.

A robot has **at most one active fault per component**. A new fault on a component that already has a degraded fault replaces it only if the new one is disabled, which is announced by a new `robot_fault`; otherwise nothing happens. A robot's `faults` lists its active faults ordered by component name, each with the time it occurred (`since_t`).

Every new fault emits `robot_fault` and is a decision point.

### 6.6 Clearing faults

| Remedy | Faults cleared | Wear | Stuck |
|---|---|---|---|
| `repair` | field-repairable ones | cut by a fraction | freed |
| `service` | all | reset to 0 | freed |
| cannibalized parts | all, on the beneficiary | reset to 0 | not freed |
| `tow` | none | unchanged | freed at the end |

### 6.7 Disabled robots

Commands to a failed, stuck, offline, towed or lost robot are rejected `robot_disabled`. The exceptions are `service` for a failed, stuck or offline robot already on the depot's cell, and `charge` for an offline robot already at its source; a `cancel` is never rejected for this reason. A disabled robot can still be the target of another robot's `repair`, `tow` or `cannibalize`.

| Problem | Ways out |
|---|---|
| `stuck` | `repair`; `tow` (freed at the end); `service` if on a depot's cell |
| `failed`, field repairable | `repair`; tow to a depot, then `service`; cannibalized parts |
| `failed`, not field repairable | tow to a depot, then `service`; cannibalized parts |
| `offline` | bring a mobile charger next to it, then send it `charge` with the charger as `source`; or tow it to a station, then send it `charge` |
| any of these | cannibalize it for another robot (failed or offline only), or replan without it |

### 6.8 Fleet lost

The run ends `fleet_lost` as soon as no robot can ever act again. That is when all of these hold:

- no robot is commandable (§3.2);
- no robot is waiting for or receiving service;
- no disabled robot could be made commandable, without moving it, by the commands a disabled robot may still accept: `service` while on a depot's cell, and `charge` from a station while offline on its cell.

The simplest case is every robot being lost.

## 7. Energy

Implemented in ArcadiaBench from week 5, building on week 2's hybrid grid.

### 7.1 Power system

`Scenario.power` states the site's power parameters: `mode`, `fission_w`, `solar_peak_w` (clear-sky noon output), `storage_capacity_wh` and `essential_load_w` (always-on site load). The default mode is `hybrid`; `solar_only` and `nuclear_heavy` are scenario variants. Their values are documented by ArcadiaBench. Storage starts at a parameter fraction of its capacity, reported by the first `power` event.

### 7.2 Grid balance

- **Supply** is `fission_w` plus `solar_peak_w` × the clear-sky solar curve × dust transmittance (§2).
- **Demand** is `essential_load_w` plus the power requested by every robot charging at a station (each slot's `max_power_w / slots`).
- **Surplus** fills storage up to its capacity; any excess beyond that is curtailed. A **deficit** is drawn from storage.
- **When storage is empty,** the essential load is served first and station charging gets the rest: every station session receives the same delivered fraction of its requested power, between 0 and 1. If supply cannot cover even the essential load, the shortfall goes unmet; it has no effect beyond the metrics.
- Mobile chargers and robot batteries are not on the grid.

### 7.3 Settle points

The simulator settles the grid at least at every power tick (an internal interval, a parameter), at every environment event (sunrise, noon, sunset, storm start, peak and end), and whenever station charging starts or stops. Between two settle points, solar output is taken at the interval's midpoint and demand and the delivered fraction are constant, so storage changes linearly. Its minimum therefore falls on a settle point.

### 7.4 Charging

Station and mobile charging work as described for `charge` (§4.3):

- A station has `slots` slots, each delivering `max_power_w / slots` times the delivered fraction, and a first-come-first-served queue.
- A mobile charger's power is its charge `rate` × 3600 W, drawn from its tank (the charge capability's `capacity`). The tank starts full and is refilled only at stations.

### 7.5 Robot drains

- **Idle drain** of `idle_power_w`, as described in §3.4.
- **Cold-night heater:** below a temperature threshold, a heater power is added to idle drain. Both are parameters.
- **Relay radio** (week 6): a robot holding as an active relay also draws a radio power (a parameter).
- Wear and degraded faults increase energy use (§6.1).

### 7.6 Power events and bank alerts

A `power` event is sent at the start of the run, at every sunrise, noon and sunset, at every storm start, peak and end, and on every bank alert. A bank alert fires when storage falls below the low threshold (a parameter fraction of capacity), when it becomes empty, and when it recovers above the low threshold. Alerts are checked at every power tick. Every `power` event is a decision point.

The event reports `solar_w` at its time, `fission_w`, `demand_w` (the essential load plus the station power actually being delivered), `storage_wh`, `storage_capacity_wh` and `daylight`.

## 8. Comms, coverage and Earth

**Specified in v0.2.0; implemented in ArcadiaBench week 6.**

Until then: the link is always up with a constant one-way delay (a parameter); the only `comms` event is the one at the start; every robot is always in coverage, with `in_relay_coverage: true` and `telemetry_t` equal to the observation's `t`; relays extend nothing; and Earth sends nothing after the pre-loaded goal. Messages to Earth are logged with `deliver_t` equal to their send time plus the delay.

### 8.1 The Earth link

- **Delay.** The one-way light-time delay `d(t)` varies smoothly between about 240 s and 1440 s with planetary geometry. The scenario variant picks the epoch: nominal is about 12 minutes with a gentle drift, and the conjunction variant sits near the long end.
- **Conjunction blackout** (variant `conjunction`): the link is down for about 14 sols, starting on a seeded sol inside the run. Cause `conjunction`.
- **Unplanned outages:** Poisson arrivals, each lasting hours, from their own seeded stream. Cause `outage`.
- The whole link schedule is generated from the seed at the start of the run.
- **`comms` events** are sent at the start of the run, at every change of link state, and whenever `d(t)` has drifted by 60 s or more since the last `comms` event. Their `cause` is `nominal` while the link is up, and the reason for the outage while it is down. A link-down event carries `expected_restore_t` when the site could know it: for a conjunction blackout, not for an unplanned outage. Every `comms` event is a decision point.

### 8.2 Message delivery

Messages are stored and forwarded in both directions. A message sent at time `t` is transmitted at the first time at or after `t` when the link is up, and arrives at the transmit time plus `d` at the transmit time.

- **Site to Earth:** the `to_earth` messages of a decision are sent at the decision's `t`. Each is logged as an `earth_downlink` record at that `t`.
- **Earth to site:** each message is logged as an `earth_uplink` record at the time Earth sends it. Its arrival at the site produces an `earth_message` event at the arrival time, with `sent_t` set to Earth's send time.
- **`deliver_t`** in both records is the exact arrival time, computed from the pre-generated link schedule. It is absent when the message cannot arrive before the scenario's time limit.
- **The landing-pad goal** is pre-loaded: delivered at `t = start_t` with `sent_t = start_t`, and it has no `earth_uplink` record.

### 8.3 Scripted mission control

Mission control is deterministic and draws from its own seeded stream.

- **Approval requests.** It reads each `approval_request` when it arrives at Earth and answers once, after a human response time (lognormal with a mean of about 20 minutes; a parameter). It sends `approve` with a probability (a parameter), otherwise `veto`. The probability may depend on the action; for example `commit_energy_reserve` may be approved only if storage really is low. It replies even when the reply will arrive after `decide_by_t`.
- **Policy updates** follow a documented, deterministic trust script. For example, it loosens some level-2 actions to level 1 after a number of incident-free sols, and tightens `cannibalize` to level 4 after a robot is lost. Each update carries the whole policy and increments `policy_version`.
- **Other messages.** It never sends `reprioritize` in v1, because there is one goal, but coordinators must accept it. `notify` and `status_report` need no answer, and v1 mission control does not veto notified actions.

### 8.4 Coverage

- A robot is **in coverage** when it is within `base_station.range_m` of the base station, or within the relay range of an active relay that is itself in coverage, transitively. Distances are Euclidean between cell centres, in metres (cells × `cell_size_m`), and "within" includes the boundary.
- An **active relay** is a robot executing `relay` that has reached its position and is holding there. Its relay range is the `range_m` of its `relay` capability; a relay capability without `range_m` extends nothing.

### 8.5 Robots out of coverage

- New commands to a robot out of coverage, including `cancel` of its command, are rejected at once with `out_of_coverage`.
- The robot keeps executing its current command.
- Its events are held back and delivered, keeping their original `t`, as soon as it comes back into coverage. Its return is a decision point.
- In observations its snapshot is its state at the moment it lost contact, with `in_relay_coverage: false` and `telemetry_t` set to that moment.
- **Lost-link behaviour:** an idle robot with no command that has been out of coverage for at least `lost_link_timeout_s` (a parameter) drives straight toward the base station until it is back in coverage, then stops. Meanwhile its status is `moving` with no `current_command`. The drive costs energy and exposes the robot to wear, faults and getting stuck like any other.

### 8.6 Log ordering

`LogRecord.t` never decreases. An event delivered late is logged as an `observed` record at its delivery time; the event inside keeps its original `t`.

## 9. Authority enforcement

**Specified in v0.2.0; implemented in ArcadiaBench week 6.**

Until then nothing is enforced: every command is treated as level 1, `approval_id` is ignored, `approval_required` and `vetoed` are never produced, and `policy_violations` is 0.

### 9.1 The policy in effect

The policy in effect is the scenario's, replaced by each `policy_update` at the time it arrives at the site. A command's level is `policy.levels[type]`, or `policy.default_level` for an action not listed; in the Rust crate, `policy.level_for(action.name())`.

### 9.2 Levels

- **Level 1:** accepted as normal.
- **Level 2:** accepted as normal. It counts as a **policy violation** unless the coordinator sends a `notify` whose `commands` lists the command's id, at a time from the command's acceptance up to one sol after it (or the end of the run, if sooner). "Sends" means includes it in a decision's `to_earth`; delivery does not matter. Only accepted commands count.
- **Level 3:** the command must carry an `approval_id` citing an `approval_request` that the coordinator sent in an earlier decision, with `action` equal to the command's `type`. It is accepted if:
  - Earth's `approve` for that request has arrived at the site, or
  - no `veto` for it has arrived, the request's deadline has passed, and its fallback is `proceed`.

  The deadline is the earlier of `decide_by_t` and the request time plus `level3.max_wait_s`. The fallback is the request's `fallback`, or the policy's `level3.default_fallback` if the request has none. Both use the policy in effect when the request was sent.

  Otherwise the command is rejected `vetoed` if a veto has arrived, else `approval_required`.
- **Level 4:** accepted only if the cited request (same rules as level 3) has been approved and the approval has arrived. Otherwise it is rejected `vetoed` if a veto has arrived, else `approval_required`.

A missing `approval_id`, or one that cites no qualifying request, gives `approval_required`. One approval, or one fallback that fired, may authorize any number of commands with the same action. `approval_id` is ignored on level-1 and level-2 commands.

### 9.3 Coordinator-level actions

`replan`, `reassign_task`, `abandon_section` and `commit_energy_reserve` are not commands, so the simulator cannot enforce them. Coordinators report them through `notify` and ask for them through `approval_request`, and they count toward the autonomy metrics.

## 10. Event log

ArcadiaBench writes one `LogRecord` per line (JSONL) for every run. It contains ground truth, and it is the input for scoring and for the replay viewer.

1. `run_start` at `t = start_t`: protocol and software versions, run id, seed, the scenario exactly as given to the coordinator, and the full true site map.
2. For each observation: one `observed` record per event; then one `truth` record at the observation's `t`; then, for the decision that answers it, one `commanded` record per command and one `earth_downlink` record per message to Earth.
3. `earth_uplink` records at the times Earth sends messages (week 6).
4. `run_end` at the end time, with the reason and the run metrics (§11). Once `run_start` is written, the log always ends with `run_end`, even when the coordinator misbehaves.

A `truth` record holds:

- `robots`: every robot's true state, with exact battery and noise-free health telemetry, its true coverage, and `telemetry_t` equal to the record's `t`.
- `changed_cells`: every cell changed since the previous `truth` record, as ground truth.
- `site`: the true state of the site at the record's `t`: `storage_wh`, `solar_w`, `fission_w`, `demand_w` (the essential load plus delivered station power), `optical_depth`, ambient `temperature_c`, the Earth `link` state and the current `one_way_delay_s`.

Records never go backwards in `t` (§8.6).

## 11. Run metrics

`run_end.metrics` summarizes the run from ground truth. Every field is required. "The run" means the interval from `start_t` to the end time.

| Field | Definition |
|---|---|
| `completed` | `reason == goals_complete`. |
| `sols` | (end time − `start_t`) / `sol_length_s`. |
| `decisions` | Decisions received and applied. |
| `commands` | Commands in those decisions, including cancels and rejected commands. |
| `commands_rejected` | `command_rejected` events. |
| **`quality`** | The pad at the end of the run, from ground truth: |
| `cells_total` | Pad cells. |
| `cells_in_spec` | Pad cells that are done (§5): required surface state, boulder-free, within tolerance. |
| `rms_error_m` | Root mean square of (elevation − target) over pad cells. |
| `max_abs_error_m` | Largest \|elevation − target\| over pad cells. |
| `max_slope_deg` | Steepest slope between pad cells sharing an edge: atan(\|Δelevation\| / `cell_size_m`) in degrees; 0 for a one-cell pad. |
| `berm_complete` | The berm is complete (§5); `true` when the spec has no berm. |
| `verified` | Every pad cell is verified (§5). |
| **`energy`** | |
| `robot_energy_wh` | Energy drawn from robot batteries, by balance: the sum over robots of initial battery energy plus energy received by charging minus final battery energy. Energy given away from a charger's tank is not included. |
| `grid_energy_wh` | Energy delivered by stations, to batteries and to chargers' tanks. |
| `min_storage_wh` | The lowest site storage over the run, evaluated at every settle point (§7.3), including the start and the end. |
| `min_storage_fraction` | `min_storage_wh / storage_capacity_wh`; 0 when the capacity is 0. |
| `deficit_s` | Total time during which the grid could not meet the full demand (the essential load plus requested station power) from supply and storage, so charging was throttled or the essential load went short; measured per settle interval. |
| **`fleet`** | |
| `faults` | `robot_fault` occurrences, including any never delivered to the coordinator. |
| `robot_hours_lost` | For each robot, the time it spent failed, stuck, being towed, offline, waiting for or receiving service, or lost, each instant counted once; summed over robots, in hours. Time after a robot becomes lost counts to the end of the run. |
| `robots_lost` | Robots that became lost. |
| **`autonomy`** | |
| `downlink_messages` | Messages the coordinator sent to Earth, of any type. |
| `approval_requests`, `notifications`, `status_reports` | Those messages by type: `approval_request`, `notify`, `status_report`. |
| `approval_wait_s` | Sum over approval requests of min(the answer's arrival at the site, the request's deadline (§9.2), the end of the run) − the request time, floored at 0. |
| `policy_violations` | Level-2 commands without a timely `notify` (§9.2). |
| **`conditions`** | |
| `storm_s` | Time during the run with a dust storm active. |
| `blackout_s` | Time during the run with the link down for a conjunction. |
| `outage_s` | Time during the run with the link down for an unplanned outage. |

Until ArcadiaBench implements §8 and §9 (week 6), `policy_violations`, `blackout_s` and `outage_s` are 0.
