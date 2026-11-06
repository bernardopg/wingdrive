# Wingbot Remote Execution Over WingDrive

> **Naming:** Wingbot is WingDrive's agent surface (UI, routes and windows).
> In this document "Wingbot" means the external agent runtime it connects to,
> including its HTTP API and the `@wingbot/api-client` package. Those names
> belong to that project and are kept. WingDrive's persisted settings still use
> the `wingbot` key and `wingbot_*` fields for compatibility (BRAND-002).

## Purpose

Define how a single Wingbot instance can operate across many user devices by using WingDrive as the permission, transport, and execution layer.

This design assumes the normal long-term deployment model:

- one Wingbot instance
- one paired WingDrive node owned by that Wingbot instance
- many user WingDrive devices in the same library

WingDrive becomes the system that decides what the agent can access, on which devices, and for which operations.

## Decision

Wingbot should never directly own the multi-device graph.

Instead:

- every Wingbot instance must be paired to a WingDrive node
- that paired WingDrive node is Wingbot's portal into the user's library
- all remote file access, shell access, future computer use, and other device-local execution go through WingDrive
- WingDrive is the source of truth for device identity, library membership, permissions, and remote dispatch

This means WingDrive is the permission and execution layer. Wingbot is the agent runtime and scheduler.

## Why This Shape

WingDrive already owns the hard distributed systems primitives:

- device identity
- pairing
- peer discovery
- library membership
- cross-device addressing
- file transfer
- library-scoped jobs

Wingbot already owns the hard agent-runtime primitives:

- channels, branches, workers
- memory and planning
- worker lifecycle
- tool orchestration
- model routing
- conversational UX

This split is the cleanest one:

- WingDrive decides whether something may happen and where it runs
- Wingbot decides what work should be done and how to delegate it

## Deployment Model

### Local Install

- WingDrive runs on the user's machine
- Wingbot runs as a subprocess of WingDrive
- both connect to the same local library and device identity

### Hosted Install

- hosted infrastructure runs one Wingbot instance and one WingDrive node together
- that WingDrive node appears as a device in the user's library
- user devices pair with that library
- Wingbot uses its paired WingDrive node to operate across the rest of the fleet

This avoids any central relay architecture beyond what WingDrive already needs for peer connectivity.

## Product Model

The user-facing product model is:

- WingDrive is installed on all user devices
- Wingbot is accessed through WingDrive, not through a separate desktop app
- Wingbot Desktop is replaced by WingDrive UI surfaces
- one Wingbot instance can act across the user's device fleet because WingDrive provides the device graph and permission system

This is the long-term convergence point between the two products.

## Core Principle

Wingbot should know what devices exist and what it is allowed to do.

Wingbot should not be the final authority that enforces those decisions.

WingDrive remains the enforcement layer.

That gives us:

- one security model
- one device graph
- one permission UX
- one audit surface
- one cross-device execution substrate

## Architecture

```text
User
  -> WingDrive UI
    -> Wingbot runtime
      -> worker with execution_target
        -> WingDrive client bound to paired Wingbot node
          -> permission + routing decision
            -> target WingDrive device
              -> local execution on that device
```

### Responsibility Split

#### Wingbot owns

- chat and voice interaction
- planning and delegation
- spawning workers
- worker state and status
- memory
- model routing
- deciding which target device should perform a task

#### WingDrive owns

- library authentication
- device graph
- path and subtree permissions
- capability permissions
- policy enforcement
- forwarding execution to the correct peer device
- auditing and eventing of remote operations

## Paired Node Model

Every Wingbot instance has exactly one paired WingDrive node.

That node is Wingbot's home device inside the library.

It is responsible for:

- authenticating Wingbot to the library
- maintaining the current device graph
- resolving allowed targets
- forwarding remote operations to peer devices
- storing policy and audit metadata locally as part of the library context

This is the key simplification that avoids making Wingbot maintain direct relationships with many devices.

## Execution Model

### Current Wingbot behavior

Today, Wingbot workers execute shell and file tools on the machine where Wingbot itself runs.

### Target behavior

Workers keep the same conceptual tools:

- `shell`
- `file`
- future `computer_use`

But those tools become execution-target aware.

Each worker binds to exactly one `execution_target` when it is spawned.

Examples:

- local paired Wingbot device
- user's MacBook
- Windows workstation
- NAS device
- future mobile target for mobile-safe operations

The tool interface stays simple for the model. The transport changes under the hood.

## Recommended Wingbot Integration Shape

Add an `execution_target` abstraction to workers.

Recommended shape:

- keep current worker lifecycle unchanged
- keep current tool names unchanged
- swap local shell/file implementations for WingDrive-backed proxy tools when target is remote

This means the model still thinks in terms of ordinary work:

- read files
- edit files
- run commands
- use the computer

But the actual execution happens through WingDrive according to policy.

## Recommended WingDrive Integration Shape

Add a new agent principal and remote execution protocol.

WingDrive should support:

- identifying a Wingbot instance as a library-scoped principal
- resolving what devices and subtrees it may access
- forwarding typed operations to the correct device
- enforcing policy before any peer execution occurs

This is not just raw file transfer. It is policy-aware remote operation dispatch.

## Principal Model

WingDrive needs a new principal type for agent access.

Suggested model:

```text
AgentPrincipal
  id
  library_id
  kind                 # wingbot
  paired_device_id
  display_name
  created_at
  updated_at
  status
```

This principal represents the Wingbot instance inside the library.

All remote operations performed by Wingbot should be evaluated against this principal.

## Policy Model

Permissions must be library-scoped and target-aware.

Suggested layers:

### Device Access Policy

Determines which devices the Wingbot principal may access.

Examples:

- may access MacBook and NAS
- may not access iPhone

### Location and Subtree Policy

Determines which locations or paths are accessible on those devices.

Examples:

- allow `~/Projects`
- deny `~/Documents/Finance`
- allow NAS media archive as read-only

### Operation Policy

Determines what Wingbot may do there.

Examples:

- `list`
- `read`
- `search`
- `write`
- `move`
- `delete`
- `shell`
- `computer_use`

### Confirmation Policy

Determines which actions require live user confirmation.

Examples:

- destructive actions require confirmation
- shell allowed only in trusted workspace roots
- computer use allowed only on approved desktop devices

## Effective Permission Resolution

For every request, WingDrive should resolve permissions using:

- `agent_principal`
- `library_id`
- `target_device_id`
- `target_location_or_path`
- `operation_kind`

That resolution should happen on the paired Wingbot node before forwarding, and again on the target device if needed for defense in depth.

## Request Flow

### Example: remote shell command

```text
1. User asks Wingbot to work on a repo on the MacBook
2. Wingbot spawns a worker with execution_target = MacBook
3. Worker calls shell tool
4. Proxy tool sends request to paired WingDrive node
5. Paired node resolves effective policy for principal + MacBook + path + shell
6. If allowed, paired node forwards typed execution request to MacBook WingDrive
7. MacBook WingDrive executes locally inside its OS context
8. Result returns to paired node
9. Result returns to Wingbot worker
10. Worker continues and reports status
```

### Example: remote file access

```text
1. Worker targets NAS device
2. Worker calls file read/list/edit tool
3. WingDrive permission system checks subtree and capability rules
4. NAS device executes file operation locally
5. Response returns with audit metadata
```

## Capability Surface

WingDrive should model remote execution as typed capabilities, not ad hoc commands.

Initial capability families:

- filesystem query
- filesystem mutation
- shell execution
- search and indexing queries
- future computer use
- future application integration and automation

This keeps the permission model explicit and inspectable.

## Transport Model

The transport should be a new WingDrive peer execution protocol.

It should live alongside existing pairing, library sync, file transfer, and remote job activity protocols.

Recommended shape:

- typed operation envelope
- library-scoped principal identity
- target device identity
- capability metadata
- request ID and audit metadata
- result payload and status

The target device should execute through the same internal dispatch and action/query system where practical, not through a completely separate execution stack.

## Audit Model

Every Wingbot-routed operation should be auditable.

Suggested audit fields:

```text
request_id
library_id
agent_principal_id
origin_device_id        # paired Wingbot node
target_device_id
operation_kind
target_path
policy_decision
requires_confirmation
timestamp
result_status
```

This is critical for trust, debugging, and future approval workflows.

## Device Graph Context for Wingbot

Wingbot needs a compact view of the library graph.

It should know:

- what devices exist
- whether they are online
- what capabilities they expose
- which roots are accessible
- which roots are writable or restricted
- high-level policy summaries

This context should be concise for channels and richer for workers.

Do not dump the full graph blindly into every prompt.

## Why This Replaces Wingbot Desktop

If WingDrive is the permission and execution layer, then WingDrive is also the right user interface layer.

That means:

- chat happens in WingDrive
- voice and floating panels happen in WingDrive
- permission granting happens in WingDrive
- device selection happens in WingDrive
- file and context views happen in WingDrive

Wingbot Desktop becomes redundant once this model exists.

## Local vs Hosted Symmetry

This design intentionally keeps local and hosted deployments symmetric.

### Local

- WingDrive launches Wingbot locally
- both share one local paired device

### Hosted

- hosted environment runs one Wingbot and one WingDrive node together
- that node is visible as a device in the user's library
- user pairs the rest of their devices into the same library

Same architecture, different packaging.

## What Already Exists

WingDrive already has:

- device identity and pairing
- library-scoped device membership
- peer discovery
- cross-device addressing
- file transfer
- typed action/query dispatch
- library-scoped jobs and remote job visibility

Wingbot already has:

- worker lifecycle and isolated tool servers
- local shell and file tools
- alternate worker backend precedent
- strong API and runtime model

The integration does not require rewriting either product from scratch. It requires adding the right boundary.

## What Is Missing

### In WingDrive

- agent principal identity
- per-device policy model
- per-subtree capability policy
- effective permission resolution
- remote operation forwarding
- audit-first remote execution records

### In Wingbot

- worker execution target abstraction
- WingDrive-backed proxy tools
- device/capability awareness in worker scheduling
- UI that surfaces device targeting and approvals through WingDrive

## Recommended Phases

### Phase 1: Principal and Policy Model

- add Wingbot principal type to WingDrive
- add device, subtree, and capability policies
- add effective-permission resolution

### Phase 2: Remote File Queries

- allow Wingbot-routed list/read/search against target devices through WingDrive
- wire policy enforcement and audit logging

### Phase 3: Remote Shell Execution

- add shell capability and remote execution protocol
- bind worker shell tool to WingDrive proxy

### Phase 4: WingDrive UI as the Agent Surface

- embed chat, voice, target-device status, and permission controls in WingDrive
- remove dependency on Wingbot Desktop

### Phase 5: Computer Use

- add computer-use capability for approved devices
- route through same principal and policy model

### Phase 6: Hosted Productization

- ship hosted Wingbot with embedded WingDrive node
- build onboarding around library pairing

## Open Questions

1. Should the target device perform a second policy check locally, or should the paired Wingbot node be the only enforcement point?
2. How should offline devices be represented in worker scheduling and retries?
3. Which operations should always require live approval even when globally allowed?
4. Should execution target be chosen explicitly by the model, by policy, or by a scheduler that picks the best allowed device?
5. How should mobile devices participate, read-only, limited actions, or full capability subsets?

## Recommendation

Adopt the paired-node architecture.

The correct long-term model is:

- one Wingbot instance
- one paired WingDrive node for that Wingbot
- many user WingDrive devices in the same library
- WingDrive as the permission and execution layer
- Wingbot as the agent runtime and scheduler

This gives us a clean answer to a core product question:

- how does one agent operate naturally across a user's whole device fleet without centralizing trust?

By operating through WingDrive.
