# 09 — Forgeyard Transport, QUIC & Internal Protocol System Architecture

**Document type:** Core Communication & Protocol System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** Daemon↔agent transport, QUIC connection/session lifecycle, Postcard envelopes, protocol negotiation, multiplexed streams, authentication, mTLS, message identity, idempotency, backpressure, reconnect/resume, lease delivery, control messages, logs, CAS transfer boundaries, compatibility, rolling upgrades, and transport observability  
**Architecture style:** QUIC-native internal transport with versioned Postcard control protocol, typed envelopes, stream separation, at-least-once-safe commands, explicit compatibility negotiation, and public/private protocol separation  
**Status:** Target production architecture for the new Forgeyard repository  
**Relationship to prior work:** Builds directly on `07-forgeyard-runner-agent-system-architecture.md` and `08-forgeyard-sandbox-executor-system-architecture.md`, and integrates with the Run/Job state machine, scheduler, CAS transfer layer, identity/trust, event/log system, and daemon application composition.

---

# 1. Purpose

Forgeyard needs a reliable internal communication layer between:

```text
forgeyard-daemon
forgeyard-agent
forgeyard-signing-worker
forgeyard-device-agent
future worker types
```

The internal protocol must support:

```text
runner registration
heartbeats
lease delivery
attempt state changes
cancellation
log streaming
CAS transfer coordination
capability refresh
reconnect/resume
health/control
```

The central rule is:

> **Forgeyard's native machine-to-machine protocol is QUIC + versioned Postcard messages, with separate streams for logically independent traffic classes.**

A second rule is:

> **The protocol is at-least-once safe: messages may be duplicated, reordered across independent streams, delayed, or retried, and correctness must still hold.**

A third rule is:

> **Public REST/JSON/WebSocket APIs are not the agent protocol. Internal and public protocols evolve independently.**

---

# 2. Architectural Position

```text
                 Forgeyard Daemon
                       │
                QUIC + TLS/mTLS
                       │
                       ▼
                Connection Session
                       │
      ┌────────────────┼────────────────┐
      ▼                ▼                ▼
   Control           Logs          CAS/Data
      │                │                │
      └────────────────┼────────────────┘
                       ▼
                  Agent Runtime
```

Public side:

```text
Users/UI/CLI
     │
 REST/JSON / WS/SSE
     │
 Forgeyard Daemon
     │
 QUIC/Postcard
     │
 Agents
```

---

# 3. Goals

The transport/protocol subsystem MUST:

1. use QUIC for native daemon↔agent transport;
2. use TLS encryption;
3. support mTLS for strong agent identity;
4. use Postcard for compact internal control messages;
5. version every protocol family;
6. negotiate compatible versions;
7. support rolling N/N-1 compatibility where practical;
8. support typed message envelopes;
9. support MessageId;
10. support CorrelationId;
11. support stream multiplexing;
12. separate control traffic from logs;
13. separate CAS/data streams from control;
14. apply backpressure;
15. support reconnect;
16. support resume/resync;
17. support duplicate delivery;
18. support idempotent commands;
19. support heartbeats;
20. support cancellation priority;
21. support bounded message size;
22. support authentication/authorization;
23. support connection fencing;
24. expose metrics/traces;
25. support protocol conformance tests;
26. avoid coupling domain models directly to wire structs;
27. avoid JSON internally except where interoperability requires it;
28. work through proxies/NAT where possible;
29. support multiple daemon endpoints;
30. remain extensible to future worker roles.

---

# 4. Non-Goals

The internal transport does not:

```text
define business authorization
schedule jobs
decide job state
store blobs itself
replace CAS
replace metadata storage
```

It moves authenticated typed messages.

---

# 5. Workspace Structure

```text
crates/transport/
├── forgeyard-transport/
├── forgeyard-transport-model/
├── forgeyard-transport-quic/
├── forgeyard-transport-session/
├── forgeyard-transport-stream/
├── forgeyard-transport-auth/
├── forgeyard-transport-backpressure/
├── forgeyard-transport-reconnect/
├── forgeyard-transport-health/
├── forgeyard-transport-metrics/
└── forgeyard-transport-testkit/
```

Protocol crates:

```text
crates/protocol/
├── forgeyard-version/
├── forgeyard-envelope/
├── forgeyard-wire/
├── forgeyard-wire-agent/
├── forgeyard-wire-cas/
├── forgeyard-wire-log/
├── forgeyard-wire-health/
├── forgeyard-wire-admin/
├── forgeyard-wire-signing/
├── forgeyard-wire-device/
└── forgeyard-protocol-testkit/
```

---

# 6. Transport Trait

Higher-level components should not depend on Quinn/QUIC internals directly.

```rust
#[async_trait]
pub trait TransportConnection: Send + Sync {
    async fn open_bi(&self, kind: StreamKind)
        -> Result<Box<dyn BiStream>, TransportError>;

    async fn open_uni(&self, kind: StreamKind)
        -> Result<Box<dyn SendStream>, TransportError>;

    async fn close(&self, reason: CloseReason)
        -> Result<(), TransportError>;
}
```

---

# 7. QUIC Implementation

`forgeyard-transport-quic` implements the trait using a Rust QUIC stack.

Exact crate is an implementation detail.

The architecture must not leak:

```text
quinn::Connection
```

through all domain/service crates.

---

# 8. Why QUIC

Desired properties:

```text
TLS built in
multiplexed streams
independent stream flow control
no TCP head-of-line blocking across streams
connection migration potential
efficient binary transport
bidirectional/unidirectional streams
```

---

# 9. One Logical Agent Connection

Each online agent normally maintains:

```text
one active QUIC connection
```

to one daemon endpoint.

Within it:

```text
many streams
```

---

# 10. Connection Identity

Connection is bound to:

```text
RunnerId
AgentSessionId
authenticated certificate/principal
protocol version
```

---

# 11. Connection Session

```rust
pub struct ConnectionSession {
    pub runner: RunnerId,
    pub agent_session: AgentSessionId,
    pub transport_session: TransportSessionId,
    pub protocol: ProtocolSelection,
    pub connected_at: Timestamp,
}
```

---

# 12. TransportSessionId

```rust
pub struct TransportSessionId(Ulid);
```

Changes on every network connection.

Do not confuse with AgentSessionId.

---

# 13. Why Three Identities

```text
RunnerId
  stable logical runner

AgentSessionId
  one agent process/runtime session

TransportSessionId
  one network connection
```

This supports reconnect of same agent process.

---

# 14. Reconnect Example

```text
RunnerId = R1
AgentSessionId = A1
TransportSessionId = T1
connection lost
TransportSessionId = T2
AgentSessionId remains A1
```

---

# 15. Restart Example

```text
RunnerId = R1
AgentSessionId A1 ends
Agent restarts
AgentSessionId = A2
TransportSessionId = T3
```

Old leases bound to A1 are not silently inherited.

---

# 16. Authentication

Distributed production should use:

```text
TLS
+
client authentication
```

preferably:

```text
mTLS
```

for agents.

---

# 17. Certificate Binding

Agent certificate maps to:

```text
RunnerId / workload identity
```

through trust subsystem.

---

# 18. Registration Still Required

Successful TLS authentication does not mean:

```text
runner is ready
```

Agent must register:

```text
session
version
capabilities
```

---

# 19. TLS Server Authentication

Agent validates daemon certificate/trust root.

No insecure skip-verify production mode.

---

# 20. Bootstrap Enrollment

Bootstrap may use one-time token over TLS.

After enrollment:

```text
mTLS certificate
```

or equivalent long-term identity.

---

# 21. Credential Rotation

Transport supports certificate refresh/reconnect without losing RunnerId.

---

# 22. Certificate Expiry

Agent warns before expiry.

Daemon can reject expired credentials.

---

# 23. Protocol Version

```rust
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}
```

---

# 24. Supported Range

```rust
pub struct ProtocolRange {
    pub min: ProtocolVersion,
    pub max: ProtocolVersion,
}
```

---

# 25. Negotiation

Handshake:

```text
agent support
daemon support
  ↓
highest mutually supported compatible version
```

---

# 26. Major Version

Breaking semantics.

No compatibility assumed.

---

# 27. Minor Version

Backward-compatible extension within defined policy.

---

# 28. N/N-1 Rolling Policy

Recommended production target:

```text
daemon N supports agent N and N-1
agent N supports daemon N and possibly N-1
```

exact matrix documented per release.

---

# 29. Protocol Families

Do not use one global schema for everything.

Families:

```text
agent control
CAS transfer
logs
health
signing
device
```

Each may have own sub-version where useful.

---

# 30. Envelope

```rust
pub struct Envelope<T> {
    pub protocol: ProtocolVersion,
    pub message_id: MessageId,
    pub correlation_id: Option<CorrelationId>,
    pub sent_at: Timestamp,
    pub payload: T,
}
```

---

# 31. MessageId

Every retryable/meaningful control message has stable:

```text
MessageId
```

---

# 32. CorrelationId

Links:

```text
lease offer
accept
phase events
completion
```

or request/response operations.

---

# 33. RequestId

Could reuse CorrelationId or define:

```rust
pub struct RequestId(Ulid);
```

Avoid redundant IDs unless semantics differ.

---

# 34. Wire Types

Domain types should not be serialized directly.

Wire crate defines:

```rust
pub struct WireLeaseOfferV1 { ... }
```

Conversions:

```text
domain -> wire
wire -> validated domain command
```

---

# 35. Why Separate Wire Types

Allows:

```text
domain refactor
without protocol break
```

---

# 36. Postcard

Use for:

```text
control messages
small structured events
capability reports
```

---

# 37. Large Bytes

Do not wrap huge artifact/log payloads in one Postcard object.

Use:

```text
small Postcard header
+
streamed bytes
```

---

# 38. Message Framing

For message streams:

```text
length prefix
+
Postcard envelope
```

or stream-per-message pattern.

---

# 39. Recommended Control Channel

Long-lived bidirectional control stream with framed small messages.

---

# 40. Stream Kind

```rust
pub enum StreamKind {
    Control,
    Heartbeat,
    Log,
    CasUpload,
    CasDownload,
    Diagnostic,
    Device,
    Signing,
}
```

QUIC stream application header identifies kind.

---

# 41. Control Stream

Carries:

```text
registration
lease offer
accept/reject
cancel
phase transitions
completion
capability update
resync
```

---

# 42. Heartbeat

Can share control stream or dedicated lightweight stream.

Recommended initially:

```text
control stream
```

with high priority semantics.

---

# 43. Log Streams

Per attempt or multiplexed stream family.

Recommended:

```text
one unidirectional log stream per attempt
```

or bounded groups.

---

# 44. CAS Streams

Large objects use independent streams.

One object/session per stream or chunk set.

---

# 45. Why Separation

Slow artifact upload must not block:

```text
cancellation
lease renewal
heartbeat
```

---

# 46. Stream Priority

QUIC implementations may support stream priorities differently.

Application must also prioritize queues:

```text
cancel
lease renew
control
logs
CAS background
```

---

# 47. Control Priority

Highest.

---

# 48. Log Priority

Medium.

---

# 49. Background Replication

Lower than active job-critical traffic.

---

# 50. Message Size Limits

Every structured message type has maximum encoded size.

Examples:

```text
control frame < configured limit
capability report bounded
heartbeat bounded
```

---

# 51. Capability Report Size

Avoid embedding:

```text
millions of CAS IDs
```

Use summaries.

---

# 52. Postcard Decode Limits

Never decode unbounded attacker-controlled length into memory.

Use framed bounded buffers.

---

# 53. Unknown Message Variant

Minor-compatible protocol may safely ignore known-extensible optional message types only if semantics allow.

Otherwise:

```text
protocol error
```

---

# 54. Enum Compatibility

Do not depend on serde enum layout changing implicitly.

Wire enums version explicitly.

---

# 55. Wire Schema Stability

Once released:

```text
field semantics stable
```

Add new optional fields/versioned variants carefully.

---

# 56. Canonical vs Wire Encoding

Wire encoding need not be content-address canonical encoding.

Do not reuse blindly for digest identity.

---

# 57. Registration Protocol

Sequence:

```text
TLS connect
  ↓
TransportHello
  ↓
version negotiation
  ↓
RegisterAgent
  ↓
RegistrationAccepted
  ↓
control session active
```

---

# 58. TransportHello

```rust
pub struct TransportHello {
    pub role: PeerRole,
    pub versions: ProtocolRange,
    pub agent_session: Option<AgentSessionId>,
}
```

---

# 59. Peer Roles

```rust
pub enum PeerRole {
    Agent,
    DeviceAgent,
    SigningWorker,
    InternalWorker,
}
```

---

# 60. Registration Timeout

If peer authenticates but does not register quickly:

```text
close connection
```

---

# 61. Duplicate Session Connection

Same RunnerId + AgentSessionId may reconnect.

Daemon must fence old TransportSessionId.

---

# 62. Session Fencing

When T2 becomes active:

```text
T1 control authority revoked
```

---

# 63. Late T1 Message

Rejected based on transport/session context where relevant.

---

# 64. Lease Message

```rust
pub struct WireLeaseOfferV1 {
    pub lease: LeaseId,
    pub attempt: JobAttemptId,
    pub job: JobId,
    pub spec_id: JobSpecId,
    pub expires_at: Timestamp,
    pub payload_ref: CasObjectRef,
}
```

Large JobSpec may be referenced through CAS rather than embedded.

---

# 65. JobSpec Transport

Options:

```text
small spec inline
large spec CAS-ref
```

Recommended:

```text
wire carries JobSpecId + CAS/reference/compact spec
```

---

# 66. JobSpec Verification

Agent verifies:

```text
digest == JobSpecId
```

---

# 67. Lease Delivery

At-least-once.

Duplicate offers safe.

---

# 68. Lease Accept

```rust
pub struct WireLeaseAcceptedV1 {
    pub lease: LeaseId,
    pub attempt: JobAttemptId,
    pub session: AgentSessionId,
}
```

---

# 69. Lease Reject

Typed reason.

---

# 70. Lease Renewal

Heartbeat or explicit:

```rust
pub struct WireLeaseRenewV1 {
    pub lease: LeaseId,
    pub attempt: JobAttemptId,
    pub observed_phase: AttemptPhase,
}
```

Daemon extends if still authoritative.

---

# 71. Lease Renewal Ack

Returns new authoritative expiry.

---

# 72. Cancellation

```rust
pub struct WireCancelAttemptV1 {
    pub attempt: JobAttemptId,
    pub lease: LeaseId,
    pub reason: CancellationReason,
}
```

---

# 73. Cancel Idempotency

Repeated cancel safe.

---

# 74. Phase Change

```text
Preparing
Running
UploadingOutputs
```

small control messages.

---

# 75. Completion

```rust
pub struct WireAttemptCompletionV1 {
    pub job: JobId,
    pub attempt: JobAttemptId,
    pub lease: LeaseId,
    pub spec: JobSpecId,
    pub result: WireExecutionResultV1,
    pub outputs: Vec<CasObjectRef>,
}
```

---

# 76. Completion Ack

```text
Accepted
DuplicateAccepted
StaleRejected
InvalidRejected
```

---

# 77. Unknown Completion Outcome

If connection breaks after send:

retry same MessageId after reconnect.

---

# 78. Dedup Storage

Control plane records processed message IDs for operations requiring exact idempotency.

---

# 79. Dedup Retention

Bounded by operation TTL/history.

Do not store every heartbeat forever.

---

# 80. Heartbeat Wire

```rust
pub struct WireHeartbeatV1 {
    pub runner: RunnerId,
    pub session: AgentSessionId,
    pub active_attempts: Vec<WireActiveAttemptV1>,
    pub capability_digest: CapabilityDigest,
    pub resources: WireResourceSummaryV1,
}
```

---

# 81. Heartbeat Sequence

Optional:

```rust
pub struct HeartbeatSeq(u64);
```

helps detect out-of-order duplicates.

---

# 82. Heartbeat Loss

Expected.

Liveness uses timeout, not guaranteed delivery.

---

# 83. Log Protocol

Header:

```rust
pub struct LogStreamHeaderV1 {
    pub attempt: JobAttemptId,
    pub lease: LeaseId,
    pub first_seq: LogSeq,
    pub encoding: LogEncoding,
}
```

Then frames.

---

# 84. Log Frame

```rust
pub struct WireLogFrameV1 {
    pub seq: LogSeq,
    pub stream: LogStreamKind,
    pub timestamp: Timestamp,
    pub bytes: BoundedBytes,
}
```

---

# 85. Log Ordering

Sequence within attempt stream.

---

# 86. Log Backfill

Reconnect request:

```rust
pub struct LogResumeRequest {
    pub attempt: JobAttemptId,
    pub from_seq: LogSeq,
}
```

---

# 87. CAS Transfer

Control protocol negotiates object transfer.

Large bytes on dedicated streams.

---

# 88. CAS NeedObjects

```rust
pub struct NeedObjectsV1 {
    pub objects: Vec<CasObjectRef>,
}
```

Bound batch size.

---

# 89. CAS MissingObjects

Returns subset.

---

# 90. CAS Download Header

```rust
pub struct CasObjectStreamHeaderV1 {
    pub object: CasObjectRef,
    pub offset: u64,
    pub length: ByteSize,
}
```

---

# 91. CAS Verification

Receiver computes digest.

Transport checksum alone is insufficient.

---

# 92. CAS Resume

Request byte/chunk ranges only after verified local partial state.

---

# 93. Control vs CAS Authorization

A valid agent connection does not automatically allow arbitrary CAS enumeration.

Object access is scoped to leased/authorized work.

---

# 94. Capability Update

```rust
pub struct CapabilityUpdateV1 {
    pub digest: CapabilityDigest,
    pub full: RunnerCapabilitiesWireV1,
}
```

---

# 95. Capability Delta

Future optimization.

Initial implementation:

```text
full snapshot on change
```

---

# 96. Health Protocol

Agent can report health summary.

Daemon can request doctor probe selectively.

---

# 97. Admin Commands

Do not expose broad remote shell.

Allowed typed commands:

```text
drain
resume
refresh capabilities
run health probe
```

---

# 98. No Remote Shell

Critical invariant.

Forgeyard daemon cannot send:

```text
"run arbitrary command on runner"
```

outside a leased JobSpec.

---

# 99. Signing Protocol

Separate from general agent execution protocol.

Signing worker accepts:

```text
typed sign requests
digest/material refs
policy proof
```

not arbitrary shell.

---

# 100. Device Protocol

Separate typed device operations:

```text
discover
reserve
install
run test
reset
```

---

# 101. Connection Lifecycle

States:

```rust
pub enum ConnectionState {
    Connecting,
    Authenticating,
    Negotiating,
    Registering,
    Active,
    Draining,
    Closing,
    Closed,
}
```

---

# 102. Close Reason

```rust
pub enum CloseReason {
    Normal,
    Shutdown,
    ProtocolError,
    AuthenticationFailure,
    VersionMismatch,
    SessionFenced,
    RunnerDisabled,
    InternalError,
}
```

---

# 103. QUIC Application Error Codes

Map stable Forgeyard protocol close codes.

---

# 104. Reconnect

Agent retries endpoint(s) with exponential backoff + jitter.

---

# 105. Endpoint List

```rust
pub struct ControlPlaneEndpoint {
    pub address: EndpointAddress,
    pub server_name: ServerName,
}
```

---

# 106. Endpoint Failover

Try configured/discovered endpoints.

---

# 107. DNS Discovery

Can use DNS/service discovery later.

Not required for core.

---

# 108. Connection Migration

QUIC may support path migration.

Treat as transport optimization.

Session identity remains same.

---

# 109. NAT

Agents initiate outbound connections, simplifying firewall/NAT.

---

# 110. Incoming Agent Port

Daemon listens on configured QUIC port.

---

# 111. Firewall

Production requires explicit UDP allowance for QUIC.

---

# 112. TCP Fallback

Should Forgeyard support fallback?

Recommended:

```text
native QUIC required for distributed agent protocol
```

Add TCP fallback only if operational evidence demands it.

Avoid duplicating protocol stacks initially.

---

# 113. Proxy Environments

QUIC-blocked enterprise networks may require relay/tunnel later.

Design transport trait allows alternate backend.

---

# 114. Transport Backend Future

Potential:

```text
QUIC
WebTransport
TCP/TLS fallback
relay tunnel
```

but one semantics layer.

---

# 115. Reconnect Resync

After registration on new transport:

```text
agent sends active attempts
last significant event seq
last log seq
capability digest
```

---

# 116. Resync Request

```rust
pub struct AgentResyncV1 {
    pub active_attempts: Vec<ActiveAttemptResyncV1>,
    pub capability_digest: CapabilityDigest,
}
```

---

# 117. Daemon Resync Response

Per attempt:

```text
Continue
Cancel
Stale
```

---

# 118. Continue

Includes current lease expiry.

---

# 119. Cancel

Includes reason.

---

# 120. Stale

Agent terminates and cleans up.

---

# 121. Resync Idempotency

Repeated reconnect/resync safe.

---

# 122. In-Flight Control Messages

Agent retries only operations whose outcome is unknown and idempotent.

---

# 123. Request/Response Pattern

Use CorrelationId.

---

# 124. Timeouts

Every request has bounded wait.

---

# 125. No Infinite Await

Transport request APIs enforce deadline.

---

# 126. Backpressure

Three levels:

```text
per stream
per connection
per application queue
```

---

# 127. Bounded Queues

All outbound/inbound internal channels bounded.

---

# 128. Control Queue

Small but high priority.

---

# 129. Log Queue

Larger/spoolable.

---

# 130. CAS Queue

Bandwidth-heavy and concurrency-limited.

---

# 131. Backpressure Strategy

If log consumer slow:

```text
spool
```

If CAS backend slow:

```text
pause stream/read
```

If control queue full:

```text
treat as severe transport health issue
```

---

# 132. Cancellation Priority

Never wait behind multi-GB upload.

---

# 133. Stream Concurrency Limits

Bound open streams per connection.

---

# 134. CAS Parallelism

Configurable based on bandwidth/resources.

---

# 135. Log Stream Count

Potential one per active attempt.

Bound total attempts by runner resource policy.

---

# 136. Flow Control

Tune QUIC stream/connection windows.

Operational optimization.

---

# 137. Memory Safety

Do not allocate entire peer-advertised frame length without max bound.

---

# 138. Malformed Message

Close stream or connection depending severity.

---

# 139. Protocol Violation Severity

```rust
pub enum ProtocolViolationSeverity {
    Stream,
    Session,
    Security,
}
```

---

# 140. Stream-Level Violation

Malformed log frame may close log stream.

---

# 141. Session-Level Violation

Invalid registration/control semantics may close connection.

---

# 142. Security Violation

May disable/revoke runner depending policy.

---

# 143. Replay

MessageId + lease/session binding prevents harmful replay of state-changing commands.

TLS also protects channel.

---

# 144. Old Lease Replay

Rejected because current attempt/lease differs or expired.

---

# 145. Old Session Replay

Lease bound to AgentSessionId where required.

---

# 146. Timestamp Trust

Do not trust peer `sent_at` for authority.

Use for diagnostics only.

Control plane uses own clock.

---

# 147. Clock Sync

Protocol may communicate server time for diagnostics/deadline conversion.

---

# 148. Deadlines

Send authoritative absolute expiry + local duration guidance.

---

# 149. Sensitive Data

Control messages may reference SecretRef but should not normally carry long-lived secret material.

---

# 150. Secret Delivery

If secrets must pass daemon→agent:

use dedicated encrypted/authenticated channel and short-lived payload.

Same QUIC TLS connection can carry, but logical protocol must prevent logging/persistence.

---

# 151. Secret Wire Type

```rust
pub struct SecretMaterialWire {
    pub lease: LeaseId,
    pub secret_id: SecretRef,
    pub bytes: ZeroizingBytes,
    pub expires_at: Timestamp,
}
```

Detailed secret architecture later.

---

# 152. No Secret Debug

Wire secret types implement redacted Debug.

---

# 153. Compression

Control messages usually small; no compression necessary.

Logs/CAS may compress at application layer when beneficial.

---

# 154. Compression Bomb

Bound decompressed size.

---

# 155. Log Compression

Optional chunk-level compression.

---

# 156. CAS Compression

CAS bytes are exact logical content; transport compression must decompress before digest verification.

---

# 157. Protocol Documentation

```text
protocols/internal/
├── agent-daemon.md
├── session.md
├── leases.md
├── heartbeat.md
├── logs.md
├── cas-transfer.md
├── reconnect.md
├── compatibility.md
└── security.md
```

---

# 158. Message Registry

Document stable message kinds and version.

---

# 159. Protocol IDs

```rust
pub struct MessageTypeId(u16);
```

Could be explicit if framing requires.

---

# 160. Avoid Implicit Rust Enum Discriminants

Wire discriminants must be stable.

---

# 161. Versioned Message Example

```rust
pub enum AgentControlMessageV1 {
    Register(RegisterAgentV1),
    LeaseOffer(LeaseOfferV1),
    Cancel(CancelAttemptV1),
    CapabilityUpdate(CapabilityUpdateV1),
}
```

---

# 162. Future V2

Breaking changes:

```text
AgentControlMessageV2
```

not silently mutate V1 decode semantics.

---

# 163. Optional Fields

Use only when absence has clear older-version meaning.

---

# 164. Unknown Fields

Postcard/serde compatibility needs careful schema discipline.

Test every compatibility case.

---

# 165. Protocol Test Vectors

Store encoded golden vectors for released wire schemas.

---

# 166. Golden Vector Caution

Useful for stable externalized wire formats.

Required for protocol compatibility.

---

# 167. Cross-Version Tests

```text
daemon N ↔ agent N
daemon N ↔ agent N-1
daemon N-1 ↔ agent N where supported
```

---

# 168. Protocol Testkit

```text
forgeyard-protocol-testkit/src/
├── lib.rs
├── vectors.rs
├── fake_peer.rs
├── compatibility.rs
├── framing.rs
├── malformed.rs
└── assertions.rs
```

---

# 169. Transport Testkit

```text
forgeyard-transport-testkit/src/
├── lib.rs
├── in_memory.rs
├── flaky.rs
├── latency.rs
├── bandwidth.rs
├── disconnect.rs
└── tls.rs
```

---

# 170. In-Memory Transport

Useful for service tests without real QUIC.

Must preserve semantic stream/message behavior enough for tests.

---

# 171. Real QUIC Integration Tests

Still required.

---

# 172. TLS Tests

Test:

```text
valid cert
expired cert
wrong CA
revoked runner
server-name mismatch
```

---

# 173. Version Negotiation Tests

No overlap -> reject cleanly.

---

# 174. Duplicate Message Tests

Lease/completion/cancel.

---

# 175. Reordering Tests

Independent streams can reorder globally.

Ensure control semantics do not depend on log/CAS ordering.

---

# 176. Disconnect Tests

At:

```text
registration
lease delivery
running
log streaming
CAS upload
completion ack
```

---

# 177. Completion Unknown Test

Send completion, daemon commits, drop connection before ACK, reconnect, resend same MessageId -> DuplicateAccepted.

---

# 178. Session Fencing Test

T1 active, T2 reconnect same AgentSessionId, T1 messages rejected/fenced.

---

# 179. Old Agent Session Test

A1 lease, process restarts A2, A1 completion replay rejected.

---

# 180. Large Log Test

Control cancellation remains responsive while log stream saturated.

---

# 181. Large CAS Test

Heartbeat/lease renewal unaffected by CAS transfer.

---

# 182. Malformed Frame Fuzzing

High priority.

---

# 183. Postcard Decode Fuzzing

Fuzz each public internal wire enum.

---

# 184. Stream Header Fuzzing

Malformed StreamKind/header must not panic.

---

# 185. TLS/QUIC Fuzz Scope

Underlying library handles packet parsing; Forgeyard fuzzes application framing/state.

---

# 186. Protocol State Machine

Session-level allowed order:

```text
Connected
  ↓
Hello
  ↓
Negotiated
  ↓
Registered
  ↓
Active
```

Messages outside legal state rejected.

---

# 187. Before Registration

Allowed:

```text
Hello
Register
```

Not:

```text
LeaseAccepted
Completion
```

---

# 188. Active Session

Normal control messages.

---

# 189. Draining

No new lease offers.

Existing control allowed.

---

# 190. Closing

No new work.

---

# 191. Peer Role Protocol

Signing worker should not send:

```text
Agent LeaseAccepted for arbitrary build
```

unless role supports.

---

# 192. Role Capabilities

Protocol enforces allowed message families per role.

---

# 193. Authorization

Authenticated runner may access:

```text
its own leases
authorized CAS objects
its attempt control
```

not arbitrary project data.

---

# 194. Tenant Context

Lease/spec carries project/tenant scope where needed, but agent authorization is based on server-issued work.

---

# 195. No Agent Querying Metadata

Agent should not have general APIs like:

```text
list all projects
```

---

# 196. Public REST Separation

Public API:

```text
Axum
JSON
OAuth/OIDC session
```

Internal:

```text
QUIC
Postcard
mTLS runner identity
```

---

# 197. Public WebSocket/SSE

For UI live events.

Do not tunnel agent protocol through browser WebSocket by default.

---

# 198. CLI

Normal CLI uses public/admin HTTP API.

Agent daemon control uses QUIC.

---

# 199. gRPC

Use only for interoperability where required:

```text
Bazel RBE
external ecosystem
```

not Forgeyard native internal protocol.

---

# 200. Protocol Adapter

RBE gRPC converts to Forgeyard internal services/domain.

---

# 201. REST JSON Types

Separate `forgeyard-api-model`.

---

# 202. JSON Only When Necessary

Provider APIs, REST, webhooks.

---

# 203. Internal Protocol Version Recording

Attempt/run audit may record:

```text
agent version
protocol version
```

for debugging.

---

# 204. Connection Metadata

Store ephemeral:

```text
last connected
remote address
protocol
agent version
```

not sensitive certificate private data.

---

# 205. Remote Address

Diagnostic only.

Do not use IP as identity.

---

# 206. NAT Reconnect

IP may change.

Runner identity remains certificate/RunnerId.

---

# 207. Metrics

Transport metrics:

```text
quic_connections_active
quic_connections_total
transport_reconnects
transport_handshake_latency
transport_registration_failures
transport_bytes_in
transport_bytes_out
transport_streams_active
transport_control_queue_depth
transport_log_queue_depth
transport_cas_queue_depth
transport_protocol_errors
transport_session_fenced
```

---

# 208. Message Metrics

Low-cardinality:

```text
message family
success/error
```

Not MessageId.

---

# 209. Latency Metrics

```text
lease_offer_to_ack
cancel_delivery
heartbeat RTT
completion_ack
```

---

# 210. Tracing

Spans:

```text
transport.connect
transport.handshake
protocol.negotiate
agent.register
lease.dispatch
lease.ack
attempt.complete
cas.stream
log.stream
reconnect.resync
```

---

# 211. Trace Propagation

CorrelationId/trace context can propagate across daemon-agent.

Use W3C trace IDs where observability subsystem specifies.

---

# 212. Privacy

Do not log:

```text
secret payload
private keys
bearer tokens
full sensitive job env
```

---

# 213. Health

Daemon transport health:

```text
listener active
cert valid
connection count
handshake failures
```

Agent transport health:

```text
connected
last heartbeat ack
protocol
endpoint
```

---

# 214. Doctor

```text
forgeyard transport doctor
forgeyard-agent doctor
```

Checks:

```text
UDP reachability
TLS trust
protocol compatibility
MTU issues where detectable
clock sanity
```

---

# 215. QUIC MTU

QUIC handles path MTU discovery, but operational issues can occur.

Expose diagnostics.

---

# 216. Keepalive

Use carefully.

Enough to detect dead NAT mappings without excessive traffic.

---

# 217. Idle Timeout

Long build with no control activity still has heartbeat.

---

# 218. Connection Idle Policy

Heartbeat prevents idle disconnect.

---

# 219. Retry Storm

Many agents reconnecting after daemon outage can create thundering herd.

Use jittered exponential backoff.

---

# 220. Server Admission

Rate-limit registration handshakes if overloaded.

---

# 221. Handshake DoS

TLS/QUIC implementation plus application registration limits.

---

# 222. Per-Runner Connection Limit

Usually one active AgentSession connection.

---

# 223. Per-IP Limit

Can supplement DoS control, but NAT may share many runners.

Do not rely solely on IP.

---

# 224. Certificate Revocation

Trust subsystem can revoke runner.

Daemon closes active connection.

---

# 225. Runner Disabled

Close/fence active session after cancellation/drain policy.

---

# 226. Graceful Daemon Shutdown

Daemon:

```text
stop new registrations/leasing
send reconnect/drain hints if useful
close connections gracefully
```

Existing jobs may reconnect to another daemon.

---

# 227. HA Daemons

Agent endpoint list points to cluster.

Any compatible daemon can accept reconnect.

---

# 228. Session State in HA

Critical state is persisted:

```text
RunnerId
AgentSessionId
leases
attempts
```

TransportSessionId is per-daemon ephemeral.

---

# 229. Reconnect to Different Daemon

New daemon reads authoritative store and returns Continue/Cancel/Stale.

---

# 230. Sticky In-Memory State

Must not be required for correctness.

---

# 231. Stream Recovery

QUIC streams do not survive connection loss.

Application-level resume handles:

```text
logs
CAS partial transfers
completion retries
```

---

# 232. Control Message Recovery

State reconcilers + idempotent commands.

---

# 233. Log Recovery

Sequence-based resume.

---

# 234. CAS Recovery

Digest/range/chunk-based resume.

---

# 235. Heartbeat Recovery

No need to replay every missed heartbeat.

Send fresh state.

---

# 236. Capability Recovery

Digest comparison.

---

# 237. Message Delivery Semantics

Control:

```text
at least once for retryable state-changing messages
```

Heartbeats:

```text
best effort repeated
```

Logs:

```text
ordered by per-attempt sequence with resumable gaps
```

CAS:

```text
verified stream with resumable ranges/chunks
```

---

# 238. No Exactly-Once Claim

Network delivery is not exactly once.

State effects are made idempotent.

---

# 239. Ordered Control

Within one control stream, frames ordered.

Across reconnect, application IDs handle duplicates.

---

# 240. Multi-Stream Global Order

Do not assume.

---

# 241. Cancel vs Log

Cancel may arrive before previously sent log frames.

Correct.

---

# 242. Completion vs Final Logs

Completion may be accepted while final log chunk still flushing if protocol allows.

Recommended:

```text
runner finalizes durable log refs before completion
```

for complete history, but live tail may lag.

---

# 243. Finalization Barrier

Agent can send:

```text
logs finalized
outputs finalized
completion
```

---

# 244. Job Success Requirements

Run/Job architecture decides which evidence must be committed.

Transport only carries refs.

---

# 245. Protocol Extensibility

New optional features advertised in capability/feature negotiation.

---

# 246. Feature Flags

```rust
pub enum ProtocolFeature {
    LogResumeV1,
    CasRangeResumeV1,
    DeviceControlV1,
    ...
}
```

---

# 247. Feature Negotiation

Both sides enable only shared features.

---

# 248. Major Semantics

Do not use feature bits to hide truly breaking protocol changes forever.

---

# 249. Compatibility Manifest

Each release publishes:

```text
daemon protocol support
agent support
feature set
```

---

# 250. Upgrade Order

Recommended:

```text
upgrade daemon first
then agents
```

when daemon N supports N-1 agents.

---

# 251. Downgrade

Supported only within documented compatibility.

---

# 252. Agent Too New

Daemon rejects with clear supported range.

---

# 253. Agent Too Old

Same.

---

# 254. CLI Error

Agent logs actionable:

```text
server supports 3-4, agent supports 1-2
```

---

# 255. Protocol Code Generation?

Not required.

Rust types can define schemas directly.

But stable schema discipline/testing mandatory.

---

# 256. Schema Definition Source

Rust wire structs are implementation source of truth.

Docs/test vectors mirror.

---

# 257. External IDL

Could adopt later for non-Rust interoperability.

Not needed for pure Rust native protocol.

---

# 258. Postcard + Serde Risks

Serde derive changes can alter wire layout.

Therefore:

```text
wire structs frozen/versioned
```

and reviewed carefully.

---

# 259. Wire Struct Rules

1. no casual field reorder;
2. no implicit default semantics without tests;
3. stable enum variants;
4. bounded collections;
5. explicit versioning.

---

# 260. Message Conversion

Wire decode:

```text
bytes
  ↓
wire type
  ↓
validate
  ↓
domain command
```

Never call domain service with unvalidated wire payload.

---

# 261. Validation Examples

```text
LeaseId parse
size bounds
attempt belongs to runner context
timestamp sane
capability schema supported
```

---

# 262. Error Response

Protocol error reply can include:

```text
stable code
safe message
correlation id
```

---

# 263. Protocol Error

```rust
pub struct WireErrorV1 {
    pub code: ProtocolErrorCode,
    pub retry: RetryClass,
    pub message: BoundedString,
}
```

---

# 264. No Internal Stack Trace

Never send raw backtrace to peer.

---

# 265. Retry Guidance

Useful for:

```text
temporary unavailable
rate limited
stale
unsupported
```

---

# 266. Rate Limiting

Per connection:

```text
registration
control messages
health requests
CAS requests
```

---

# 267. Heartbeat Abuse

Bound frequency; ignore excessive.

---

# 268. Log Abuse

Apply bandwidth/size quotas.

---

# 269. CAS Abuse

Object authorization + bandwidth/resource limits.

---

# 270. Security Threats

```text
runner impersonation
daemon impersonation
replay
stale lease replay
malformed Postcard
memory exhaustion
stream exhaustion
log flood
CAS flood
session hijack
protocol downgrade
```

---

# 271. Runner Impersonation Defense

mTLS/cert binding.

---

# 272. Daemon Impersonation Defense

server TLS validation.

---

# 273. Replay Defense

TLS + MessageId + lease/session checks.

---

# 274. Downgrade Defense

Version negotiation occurs inside authenticated channel and policy can reject insecure old versions.

---

# 275. Stream Exhaustion Defense

Limit streams.

---

# 276. Memory Exhaustion Defense

Bound frames and queues.

---

# 277. Log Flood Defense

Spool/rate/resource limits.

---

# 278. CAS Flood Defense

Job-authorized object set + quotas.

---

# 279. Security Audit Events

Record:

```text
auth failure
session fencing
protocol downgrade rejection
replayed stale lease
malformed security-sensitive message
runner revocation
```

---

# 280. Transport Config

Example RON:

```ron
(
    agent_transport: (
        listen: "0.0.0.0:7443",
        protocol_min: (major: 1, minor: 0),
        protocol_max: (major: 1, minor: 3),
        tls: (
            certificate: Secret("transport/server-cert"),
            private_key: Secret("transport/server-key"),
            client_ca: "config/runner-ca.pem",
        ),
    ),
)
```

---

# 281. Agent Config

```ron
(
    control_plane: (
        endpoints: [
            "quic://forgeyard.example.internal:7443",
        ],
        server_name: "forgeyard.example.internal",
        credential: Secret("runner/identity"),
    ),
)
```

---

# 282. URI Scheme

Custom user-facing config scheme may be:

```text
quic://
```

or Forgeyard-specific.

Implementation normalizes.

---

# 283. No Raw Private Key in RON

Use SecretRef/file protected config.

---

# 284. Transport CLI

```text
forgeyard transport status
forgeyard transport sessions
forgeyard transport doctor
forgeyard transport protocols
```

---

# 285. `transport sessions`

Admin view:

```text
runner
agent session
connected daemon
protocol
agent version
last heartbeat
```

---

# 286. Agent Debug

```text
forgeyard-agent connection status
```

---

# 287. Protocol Inspector

Tool:

```text
tools/forgeyard-protocol-inspect/
```

Can decode captured/test Postcard frames.

Must not expose secret payload carelessly.

---

# 288. Test Capture

Use synthetic data.

---

# 289. Debug Logging

Message type/ID, not entire payload by default.

---

# 290. Implementation Phase 1 — Version/Envelope

Implement:

```text
ProtocolVersion
ProtocolRange
Envelope<T>
MessageId
CorrelationId
```

---

# 291. Phase 2 — QUIC Transport

Implement:

```text
TLS
connect/listen
stream abstraction
close codes
```

---

# 292. Phase 3 — Registration

Implement:

```text
Hello
negotiation
RegisterAgent
session binding
```

---

# 293. Phase 4 — Control Protocol

Implement:

```text
lease offer
accept/reject
phase
cancel
completion
heartbeat
```

---

# 294. Phase 5 — Reconnect/Resync

Implement session fencing and active-attempt resync.

---

# 295. Phase 6 — Logs

Implement dedicated log streams + sequence resume.

---

# 296. Phase 7 — CAS Transfer

Integrate missing-check and streaming object transfer.

---

# 297. Phase 8 — mTLS Rotation / HA

Multiple endpoints, reconnect to another daemon.

---

# 298. Phase 9 — Device/Signing Protocols

Typed separate worker message families.

---

# 299. Phase 10 — Hardening

Fuzzing, compatibility matrices, overload tests, security audits.

---

# 300. Acceptance Tests

1. Agent and daemon negotiate highest compatible version.
2. No common major version rejects cleanly.
3. Invalid server certificate is rejected.
4. Invalid client certificate is rejected.
5. Runner identity binds to authenticated credential.
6. Registration required before lease messages.
7. Duplicate AgentSession connection fences old transport session.
8. Lease delivery is safe under duplicate send.
9. Completion retry after lost ACK is idempotent.
10. Old LeaseId replay is rejected.
11. Old AgentSession replay is rejected.
12. Cancellation remains responsive during large log upload.
13. Heartbeat remains responsive during large CAS transfer.
14. Log sequence resumes after reconnect.
15. CAS partial transfer resumes and verifies digest.
16. Malformed Postcard frame never panics.
17. Oversized frame is rejected before unbounded allocation.
18. Stream count limit prevents exhaustion.
19. Protocol error response contains safe stable code.
20. Public REST DTO changes do not alter agent wire protocol.
21. Domain struct refactor does not change released wire vectors.
22. Daemon N communicates with agent N-1 according to matrix.
23. Agent reconnect to different daemon can resync active attempt.
24. Same AgentSession keeps attempt authority across transport reconnect.
25. New AgentSession does not inherit old lease authority.
26. Runner cannot enumerate arbitrary CAS objects.
27. Daemon cannot send arbitrary remote-shell command.
28. Signing worker rejects general execution messages.
29. Device agent accepts only device protocol family.
30. Transport queues remain bounded under log/CAS pressure.
31. mTLS credential rotation succeeds without new RunnerId.
32. Disabled runner connection is closed/fenced.
33. Backoff prevents reconnect storm.
34. Protocol inspector decodes released test vectors.
35. Forgeyard self-hosted agents use this exact protocol.

---

# 301. Production Readiness Gates

Do not call native transport/protocol production-ready until:

```text
mTLS verified
version negotiation stable
wire schemas versioned
golden test vectors exist
lease/control protocol idempotent
reconnect/resync works
session fencing works
log streams bounded/resumable
CAS transfer verified
overload/backpressure tested
malformed message fuzzing passes
N/N-1 matrix tested
metrics/doctor available
```

---

# 302. Architectural Invariants

1. native agent transport is QUIC + TLS;
2. production agent identity uses strong authentication;
3. Postcard is internal structured control encoding;
4. domain types are not wire types;
5. wire schemas are explicitly versioned;
6. MessageId is stable across retry;
7. delivery effects are idempotent;
8. control traffic is independent from log/CAS traffic;
9. cancellation cannot be blocked by bulk transfer;
10. large bytes are streamed, not giant Postcard payloads;
11. AgentSessionId differs from TransportSessionId;
12. reconnect may change TransportSessionId without changing agent authority;
13. restart changes AgentSessionId;
14. old session cannot inherit new authority;
15. one active transport session per agent session is fenced;
16. public REST/WS protocol is separate;
17. gRPC is interoperability only;
18. internal protocol does not claim exactly-once delivery;
19. stale lease replay is harmless;
20. message/frame sizes are bounded;
21. stream counts are bounded;
22. peer role constrains allowed message families;
23. agent cannot query arbitrary business metadata;
24. agent cannot receive arbitrary remote shell;
25. all compatibility claims are tested;
26. wire enum layout is never casually changed;
27. HA reconnect requires no daemon-local hidden authority;
28. logs/CAS resume at application level after connection loss;
29. secrets are never logged through wire debugging;
30. Forgeyard dogfoods its own transport protocol.

---

# 303. Final Target Architecture

```text
                    Forgeyard Daemon
                          │
                    QUIC + mTLS
                          │
                          ▼
                 TransportSessionId
                          │
                  version negotiation
                          │
                          ▼
                    AgentSessionId
                          │
                registration/resync
                          │
          ┌───────────────┼────────────────┐
          ▼               ▼                ▼
       Control           Logs             CAS
          │               │                │
          │               │                │
   lease/cancel/      sequenced        streamed
   heartbeat/result   resumable        verified
          │               │                │
          └───────────────┼────────────────┘
                          ▼
                    Forgeyard Agent
```

---

# 304. Final Architectural Position

Connection identity:

```text
authenticated RunnerId
+
AgentSessionId
+
TransportSessionId
+
negotiated protocol
```

Control delivery:

```text
Envelope<Message>
+
MessageId
+
CorrelationId
  ↓
validate
  ↓
idempotent domain operation
```

Bulk transfer:

```text
small Postcard control header
  ↓
dedicated QUIC stream
  ↓
stream bytes
  ↓
digest verification
```

Reconnect:

```text
connection lost
  ↓
new TransportSessionId
  ↓
same AgentSessionId
  ↓
registration/resync
  ↓
Continue / Cancel / Stale
```

The key guarantee is:

> **Forgeyard's internal protocol remains compact and fast without sacrificing correctness: every state-changing effect is identity-bound, versioned, authenticated, retry-safe, and independent from bulk log/artifact traffic.**

---

# 305. New-Repository Sequence

```text
01 Core Domain & Foundation
02 Storage & Metadata
03 CAS & Artifact Data Plane
04 Pipeline IR / Parsing / Planning
05 Run / Job State Machine
06 Scheduler
07 Runner / Agent
08 Sandbox & Executor
09 Transport / QUIC / Protocol
10 Events / Reconciliation
11 Policy / Authorization / Identity
12 Secrets & Trust
13 Supply Chain / SBOM / Provenance / Signing
14 Packaging
15 Release
16 Deployment
17 Observability / Health / Doctor
18 API / Axum
19 Dioxus UI
20 Device Lab
21 SCM Provider Integrations
22 HA / Coordination / Raft
23 RBE Interop
24 Plugin / Extension Architecture
25 Operations / Backup / Upgrade / DR
26 Self-Hosting / Bootstrap / Release of Forgeyard
```
