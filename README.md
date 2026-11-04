# arcadia-protocol

The versioned interface spec between a **coordinator** (such as [ArcadiaOS](https://github.com/Gabe-Soler/arcadiaos)) and the **ArcadiaBench** simulator ([ArcadiaBench](https://github.com/Gabe-Soler/ArcadiaBench)). The two projects never import each other's code. They only share this spec, so any team can plug a coordinator written in any language into ArcadiaBench.

**Protocol version: 0.2.0**

The schemas define the messages; **[docs/semantics.md](docs/semantics.md)** defines what they mean. It is the normative description of the behaviour behind the interface: decision points, time, every command's preconditions, effects, events and failure reasons, faults and repair, energy, comms and coverage, authority enforcement, and the run metrics.

## Who talks to whom

ArcadiaOS runs **on Mars**, at the construction site. Commands to robots are local and instant. The 4–24 minute light-time delay, the multi-week conjunction blackouts and the random outages all sit on the **Earth link**, between mission control and the coordinator. ArcadiaBench simulates both the site and a scripted mission control, and routes Earth messages through its delay model.

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
sim → coord   hello        protocol version, run id, seed, scenario (prior map, robots, stations, depots, power, base station, policy)
coord → sim   welcome      protocol version, coordinator name/version
              ┌─ repeat ───────────────────────────────────────────────────────────────┐
sim → coord   │ observation  seq=n, t, events since last observation, robot snapshot   │
coord → sim   │ decision     seq=n, robot commands, messages for Earth, optional wake_at│
              └────────────────────────────────────────────────────────────────────────┘
sim → coord   end          reason (goals_complete, time_limit, ...)
```

- **Deterministic:** sim time does not advance while the simulator waits for a decision, so a run is fully reproducible from its seed.
- **Decision points:** the simulator sends an observation when a command completes, fails or is rejected; a fault happens; weather, power or comms change; an Earth message arrives; or the coordinator's `wake_at` time is reached. [docs/semantics.md](docs/semantics.md#13-decision-points) has the full list.
- **Goals arrive over the Earth link.** They're not in the scenario. The first goal is delivered as an `earth_message` event at t = 0.
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
| `messages.schema.json` | The loop envelope: `Hello`, `Welcome`, `Observation`, `Decision`, `End`, `Error` |
| `event_log.schema.json` | `LogRecord`: the JSONL run log with ground truth (`SiteTruth`) and the run metrics (`RunMetrics`), used for scoring and the replay viewer |

Worked examples of every message live in [`examples/valid/`](examples/valid/). [`examples/invalid/`](examples/invalid/) holds messages the schemas must reject.

### Conventions

- **Units:** SI throughout: metres, seconds, kilograms, watt-hours (Wh), watts (W).
- **Time:** `SimTime` is seconds since mission start. One sol is 88,775 s.
- **Grid:** integer cells. x grows east, y grows north, and the origin is the south-west corner. `GridRect` includes both corners. Cell size is given in the site map.
- **Partial knowledge:** in a `Cell`, every field is optional, and an absent field means *unknown*. The same type carries orbital prior data, noisy scan results and ground truth.
- **Capabilities:** each capability has a fixed work unit: scan m², dig m³, grade m², haul kg, compact m², sinter m², repair health-fraction, charge Wh. Relay has no rate; `range_m` is its coverage radius.
- **Authority levels:** 1 autonomous · 2 notify · 3 approve-if-reachable (ask; if no answer by `decide_by_t`, apply the fallback) · 4 hold for Earth. A command executed under an approval cites it in `approval_id`, so the benchmark can audit autonomy.

## Versioning

- The spec follows semver. Each schema's `$id` embeds the release tag (`.../arcadia-protocol/v0.2.0/schemas/...`), so it resolves to the raw file on GitHub.
- Two versions are **compatible when MAJOR.MINOR match**. During 0.x, any minor bump may break compatibility. The simulator and coordinator exchange versions in `hello`/`welcome`, and either side sends `error` with code `version_mismatch` if they're incompatible.
- Every change is recorded in [CHANGELOG.md](CHANGELOG.md).

## Rust crate

This repo is also the `arcadia-protocol` crate, with serde types that mirror the schemas plus NDJSON framing helpers:

```toml
[dependencies]
arcadia-protocol = { git = "https://github.com/Gabe-Soler/arcadia-protocol", tag = "v0.2.0" }
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

## Development

```sh
cargo test     # meta-schema check + every fixture validates and round-trips through the Rust types
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

`tests/conformance.rs` is the drift guard. Each valid fixture must pass its schema, deserialize into the Rust type, and serialize back to identical JSON. Each invalid fixture must be rejected. When you change a schema, update the Rust type and the fixtures in the same commit.

## License

MIT
