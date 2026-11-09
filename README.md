# arcadia-protocol

The versioned interface spec between a **coordinator** (such as [ArcadiaOS](https://github.com/Gabe-Soler/arcadiaos)) and the **ArcadiaBench** simulator ([ArcadiaBench](https://github.com/Gabe-Soler/arcadiabench)). The two projects never import each other's code. They only share this spec, so any team can plug a coordinator written in any language into ArcadiaBench.

**Protocol version: 1.1.0**

The schemas define the messages; **[docs/semantics.md](docs/semantics.md)** defines what they mean. It is the normative description of the behaviour behind the interface: decision points, time, every command's preconditions, effects, events and failure reasons, faults and repair, energy, comms and coverage, authority enforcement, and the run metrics.

## Who talks to whom

ArcadiaOS runs **on Mars**, at the construction site. Commands to robots are local and instant. The light-time delay of many minutes, the pass windows, the multi-week conjunction blackouts and the random outages all sit on the **Earth link**, between mission control and the coordinator. ArcadiaBench simulates both the site and a scripted mission control, and routes Earth messages through its delay model.

```
  Earth (simulated)                    Mars site
 ┌──────────────────┐  EarthToSite   ┌───────────────────────────────────┐
 │  mission control │ ─────────────► │  coordinator (ArcadiaOS, ...)     │
 │  goals, approve, │   delayed,     │        ▲ observation  │ decision  │
 │  veto, policy    │ ◄───────────── │        │ (events,     ▼ (commands,│
 └──────────────────┘  SiteToEarth   │  ArcadiaBench site    to_earth)   │
                                     │  robots, terrain, power, weather  │
                                     └───────────────────────────────────┘
```

## The loop

The coordinator runs as a **subprocess** of the simulator and speaks **newline-delimited JSON** on stdin/stdout, one message per line. stderr is free for logs.

```
sim → coord   hello        protocol version, run id, seed, scenario (prior map, robots, stations, depots, power, base station, policy, constants)
coord → sim   welcome      protocol version, coordinator name/version
              ┌─ repeat ───────────────────────────────────────────────────────────────┐
sim → coord   │ observation  seq=n, t, events since last observation, robot snapshot   │
coord → sim   │ decision     seq=n, robot commands, messages for Earth, optional wake_at│
              └────────────────────────────────────────────────────────────────────────┘
sim → coord   end          reason (goals_complete, time_limit, ...)
```

- **Deterministic:** sim time does not advance while the simulator waits for a decision, so a run is fully reproducible from its seed.
- **Decision points:** the simulator sends an observation when a command completes, fails or is rejected; a fault happens; weather, power or comms change; an Earth message arrives; a robot comes back into coverage; or the coordinator's `wake_at` time is reached. [docs/semantics.md](docs/semantics.md#13-decision-points) has the full list.
- **Goals arrive over the Earth link.** They're not in the scenario. The first goal is delivered as an `earth_message` event at the start of the run.
- **Errors:** either side may send `error` and exit.

## Schemas

All schemas use JSON Schema **draft 2020-12** and live in [`schemas/`](schemas/). They are the source of truth.

| File | Defines |
|---|---|
| `common.schema.json` | `Id`, `Semver`, `SimTime`, `Duration`, `GridCoord`, `GridRect` |
| `site_map.schema.json` | `SiteMap`, `Cell` (elevation, regolith, boulders, surface state, ice) |
| `capability.schema.json` | `CapabilityName`, `CapabilitySpec`, `RobotSpec`, `ChargeStation`, `Depot`, `PowerSystem` (`PowerMode`), `BaseStation` |
| `robot_state.schema.json` | `RobotState`, `HealthTelemetry` (noisy wear indicators), `ActiveFault` |
| `command.schema.json` | `Command`: move_to, scan, dig, grade, haul, compact, sinter, repair, tow, cannibalize, service, charge, relay, wait, cancel |
| `event.schema.json` | `Event`: command accepted/rejected/completed/failed, scan_result, robot_fault, weather, power, comms, earth_message; the reason, fault and link enums |
| `earth.schema.json` | `EarthToSite` (goal, approve, veto, reprioritize, policy_update), `SiteToEarth` (notify, approval_request, status_report) |
| `policy.schema.json` | `Policy`: authority level 1–4 per action, plus the level-3 fallback rule |
| `messages.schema.json` | The loop envelope: `Hello` (with the `Scenario` and its `Constants`), `Welcome`, `Observation`, `Decision`, `End`, `Error` |
| `event_log.schema.json` | `LogRecord`: the JSONL run log with ground truth (`SiteTruth`) and the run metrics (`RunMetrics`), used for scoring and the replay viewer |

Worked examples of every message live in [`examples/valid/`](examples/valid/). [`examples/invalid/`](examples/invalid/) holds messages the schemas must reject. [`examples/later/`](examples/later/) holds messages as a simulator on a later minor version might send them: this version's schemas reject them, and a coordinator must still read them (see [Versioning](#versioning)).

### Conventions

- **Units:** SI throughout: metres, seconds, kilograms, watt-hours (Wh), watts (W).
- **Time:** `SimTime` is seconds since mission start. One sol is 88,775 s.
- **Grid:** integer cells. x grows east, y grows north, and the origin is the south-west corner. `GridRect` includes both corners. Cell size is given in the site map.
- **Partial knowledge:** in a `Cell`, every field is optional, and an absent field means *unknown*. The same type carries orbital prior data, noisy scan results and ground truth.
- **Capabilities:** each capability has a fixed work unit: scan m², dig m³, grade m², haul kg, compact m², sinter m², repair health-fraction, charge Wh. Relay has no rate; `range_m` is its coverage radius.
- **Authority levels:** 1 autonomous · 2 notify · 3 approve-if-reachable (ask; if no answer by the deadline, apply the fallback) · 4 hold for Earth. A command executed under an approval cites the `message_id` of its `approval_request` in `approval_id`, so the benchmark can audit autonomy.

## Versioning

The protocol follows semver. [docs/semantics.md §1.6](docs/semantics.md#16-versions) is the normative statement of these rules.

- **A patch release** changes no schema's shape (what validates and what does not): it fixes documentation, the Rust helpers or the fixtures.
- **A minor release only adds:** optional fields, enumeration values, and types of event, Earth message, command and log record. Every message valid under the previous minor stays valid, and nothing added needs an answer.
- **A major release** may change anything.
- **From 1.1, versions with the same MAJOR are compatible.** A simulator on 1.3 and a coordinator on 1.1 can run a session, and so can the reverse. Two rules make that work:
  - *The coordinator reads tolerantly.* It ignores fields, events and Earth messages it does not know, and takes an enumeration value it does not know as unknown, never as an error.
  - *The coordinator sends only what the simulator's version defines.* The simulator stays strict about what it receives, so a coordinator on a later minor holds back whatever was added after the version in `hello`.
- **1.0 works only with 1.0.** It had neither rule, and its compatibility was MAJOR.MINOR. Moving from 1.0 to 1.1 is the last time a minor release needs both sides to upgrade together.
- **The schemas are strict and belong to one version.** `additionalProperties: false` stays, so each version's schemas describe exactly what that version sends, and typos are caught. Validate a message against the schemas of the version that produced it, never an earlier one.
- The simulator and coordinator exchange versions in `hello`/`welcome`, and either side sends `error` with code `version_mismatch` if they're incompatible.
- Each schema's `$id` embeds the release tag (`.../arcadia-protocol/v1.1.0/schemas/...`), so it resolves to the raw file on GitHub. The crate's version is the protocol version.
- Every change is recorded in [CHANGELOG.md](CHANGELOG.md).

## What 1.1 added

- **`scenario.constants`:** the [simulator constants](docs/semantics.md#appendix-scenario-constants) a coordinator plans with, such as the grade window, repair and towing costs, the lost-link timeout and mission control's mean response time. A 1.0 coordinator had to assume them.
- **`RobotState.tank_wh`:** a mobile charger's tank level.
- **Pass windows on the Earth link:** the `between_passes` cause, and `window_end_t` on `comms` events.
- **Run metrics:** counts of `out_of_coverage` rejections and lost-link returns, and the time the link spent down between passes.
- **A closed leak:** a command aimed at a robot out of coverage is judged by that robot's last-contact state, not its true one.

## Candidates for 1.2

These came up while designing ArcadiaBench's 1.1 catalog, and are not in 1.1.0. Under the rules above, a minor release that adds them works with 1.1 on the other side.

- Advance notice of the Earth link's schedule in the scenario: the pass windows and the conjunction blackout, with the outage statistics, for weighing whether to ask Earth.
- Day length by season. The semantics fix daylight at half of each sol.
- Types generated from the schemas for languages other than Rust.

## Rust crate

This repo is also the `arcadia-protocol` crate, with serde types that mirror the schemas plus NDJSON framing helpers:

```toml
[dependencies]
arcadia-protocol = { git = "https://github.com/Gabe-Soler/arcadia-protocol", tag = "v1.1.0" }
# features = ["validate"]  # strict JSON Schema validation against the embedded schemas
```

```rust
use arcadia_protocol::{wire, CoordinatorToSim, Decision, SimToCoordinator};

let mut stdin = std::io::stdin().lock();
let mut stdout = std::io::stdout().lock();
while let Some(msg) = wire::recv::<_, SimToCoordinator>(&mut stdin)? {
    if let SimToCoordinator::Observation(obs) = msg {
        wire::send(&mut stdout, &CoordinatorToSim::Decision(Decision::empty(obs.seq)))?;
    }
    // ... handle hello / end / error
}
```

The Rust types are **lenient**: they don't enforce ranges, patterns or unknown fields. Use the `validate` feature (`validate::validator("messages.schema.json", "Decision")`) when you need strict checking.

That leniency is how a coordinator reads a later minor version. Unknown fields are ignored, and every enum a simulator sends (`EventKind`, `RobotStatus`, `CommsCause` and the rest) has an `Unknown` variant that takes any value this version does not define, so a `match` needs an arm for it. The enums a coordinator sends (`Action`, `Downlink`) have none. `is_compatible(a, b)` implements the compatibility rule. A minor release may add enum variants and struct fields to the crate, so depend on an exact tag.

## Development

```sh
cargo test     # meta-schema check + every fixture validates and round-trips through the Rust types
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
cargo build --no-default-features   # consumers build without `validate`
```

`tests/conformance.rs` is the drift guard. Each valid fixture must pass its schema, deserialize into the Rust type, and serialize back to identical JSON. Each invalid fixture must be rejected. Each later-minor fixture must be rejected by the schema and still read by the Rust type. When you change a schema, update the Rust type and the fixtures in the same commit.

## License

MIT
