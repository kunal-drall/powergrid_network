## Governance setup (quick)

1) Deploy contracts (token, registry, grid, governance).
2) Point Registry/Grid to Governance (one-time, owner only):
	- `ResourceRegistry.set_governance_address(governance)`
	- `GridService.set_governance_address(governance)`
3) Enable GridService to mint rewards via Governance proposal:
	- Create proposal: `SetTokenMinter(grid_service, true)` and execute after it passes.
4) Authorize your oracle/ops account to control Grid events:
	- Create proposal: `SetGridAuthorizedCaller(oracle, true)` and execute.
5) Optional parameters via proposals:
	- `UpdateCompensationRate(new_rate)` for `GridService`
	- `UpdateMinStake(min_stake)` and `UpdateReputationThreshold(threshold)` for `ResourceRegistry`

Tip: Quorum and duration are enforced by the Governance contract; ensure proposals reach quorum before execution.

## Oracle: triggering events with GridSignal

Authorized callers (governance, owner, or added callers) can push real-time signals to `GridService.ingest_grid_signal`.

Signal shape (Rust):

```
GridSignal {
  event_type: GridEventType::DemandResponse,
  duration_minutes: 30,
  target_reduction_kw: 100,
  severity: 3,              // 1..=5 scales compensation
  start: true,              // create/start an event
  complete_event_id: None,  // or Some(id) to complete
}
```

Effects:
- If `start` is true, a new event is created with compensation = `default_compensation_rate * severity`.
- If `complete_event_id` is Some(id), the contract attempts to complete that event.
- Rewards on verification are reputation-weighted (0.8x–1.2x) based on `ResourceRegistry.get_device_reputation`.

## Script quick check (WSL)

Run a bash syntax check for all `.sh` files under `scripts/` from PowerShell:

```powershell
wsl -e bash -lc 'cd ~/powergrid_network/scripts; shopt -s nullglob; for f in *.sh; do printf "Checking %s\\n" "$f"; bash -n "$f" || { echo "Syntax error in $f"; exit 1; }; done; echo OK'
```

Notes:
- Keep the entire command on one line.
- Using single quotes around the bash payload prevents PowerShell from expanding `$f`.

## Governance setup (quick start)

Use these steps after deployment so governance can manage parameters and roles across contracts:

1) Point contracts to Governance
- ResourceRegistry: call `set_governance_address(governance_addr)`
- GridService: call `set_governance_address(governance_addr)`

2) Authorize cross-contract operations via proposals
- Grant GridService minter on Token: `SetTokenMinter(GridService, true)`
- Allow GridService to update device performance in Registry: `SetRegistryAuthorizedCaller(GridService, true)`
- Add your oracle/aggregator to GridService callers: `SetGridAuthorizedCaller(Oracle, true)`
- Set default compensation rate: `UpdateCompensationRate(new_rate)`
- Adjust min stake / reputation threshold: `UpdateMinStake(v)` / `UpdateReputationThreshold(v)`

3) Execute proposals after voting period
- Governance enforces quorum and delay; once passed and timelock elapsed, call `execute_proposal(id)`.

Notes
- Treasury spend uses PSP22 transfer; ensure Governance holds tokens or has allowance.
- For local dev, you can grant minter/admin roles to speed-up iterating; use proposals in production.

## Oracle: triggering grid events via ingest_grid_signal

GridService exposes an authorized-only `ingest_grid_signal(GridSignal)` to create/complete events based on external conditions. Example payload:

```rust
use powergrid_shared::{GridSignal, GridEventType};

let signal = GridSignal {
	event_type: GridEventType::PeakShaving,
	duration_minutes: 30,
	target_reduction_kw: 150,
	severity: 3,              // 1..=5, scales compensation 1x..=5x of default
	start: true,              // create a new event now
	complete_event_id: None,  // or Some(id) to complete
};

// As an authorized caller (oracle/aggregator), call:
// grid_service.ingest_grid_signal(signal)
```

Behavior
- If `start == true`, a new event is created using default compensation rate multiplied by `severity`.
- If `complete_event_id` is set, the contract attempts to complete that event.
- Reputation-weighted rewards (0.8x–1.2x) are applied when verifying participation.

## Governance setup and oracle signal ingestion

1) Deploy contracts (token, registry, grid_service, governance)

2) Wire addresses
- Call `ResourceRegistry::set_governance_address(governance)` and `GridService::set_governance_address(governance)` as owner.

3) Roles via proposals
- Propose `SetTokenMinter(grid_service, true)` so GridService can mint rewards.
- Propose `SetGridAuthorizedCaller(oracle, true)` to allow oracle to ingest signals.
- Propose `SetRegistryAuthorizedCaller(grid_service, true)` so GridService can update device performance.
- Optionally set default compensation: `UpdateCompensationRate(new_rate)`.

4) Queue and execute
- After voting ends, call `queue_proposal(id)` and wait `timelock_seconds` before `execute_proposal(id)`.

5) Oracle example: ingest a grid signal

```
// Pseudocode – off-chain caller with oracle account authorized in GridService
GridSignal {
	event_type: DemandResponse,
	duration_minutes: 60,
	target_reduction_kw: 100,
	severity: 3,            // 3x default compensation
	start: true,            // create event
	complete_event_id: None // don't complete yet
}

// Later, to close:
GridSignal {
	event_type: DemandResponse,
	duration_minutes: 0,
	target_reduction_kw: 0,
	severity: 1,
	start: false,
	complete_event_id: Some(event_id)
}
```

6) Emergency controls
- Pause token transfers: `PowergridToken::set_paused(true)` (admin only).
# PowerGrid Network Docs

## Governance setup and oracle-driven grid events

This project supports on-chain governance for parameter updates and role management, plus an oracle-style entrypoint for triggering grid events.

### 1) One-time wiring after deployment

After deploying Token (PSP22), ResourceRegistry, GridService, and Governance contracts:

1. Point GridService/ResourceRegistry at the Governance contract so it can manage roles/params:
	 - GridService.set_governance_address(governance_account)
	 - ResourceRegistry.set_governance_address(governance_account)

2. Use Governance proposals to grant roles and set parameters (see `shared/src/types.rs::ProposalType`):
	 - SetTokenMinter(GridService, true) to allow GridService to mint rewards
	 - SetGridAuthorizedCaller(oracle, true) to allow your oracle/aggregator to call `ingest_grid_signal`
	 - SetRegistryAuthorizedCaller(GridService, true) so GridService can update device performance
	 - UpdateCompensationRate(new_rate) to set GridService base compensation
	 - UpdateMinStake(amount) to update registry’s minimum stake
	 - UpdateReputationThreshold(threshold) to tune reputation acceptance policy

Execute each proposal after it passes to apply changes on-chain.

### 2) Oracle calling `ingest_grid_signal`

GridService exposes `ingest_grid_signal(signal: GridSignal)` for authorized callers. The `GridSignal` payload:

```
{
	event_type: DemandResponse | FrequencyRegulation | PeakShaving | LoadBalancing | Emergency,
	duration_minutes: u64,
	target_reduction_kw: u64,
	severity: u8,                  // 1..=5; scales compensation = severity * default_rate
	start: bool,                   // true to create/start an event
	complete_event_id: Option<u64> // optionally complete a previous event
}
```

Example (TypeScript with @polkadot/api; pseudocode):

```ts
// assumes api + signer configured and contract addresses known
const signal = {
	event_type: { DemandResponse: null },
	duration_minutes: 30,
	target_reduction_kw: 50,
	severity: 3,
	start: true,
	complete_event_id: null,
};

// call grid_service.ingest_grid_signal(signal)
const { gasRequired, result, output } = await api.call.contractsApi.call(
	gridServiceAddr,
	signer.address,
	0,           // value
	null,        // gasLimit (let node estimate)
	null,        // storageDepositLimit
	gridService.abi.findMessage('ingest_grid_signal').toU8a([signal])
);
```

Notes:
- Severity 1–5 multiplies the default compensation rate (set via governance). If `start` is true, a new event is created and its id is returned.
- If `complete_event_id` is set, the contract will attempt to complete that event.
- Reward distribution on verification mints PSP22 tokens via the Token contract; ensure GridService has the MINTER role.

