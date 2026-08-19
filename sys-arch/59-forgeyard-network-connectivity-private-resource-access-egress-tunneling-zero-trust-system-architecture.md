# 59 — Forgeyard Network Connectivity, Private Resource Access, Egress Control, Tunneling & Zero-Trust Service Connectivity System Architecture

**Document type:** Core Network Connectivity, Private Resource Access, Egress Governance, Tunneling, Relay, DNS, Service Connectivity & Zero-Trust Network System Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** runner/agent connectivity, control-plane reachability, private VPC/on-prem resource access, outbound egress policy, network namespaces, service tunnels, reverse tunnels, relay/NAT traversal, DNS policy, mTLS service identity, proxying, private endpoints, webhook ingress, build/test network isolation, site-to-site connectivity, disconnected operation, and connectivity observability  
**Architecture style:** Identity-first networking, explicit reachability grants, least-privilege egress, policy-enforced service access, mTLS, bounded tunnels, no implicit trust from network location, per-job network capability, and network effects separated from application authorization  
**Status:** Target production architecture extending the Forgeyard series beyond the original 01–26 plan  
**Relationship to prior work:** Builds on Transport/QUIC, Runner/Agent, Sandbox/Executor, Secrets/Trust, Policy/Authz, Security, Multi-Tenancy, Federation, Infrastructure-as-Code, Test Environments, Deployment, Observability, and Reliability. This subsystem defines how Forgeyard components and jobs reach each other and external/private systems without turning network access into an authorization bypass.

---

# 1. Purpose

Forgeyard workloads may need to reach:

```text
the Forgeyard control plane
CAS/object storage
private package registries
internal databases
private APIs
Kubernetes clusters
cloud provider APIs
SCM providers
secret providers
artifact registries
test sandboxes
on-prem services
air-gapped networks
```

Those resources may live behind:

```text
NAT
firewalls
VPNs
private VPCs
private subnets
corporate networks
zero-trust gateways
site-to-site links
proxies
service meshes
```

Without explicit network architecture, CI/CD often degenerates into:

```text
"open outbound internet"
"put runner in the VPC"
"give it VPN access"
"allow 0.0.0.0/0"
"open SSH from CI"
"mount internal network globally"
```

That creates excessive blast radius.

The central rule is:

> **Network reachability is a capability, not authorization. A workload being able to reach an address never means it is authorized to use the service.**

A second rule is:

> **Every job receives only the network capabilities required by its declared workload and policy; broad host/VPC access is never inherited implicitly from the runner host.**

A third rule is:

> **Private connectivity is terminated, authenticated, scoped, audited, and revocable. Forgeyard does not use hidden permanent VPN credentials baked into runners.**

---

# 2. Architectural Position

```text
                     Pipeline / Job IR
                           │
                           ▼
                    Network Requirements
                           │
                           ▼
                    Policy Evaluation
                           │
                           ▼
                 Network Capability Grant
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
         Public Egress   Private Access  Forgeyard Services
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                   Sandbox Network Plane
                           │
                           ▼
                    Observed Connections
```

Control-plane connectivity:

```text
Agent / Runner
     │
     ▼
QUIC + mTLS
     │
     ▼
Forgeyard Control Plane
```

Private service access:

```text
Job Sandbox
    │
    ▼
Scoped Network Capability
    │
    ▼
Proxy / Tunnel / Private Endpoint
    │
    ▼
Target Service
```

---

# 3. Goals

The subsystem MUST:

1. define network capability identity;
2. define job network requirements;
3. support deny-by-default networking;
4. support public egress;
5. support allowlisted egress;
6. support private network access;
7. support VPC/private subnet resources;
8. support on-prem resources;
9. support private DNS;
10. support service-specific tunnels;
11. support reverse connectivity where required;
12. support relay/NAT traversal;
13. support proxy-based access;
14. support mTLS service identity;
15. support short-lived access grants;
16. support per-job sandbox network isolation;
17. support tenant isolation;
18. support site/federation routing;
19. support air-gap/disconnected operation;
20. support webhook ingress;
21. support network policy evidence;
22. support connectivity audit;
23. support DNS controls;
24. support egress observability;
25. support network failure diagnosis;
26. support rate/connection limits;
27. support UI/API/CLI;
28. support HA;
29. support DR;
30. avoid network-location-based trust.

---

# 4. Non-Goals

This subsystem does not:

```text
replace application authentication
replace service authorization
replace cloud IAM
replace enterprise VPN products
replace a full service mesh
replace firewall appliances
replace DNS infrastructure
```

Forgeyard integrates with those systems.

---

# 5. Workspace Structure

```text
crates/network/
├── forgeyard-network/
├── forgeyard-network-model/
├── forgeyard-network-policy/
├── forgeyard-network-capability/
├── forgeyard-network-egress/
├── forgeyard-network-private/
├── forgeyard-network-tunnel/
├── forgeyard-network-relay/
├── forgeyard-network-dns/
├── forgeyard-network-proxy/
├── forgeyard-network-observe/
├── forgeyard-network-health/
└── forgeyard-network-testkit/
```

Adapters:

```text
crates/network-adapters/
├── forgeyard-network-linux/
├── forgeyard-network-windows/
├── forgeyard-network-macos/
├── forgeyard-network-kubernetes/
├── forgeyard-network-aws/
├── forgeyard-network-azure/
├── forgeyard-network-gcp/
├── forgeyard-network-wireguard/
├── forgeyard-network-iroh/
└── forgeyard-network-custom/
```

Core remains transport/provider-neutral.

---

# 6. NetworkCapabilityId

```rust
pub struct NetworkCapabilityId(Digest);
```

Immutable description of permitted network behavior.

---

# 7. JobNetworkPolicy

```rust
pub struct JobNetworkPolicy {
    pub mode: JobNetworkMode,
    pub destinations: Vec<NetworkDestinationRule>,
    pub dns: DnsPolicy,
    pub ingress: JobIngressPolicy,
}
```

---

# 8. JobNetworkMode

```rust
pub enum JobNetworkMode {
    DenyAll,
    ForgeyardOnly,
    Allowlisted,
    PublicEgress,
    PrivateScoped,
    Custom(NetworkModeId),
}
```

---

# 9. Default

For hermetic build:

```text
DenyAll
```

For control-required executor setup:

```text
ForgeyardOnly
```

For dependency fetch stage:

```text
Allowlisted
```

---

# 10. PublicEgress

Not default for protected build/release stages.

---

# 11. Network Stage Separation

Recommended pipeline pattern:

```text
resolve/fetch stage -> controlled network
realization/build stage -> network denied
test integration stage -> declared services only
publish/deploy stage -> scoped provider endpoints
```

---

# 12. Network Destination

```rust
pub enum NetworkDestinationRule {
    Domain(DomainRule),
    IpNet(IpNetRule),
    Service(ServiceRef),
    PrivateResource(PrivateResourceRef),
    ForgeyardService(ForgeyardServiceKind),
}
```

---

# 13. Domain Rules

Domain allowlists are convenient but can be weaker than exact service identity.

---

# 14. DNS Rebinding

Must be considered.

---

# 15. Domain Policy

Resolve through controlled resolver.

---

# 16. Resolved IP Validation

Can enforce allowed address classes.

---

# 17. IP Rules

CIDR ranges.

---

# 18. Broad RFC1918 Range

Avoid as default.

---

# 19. Private Resource Ref

Preferred over raw CIDR when adapter can resolve exact service.

---

# 20. PrivateResourceRef

```rust
pub struct PrivateResourceRef {
    pub id: PrivateResourceId,
    pub kind: PrivateResourceKind,
}
```

---

# 21. PrivateResourceKind

Examples:

```text
Database
KubernetesApi
ObjectStore
Registry
InternalApi
MessageBroker
Custom
```

---

# 22. Resource Binding

Maps logical private resource to site/provider-specific endpoint.

---

# 23. Network Reachability vs Service Authorization

Critical separation:

```text
network capability
    +
service identity/credential
    +
application authorization
```

All required.

---

# 24. Network Capability Grant

```rust
pub struct NetworkCapabilityGrant {
    pub id: NetworkCapabilityGrantId,
    pub job_attempt: JobAttemptId,
    pub capability: NetworkCapabilityId,
    pub expires_at: Timestamp,
}
```

---

# 25. Grant Lifetime

Bounded to job/attempt.

---

# 26. No Persistent Job VPN Credential

Critical.

---

# 27. Grant Authority

Policy + scheduler/control plane.

---

# 28. Sandbox Enforcement

Job network policy enforced at sandbox boundary where possible.

---

# 29. Linux

Potential:

```text
network namespace
nftables
eBPF/cgroup hooks
proxy
```

---

# 30. Windows

Potential:

```text
Windows Filtering Platform
container/Hyper-V networking
firewall rules
```

---

# 31. macOS

Potential:

```text
PF
network extension
VM/container boundary
```

Exact enforcement differs by platform.

---

# 32. Honest Capability

If platform cannot enforce requested isolation strongly, scheduler/policy must know.

---

# 33. NetworkIsolationStrength

```rust
pub enum NetworkIsolationStrength {
    Strong,
    Moderate,
    BestEffort,
    Unavailable,
}
```

---

# 34. High-Trust Jobs

Can require `Strong`.

---

# 35. No Pretend Isolation

Critical.

---

# 36. Runner Host Network

Runner host may possess broader connectivity than job sandbox.

---

# 37. Job Must Not Inherit Host Network Blindly

Critical.

---

# 38. Host Network Mode

Forbidden for normal untrusted jobs unless explicitly required.

---

# 39. Private Connectivity Model

Recommended:

```text
private resource
   ▲
   │
site-local gateway / connector
   ▲
   │
authenticated tunnel
   ▲
   │
job-scoped proxy/tunnel
   ▲
   │
sandbox
```

---

# 40. NetworkConnectorId

```rust
pub struct NetworkConnectorId(Ulid);
```

---

# 41. Connector

Runs near private resources.

Examples:

```text
inside VPC
inside on-prem network
inside Kubernetes cluster
inside branch site
```

---

# 42. Connector Trust

Connector is privileged connectivity component.

---

# 43. Connector Does Not Carry Application Secrets

Preferred.

---

# 44. Connector Auth

mTLS/workload identity.

---

# 45. Network Connector Registration

Explicit enrollment.

---

# 46. No Open Anonymous Relay

Critical.

---

# 47. Connector Capability

```rust
pub struct NetworkConnectorCapability {
    pub destinations: Vec<PrivateResourceId>,
    pub site: SiteId,
    pub trust: ConnectorTrustClass,
}
```

---

# 48. Connector Trust Class

```rust
pub enum ConnectorTrustClass {
    Core,
    PrivateNetwork,
    Restricted,
    Quarantined,
}
```

---

# 49. Tunnel

```rust
pub struct ServiceTunnelId(Ulid);
```

---

# 50. ServiceTunnel

```rust
pub struct ServiceTunnel {
    pub id: ServiceTunnelId,
    pub grant: NetworkCapabilityGrantId,
    pub connector: NetworkConnectorId,
    pub resource: PrivateResourceId,
    pub expires_at: Timestamp,
}
```

---

# 51. Tunnel Scope

One job/resource/protocol or bounded set.

---

# 52. No General Layer-3 VPN by Default

Critical.

Service-specific tunnel/proxy preferred.

---

# 53. Why

General L3 access exposes:

```text
unrelated databases
internal admin ports
metadata services
lateral movement paths
```

---

# 54. Layer-3 Connector

May exist for enterprise integration but higher-risk.

---

# 55. Layer3NetworkGrant

Explicit and strongly restricted.

---

# 56. Reverse Tunnel

Useful when private network cannot accept inbound connections.

Connector establishes outbound authenticated session to Forgeyard relay/control plane.

---

# 57. No Inbound Firewall Opening Required

Preferred.

---

# 58. NAT Traversal

Forgeyard agent/control connectivity may use:

```text
direct QUIC
relay
site gateway
```

---

# 59. Iroh

Can be used selectively for NAT traversal/P2P transport acceleration if appropriate.

---

# 60. Iroh Is Not Authorization

Critical.

---

# 61. Relay

```rust
pub struct RelayId(Ulid);
```

---

# 62. Relay Responsibility

Forward encrypted/authenticated traffic.

---

# 63. Relay Trust

Should not need plaintext where end-to-end encryption possible.

---

# 64. Relay Does Not Grant Capability

---

# 65. Relay Selection

Site/latency/availability.

---

# 66. Direct Connection Preferred

If safe.

---

# 67. Fallback Relay

Transparent to application protocol.

---

# 68. mTLS

Internal service-to-service.

---

# 69. ServiceIdentity

Existing workload identity.

---

# 70. Network Peer Authentication

```text
certificate
service identity
site identity
protocol role
```

---

# 71. IP Address Is Not Identity

Critical.

---

# 72. Certificate Rotation

Part 12.

---

# 73. Expired Identity

Connection denied.

---

# 74. DNS Policy

```rust
pub struct DnsPolicy {
    pub resolver: DnsResolverMode,
    pub allowed_suffixes: Vec<DomainSuffix>,
    pub blocked_suffixes: Vec<DomainSuffix>,
}
```

---

# 75. Resolver Mode

```rust
pub enum DnsResolverMode {
    ForgeyardControlled,
    SiteLocal,
    System,
    Disabled,
}
```

---

# 76. Hermetic Job

DNS disabled.

---

# 77. Private Job

Site-local/private resolver where required.

---

# 78. DNS Search Domains

Explicit.

---

# 79. No Host `/etc/resolv.conf` Inheritance by Default

Where strong isolation available.

---

# 80. Split DNS

Supported through connector/site resolver.

---

# 81. DNS Logging

Metadata-sensitive.

---

# 82. Privacy

Do not retain full query history indefinitely.

---

# 83. Egress Proxy

Forgeyard-managed proxy can enforce:

```text
destination
HTTP method
TLS policy
rate limits
audit
```

---

# 84. EgressProxyId

```rust
pub struct EgressProxyId(Ulid);
```

---

# 85. HTTPS CONNECT

Can restrict destination.

---

# 86. TLS Interception

Not baseline.

---

# 87. Why

Breaks end-to-end security and introduces sensitive CA handling.

---

# 88. Enterprise TLS Inspection

External infrastructure concern if organization requires.

Forgeyard records limitation.

---

# 89. HTTP Proxy Credentials

Short-lived.

---

# 90. Dependency Fetch

Proxy useful for registries.

---

# 91. Registry Mirrors

Part 36/52 preferred over broad internet.

---

# 92. Cloud Metadata Service Protection

Critical.

Block:

```text
169.254.169.254
provider metadata endpoints
host-local management endpoints
```

unless explicitly required by workload identity mechanism.

---

# 93. Workload Identity Metadata

If cloud SDK uses metadata endpoint, expose safe scoped identity endpoint rather than raw host instance credentials.

---

# 94. SSRF Defense

Network policy blocks internal control/metadata surfaces.

---

# 95. Loopback

Sandbox-local only.

---

# 96. Host Gateway

Blocked by default.

---

# 97. Docker Socket

Not network but related escape surface.

Forbidden baseline.

---

# 98. Private Database Access

Flow:

```text
job
  ↓
network grant
  ↓
private connector
  ↓
DB endpoint
+
fresh DB credential SecretRef
```

---

# 99. Network Grant Alone

Cannot authenticate to DB.

---

# 100. Database Credential Scope

Separate.

---

# 101. Kubernetes API Access

Prefer scoped service account/token.

Network tunnel grants reachability only.

---

# 102. Cloud Provider API

Can use public endpoint + workload identity.

---

# 103. Private Cloud Endpoint

Connector when required.

---

# 104. Artifact Registry

Can be private endpoint.

---

# 105. SCM Provider

Usually public allowlist.

---

# 106. Webhook Ingress

Inbound public edge.

---

# 107. Webhook Gateway

```text
Internet
  ↓
TLS termination
  ↓
rate limit
  ↓
provider signature verification
  ↓
dedupe/persist
  ↓
internal event
```

---

# 108. Webhook Endpoint

Does not expose internal control network.

---

# 109. Reverse Proxy

Thin edge.

---

# 110. No Direct DB/CAS from Internet

Critical.

---

# 111. API Ingress

Part 18.

Network subsystem provides:

```text
TLS
routing
rate/connection protections
```

Auth remains API/authz.

---

# 112. Runner Ingress

Prefer agent-initiated outbound connection.

---

# 113. No Runner Public Listening Port Baseline

Critical.

---

# 114. Debug Session

Part 48.

Remote debug tunnel is explicit high-risk grant.

---

# 115. Debug Tunnel

```rust
pub struct DebugTunnelGrant {
    pub session: DebugSessionId,
    pub principal: PrincipalId,
    pub expires_at: Timestamp,
}
```

---

# 116. Debug Tunnel

Bound to user/session/sandbox.

---

# 117. No General SSH-to-runner Fleet

Critical.

---

# 118. Port Forwarding

Possible for preview/test environments.

---

# 119. PortForwardGrant

Explicit.

---

# 120. Example

Local browser -> private preview service.

---

# 121. Preview Environment Access

Part 53.

Can be:

```text
public authenticated
private tunnel
team-only
```

---

# 122. Test Environment

Part 56.

External service access follows test network policy.

---

# 123. Fault Injection

Cannot accidentally target production due network binding checks.

---

# 124. Network Environment Class

```rust
pub enum NetworkEnvironmentClass {
    Test,
    Preview,
    Staging,
    Production,
    ControlPlane,
}
```

---

# 125. Destructive Test Grant

Production class denied.

---

# 126. Service Connectivity Contract

```rust
pub struct ServiceConnectivityRequirement {
    pub source: ConnectivitySubject,
    pub destination: PrivateResourceId,
    pub protocol: NetworkProtocol,
    pub ports: PortSet,
}
```

---

# 127. Network Protocol

```rust
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Http,
    Https,
    Quic,
    Custom(u16),
}
```

---

# 128. Port Range

Bounded.

---

# 129. `Any Port`

High-risk.

---

# 130. East-West Control Plane

Internal Forgeyard services use explicit service identities.

---

# 131. Network Segmentation

Planes:

```text
control
execution
data
observation
trust
```

Existing five-plane architecture.

---

# 132. Segmentation Goal

Execution plane cannot directly reach trust-plane signing/admin services.

---

# 133. Signing Worker Network

Very restrictive.

---

# 134. Signing Worker Can Reach

```text
artifact input
signing service/KMS
result publication
```

not arbitrary internet.

---

# 135. Device Agent Network

Separate.

---

# 136. Mobile Device

Should not access control DB.

---

# 137. CAS Access

Through authenticated API/data endpoint.

---

# 138. Direct Object Store Presigned URL

Short-lived/scoped.

---

# 139. Presigned URL

Network access still bearer authorization at storage layer.

---

# 140. Limit Scope/expiry.

---

# 141. Tenant Network Isolation

One tenant's network grant cannot resolve/use another tenant's private connector/resource.

---

# 142. PrivateResourceId

Tenant/org scope.

---

# 143. Connector Multi-Tenant

Possible only with policy and strong logical isolation.

---

# 144. Dedicated Connector

Preferred for high-assurance tenants.

---

# 145. Network Policy Evaluation

Inputs:

```text
tenant
project
job
environment
resource
site
trust
```

---

# 146. Policy Digest

Recorded with grant.

---

# 147. Grant Freshness

If policy changes/revokes, active grant may be terminated depending severity.

---

# 148. NetworkGrantState

```rust
pub enum NetworkGrantState {
    Pending,
    Active,
    Expired,
    Revoked,
    Failed,
}
```

---

# 149. Revocation

Must propagate quickly for high-risk grants.

---

# 150. Kill Switch

Part 40 can revoke:

```text
all private tunnels
all public egress
specific connector
specific tenant/resource
```

---

# 151. Network Capability Cache

Optimization.

---

# 152. Never Cache Revoked Grant Beyond validity.

---

# 153. Rate Limits

```rust
pub struct NetworkRateLimit {
    pub bytes_per_second: Option<u64>,
    pub connections_per_second: Option<u64>,
    pub max_concurrent_connections: Option<u32>,
}
```

---

# 154. Network Quotas

Part 27.

---

# 155. Egress Cost

Part 45.

Track:

```text
internet egress
cross-region
relay traffic
artifact transfer
```

---

# 156. Cost Does Not Override Security

---

# 157. Federation

Part 51.

Connector/site selection respects:

```text
residency
site trust
availability
latency
```

---

# 158. Site-Local Connector

Preferred for local private resources.

---

# 159. Cross-Region Tunnel

Allowed only if policy/residency permits.

---

# 160. Disconnected Site

Uses local connectors/resources only.

---

# 161. Air-Gapped Site

No public egress.

---

# 162. Air-Gap Policy

Machine enforced.

---

# 163. Federation Relay

May connect sites.

---

# 164. Authority

Network control metadata has one mutable authority domain.

---

# 165. Network Connector Offline

Existing tunnels fail; new grants blocked/retry.

---

# 166. Resilience

Part 50.

Possible SLOs:

```text
agent connectivity
private tunnel establishment
DNS resolution
connector availability
```

---

# 167. Network Failure Classification

Part 48.

Distinguish:

```text
DNS
route
TLS
auth
policy deny
remote service
timeout
```

---

# 168. ConnectivityDiagnostic

```rust
pub enum ConnectivityFailureClass {
    DnsFailure,
    RouteUnavailable,
    PolicyDenied,
    IdentityRejected,
    TlsFailure,
    Timeout,
    ConnectionReset,
    RemoteServiceError,
    Unknown,
}
```

---

# 169. No "Network Error" Generic Only

Critical.

---

# 170. Network Doctor

Can test:

```text
control plane
CAS
connector
DNS
private resource reachability
```

without exposing secrets.

---

# 171. `forgeyard network doctor`

---

# 172. Network Observation

Per grant/job.

---

# 173. ConnectionObservation

```rust
pub struct ConnectionObservation {
    pub job_attempt: JobAttemptId,
    pub destination: NetworkDestinationSummary,
    pub protocol: NetworkProtocol,
    pub outcome: ConnectionOutcome,
}
```

---

# 174. Payload

Not captured by default.

---

# 175. Metadata

Minimal.

---

# 176. Privacy

Avoid full browsing/network surveillance.

---

# 177. Egress Violation

Attempt outside grant.

---

# 178. NetworkViolationEvent

```rust
pub struct NetworkViolationEvent {
    pub job_attempt: JobAttemptId,
    pub attempted: NetworkDestinationSummary,
    pub policy: NetworkCapabilityId,
}
```

---

# 179. Violation Response

Can:

```text
block
fail job
alert
```

according policy.

---

# 180. Repeated Violation

Security signal.

---

# 181. DNS Violation

Blocked suffix/query.

---

# 182. Metadata Endpoint Attempt

High-signal security event.

---

# 183. Sandbox Escape Attempt

If job reaches host management interface, security event.

---

# 184. Network Logs

Structured.

---

# 185. Retention

Part 46.

---

# 186. Sensitive Destinations

May be redacted for lower-privilege viewers.

---

# 187. Audit

Audit:

```text
private connector enrollment
network policy change
production private grant
debug tunnel
break-glass L3 access
connector quarantine
```

---

# 188. Routine connection observations

Operational telemetry.

---

# 189. Dioxus UI

Pages:

```text
Network
Connectors
Private Resources
Egress Policies
Tunnels
DNS
Violations
```

---

# 190. Connector Detail

Shows:

```text
site
trust
resources
health
last heartbeat
```

---

# 191. Network Policy Detail

Shows:

```text
mode
destinations
DNS
ingress
isolation strength
```

---

# 192. Job Detail

Can show:

```text
declared network requirements
effective grant
blocked attempts
```

---

# 193. CLI

```text
forgeyard network policy show
forgeyard network connector list
forgeyard network resource list
forgeyard network tunnel list
forgeyard network test
forgeyard network doctor
```

---

# 194. API

Potential:

```text
GET  /v1/network/connectors
GET  /v1/network/resources
GET  /v1/network/grants
POST /v1/network/connectors
POST /v1/network/test
POST /v1/network/grants/{id}/revoke
```

---

# 195. Permissions

```text
network.read
network.connector.manage
network.resource.manage
network.policy.manage
network.debug_tunnel
network.breakglass
```

---

# 196. Connector Enrollment

High privilege.

---

# 197. Break-Glass L3 Access

Highest privilege/audited/time-bounded.

---

# 198. Break-Glass

Cannot disable service authentication.

---

# 199. Network Config

Part 39.

Examples:

```text
relay endpoints
DNS servers
proxy endpoints
connector mappings
```

---

# 200. Mutable Runtime Config

Validated.

---

# 201. Private Resource Definition

Can be managed in RON/org config.

---

# 202. Secret References

No embedded passwords.

---

# 203. Connector Bootstrap

Single-use token.

---

# 204. Runtime mTLS

After enrollment.

---

# 205. Connector Binary

Part 41 update delivery.

---

# 206. Connector Baseline

Can use Part 58 image baseline.

---

# 207. Connector Compromise

Part 40.

Quarantine:

```text
revoke cert
stop new grants
terminate tunnels
invalidate site/resource routes
```

---

# 208. Existing Job

Depending severity, fail/terminate.

---

# 209. Private Resource Credential

Rotate separately if exposed.

---

# 210. No Assumption Tunnel Compromise = Service Credential Compromise, but investigate.

---

# 211. Service Mesh Integration

Optional.

---

# 212. Forgeyard can route through existing mesh gateway.

---

# 213. It does not require mesh adoption.

---

# 214. WireGuard

Optional connector transport.

---

# 215. QUIC

Preferred native Forgeyard transport where suitable.

---

# 216. HTTP CONNECT/SOCKS

Possible egress mechanisms.

---

# 217. Protocol Adapter

Network architecture stays capability-based.

---

# 218. Unix Socket

Local IPC can avoid network.

---

# 219. Local Agent/Daemon

Use local IPC where appropriate.

---

# 220. No localhost TCP exposure unnecessarily.

---

# 221. Kubernetes

Runner pod network policy.

---

# 222. Kubernetes NetworkPolicy

Adapter can enforce.

---

# 223. ServiceAccount

Separate authorization.

---

# 224. CNI Limitations

Isolation strength recorded.

---

# 225. Cloud Security Groups

Coarse host/site boundary.

---

# 226. Not sufficient for per-job policy by itself.

---

# 227. Egress Gateway

Useful for centralized auditing/allowlisting.

---

# 228. Regional Egress

Can preserve source IP reputation.

---

# 229. IP Allowlist External Service

If vendor requires fixed source IP, route through controlled egress gateway.

---

# 230. Fixed Egress Identity

```rust
pub struct EgressIdentityId(Ulid);
```

---

# 231. Egress Identity

Not authentication credential.

---

# 232. Vendor API Auth

Still separate token/OIDC.

---

# 233. Domain Verification

TLS certificate required.

---

# 234. Plain HTTP

Blocked by default outside localhost/test unless explicit.

---

# 235. TLS Policy

```rust
pub struct TlsClientPolicy {
    pub minimum_version: TlsVersion,
    pub custom_roots: Vec<TrustBundleRef>,
}
```

---

# 236. Custom CA

Project/tenant-scoped.

---

# 237. No Global Trust of Arbitrary Project CA

Critical.

---

# 238. TLS Pinning

Optional for high-assurance internal services.

---

# 239. Certificate Revocation

Trust subsystem.

---

# 240. Proxy CA

If organization uses inspection, explicit installation in runner baseline/profile.

---

# 241. Hermetic Builds

Network = denied.

---

# 242. Dependency Fetch

Use curated mirrors.

---

# 243. Reproducibility

Any allowed network input must resolve to fixed-output content where required.

---

# 244. Network Determinism

Network access itself is not deterministic.

Therefore exact downloaded content enters derivation through digest/lock.

---

# 245. Release/Signing

Release publication endpoints allowlisted.

---

# 246. Signing worker should not need broad internet.

---

# 247. Deployment

Deployment adapters need environment/provider endpoints.

---

# 248. Deployment Network Grant

Exact environment/provider scope.

---

# 249. IaC

Part 53 plan/apply worker network differs.

---

# 250. Plan

Read-only provider endpoints where possible.

---

# 251. Apply

Write-capable provider endpoint reachability.

---

# 252. AI

Part 55 remote model endpoint egress policy.

---

# 253. Source-Egress Policy

AI remote provider destination explicitly controlled.

---

# 254. Artifact Registry

Part 52 internal registry may be private resource.

---

# 255. Test Data

Part 56 external test sandbox access allowlisted.

---

# 256. Device Lab

Devices may require outbound internet/test backend.

Explicit device network profiles.

---

# 257. DeviceNetworkProfileId

```rust
pub struct DeviceNetworkProfileId(Digest);
```

---

# 258. Test Device

Can be isolated Wi-Fi/VLAN.

---

# 259. Production Device Account

Not used.

---

# 260. Network Policy Compilation

High-level rules compile to platform enforcement.

---

# 261. NetworkPolicyCompiler

```rust
pub trait NetworkPolicyCompiler {
    fn compile(
        &self,
        policy: &JobNetworkPolicy,
        platform: &RunnerPlatform,
    ) -> Result<CompiledNetworkPolicy, NetworkPolicyError>;
}
```

---

# 262. Compiled Policy Digest

Recorded.

---

# 263. Enforcement Drift

Runtime can verify applied policy digest.

---

# 264. NetworkPolicyAttestation

```rust
pub struct NetworkPolicyAttestation {
    pub job_attempt: JobAttemptId,
    pub expected: Digest,
    pub observed: Option<Digest>,
}
```

---

# 265. If Enforcement Missing

High-trust job fails to start.

---

# 266. Best-Effort Profile

Allowed only where policy permits.

---

# 267. Retry

Network policy setup failure = infrastructure failure.

---

# 268. No Run Without Requested Isolation

Critical.

---

# 269. Connector Routing

Resource -> eligible connector set.

---

# 270. Hard Filters

```text
tenant
site
resource
trust
health
residency
```

---

# 271. Soft Score

```text
latency
load
cost
```

---

# 272. Connector Failover

Another equivalent connector can be used.

---

# 273. Tunnel Session Identity

New ephemeral session.

---

# 274. No Long-Lived Shared Tunnel Between Unrelated Jobs

Critical.

---

# 275. Pooling

Transport connection pooling may exist beneath isolated logical sessions.

---

# 276. Logical isolation mandatory.

---

# 277. Rate/Abuse Protection

Build jobs can accidentally generate traffic storms.

---

# 278. Egress Limit

Policy.

---

# 279. DDoS Abuse

Outbound traffic caps.

---

# 280. Crypto Mining/Malware

Network anomalies security signal.

---

# 281. No Content Inspection Required for Baseline enforcement.

---

# 282. Network Anomaly Detection

Optional advisory.

---

# 283. AI

Can summarize anomalies but not block autonomously unless deterministic threshold policy.

---

# 284. Observability Metrics

```text
network_grants_total
network_grant_denied_total
network_tunnels_active
network_tunnel_failures_total
network_policy_violations_total
network_connector_health
network_relay_bytes_total
network_dns_failures_total
```

---

# 285. Labels

Low-cardinality:

```text
mode
result
connector_class
failure_class
```

---

# 286. No destination hostname labels in general metrics.

---

# 287. Tracing

```text
network.grant
network.compile
network.enforce
network.connect
network.tunnel
network.resolve
network.revoke
```

---

# 288. Sensitive Data

Destination summaries may be redacted.

---

# 289. Reliability

Connection SLO by service class.

---

# 290. Doctor

```text
forgeyard network doctor
```

Checks:

```text
agent->control QUIC
mTLS
DNS
CAS
registry
connector
relay
private resource
egress proxy
policy enforcement
```

---

# 291. Network Test

Uses synthetic reachability probes.

---

# 292. No Real destructive query.

---

# 293. Health

```rust
pub enum NetworkSubsystemHealth {
    Healthy,
    Degraded,
    RelayDegraded,
    ConnectorDegraded,
    PolicyEnforcementDegraded,
    Unhealthy,
}
```

---

# 294. Policy Enforcement Degraded

High-assurance execution may stop.

---

# 295. HA

Relays/connectors can be redundant.

---

# 296. State

Network grants persisted.

---

# 297. Tunnel sessions ephemeral but reconstructable/retriable.

---

# 298. Reconciler

Checks:

```text
expired grants
revoked policy
dead connectors
orphan tunnels
enforcement drift
```

---

# 299. No Raft Requirement

Normal metadata DB.

---

# 300. Federation

Authority for mutable connector/resource config belongs to one site/domain.

---

# 301. DR

Connector definitions backed up.

---

# 302. Certificates

Re-enrolled if trust recovered.

---

# 303. Relay endpoints

Rebuildable.

---

# 304. Private Network Recovery

DR runbook includes connectivity verification.

---

# 305. Testkit

```text
forgeyard-network-testkit/src/
├── lib.rs
├── policy.rs
├── capability.rs
├── connector.rs
├── tunnel.rs
├── dns.rs
├── egress.rs
├── isolation.rs
└── assertions.rs
```

---

# 306. Unit Tests

Network capability identity deterministic.

---

# 307. Deny-All Test

No outbound reachability.

---

# 308. Forgeyard-Only Test

Control/CAS reachable, internet blocked.

---

# 309. Allowlist Test

Only target destinations.

---

# 310. Metadata Endpoint Test

Blocked.

---

# 311. DNS Rebinding Test

Policy enforcement survives.

---

# 312. Host Network Test

Unavailable for normal untrusted job.

---

# 313. Connector Scope Test

Job cannot reach unrelated private resource.

---

# 314. Tenant Isolation Test

Tenant A cannot use Tenant B connector/resource.

---

# 315. Tunnel Expiry Test

Access revoked.

---

# 316. Debug Tunnel Test

User/session scoped.

---

# 317. No Runner SSH Test

No public listener baseline.

---

# 318. Policy Change Test

Revoked grant terminates where required.

---

# 319. mTLS Test

Invalid site/service identity rejected.

---

# 320. Relay Test

Relay cannot grant unauthorized route.

---

# 321. Iroh Test

Connectivity does not bypass capability/authz.

---

# 322. Network Isolation Strength Test

High-trust job rejected on insufficient platform capability.

---

# 323. Egress Gateway Test

Fixed source IP still requires service auth.

---

# 324. Custom CA Test

Tenant CA does not become global trust root.

---

# 325. Plain HTTP Test

Blocked unless explicitly permitted.

---

# 326. Air-Gap Test

Public egress impossible.

---

# 327. Federation Test

Residency disallows cross-region connector.

---

# 328. Connector Failover Test

Equivalent healthy connector selected.

---

# 329. Orphan Tunnel Test

Reconciler closes.

---

# 330. Fault Injection Test

Production endpoint cannot be targeted from test profile.

---

# 331. Fuzzing

Fuzz:

```text
network policy parser
destination rules
tunnel protocol messages
DNS policy inputs
```

---

# 332. Adversarial Tests

```text
SSRF to metadata
DNS rebinding
connector impersonation
tunnel hijack
cross-tenant route request
port scanning
```

---

# 333. Chaos Tests

```text
relay outage
connector outage
DNS failure
network partition
certificate expiry
proxy failure
```

---

# 334. Scale Test

Large concurrent tunnel/grant count.

---

# 335. Implementation Phase 1 — Job Network Policy

Deny/allowlist enforcement.

---

# 336. Phase 2 — Forgeyard Service Connectivity

QUIC/mTLS/control plane.

---

# 337. Phase 3 — Egress Proxy/DNS

Controlled external access.

---

# 338. Phase 4 — Private Network Connectors

VPC/on-prem.

---

# 339. Phase 5 — Service Tunnels

Job-scoped private access.

---

# 340. Phase 6 — Relay/NAT Traversal

Remote agents/sites.

---

# 341. Phase 7 — Debug/Preview Port Forwarding

User workflows.

---

# 342. Phase 8 — Federation/Residency

Multi-region.

---

# 343. Phase 9 — Network Attestation/Drift

High assurance.

---

# 344. Phase 10 — Cost/Rate Controls

Governance.

---

# 345. Phase 11 — Device/Advanced Platform Integration

Mobile/labs.

---

# 346. Phase 12 — Adversarial/Chaos/Scale Hardening

Production readiness.

---

# 347. Acceptance Tests

1. Network reachability never grants application authorization.
2. Job network capability is explicit.
3. Hermetic build can run with DenyAll.
4. Jobs do not inherit runner host network blindly.
5. High-trust jobs can require strong network isolation.
6. Platform inability to enforce isolation is explicit.
7. Private resources are referenced/scoped, not exposed through broad VPC access by default.
8. Job private connectivity uses bounded expiring grants.
9. Connectors authenticate through workload identity/mTLS.
10. Connectors do not create anonymous relays.
11. Layer-3 VPN access is not baseline.
12. Relay/NAT traversal does not bypass policy/authz.
13. IP address/network location is never service identity.
14. DNS policy is explicit.
15. Cloud metadata/host-management endpoints are blocked by default.
16. Dependency fetch can use allowlisted/mirrored network while hermetic realization stays offline.
17. Signing workers have narrow connectivity.
18. Runners need no inbound public listener baseline.
19. Debug tunnels are explicit, authenticated, expiring, and session-scoped.
20. Test fault injection cannot target production network classes.
21. Tenant private resources/connectors are isolated.
22. Network grants record policy digest and expiry.
23. Revocation is enforceable.
24. Cross-region connector selection obeys residency.
25. Air-gapped sites have no public egress.
26. Network policy setup failure prevents high-trust job start.
27. Connection failures are classified beyond generic "network error".
28. Network telemetry avoids payload capture by default.
29. Network violation attempts are observable.
30. HA relays/connectors are reconcilable.
31. DR includes trust/connectivity reconstruction.
32. Standalone/distributed share network semantics.
33. Network policy integrates with sandbox/executor rather than replacing it.
34. Private connectivity never substitutes for SecretRef/service credential authorization.
35. Forgeyard dogfoods deny-by-default and private-connector networking for its own CI fleets.

---

# 348. Production Readiness Gates

Do not call network architecture production-ready until:

```text
deny-all/allowlist enforcement is stable
metadata endpoint/SSRF protections pass
job/host network separation is proven
private connector scoping works
mTLS connector identity works
tunnel expiry/revocation works
cross-tenant route isolation passes
air-gap/federation residency enforcement passes
network policy attestation works for high-trust profiles
adversarial/chaos tests pass
```

---

# 349. Architectural Invariants

1. reachability is not authorization;
2. network capability is explicit;
3. jobs do not inherit host reachability;
4. hermetic stages can run with zero network;
5. network isolation strength is honest;
6. private access is resource-scoped;
7. grants are short-lived;
8. connectors authenticate;
9. relays do not grant authority;
10. IP address is not identity;
11. mTLS/workload identity protects internal service connectivity;
12. broad L3 access is exceptional;
13. cloud metadata endpoints are blocked by default;
14. DNS policy is explicit;
15. private connectivity does not replace service credentials;
16. signing/trust-plane connectivity is narrow;
17. runners do not expose public listener baseline;
18. debug tunnels are explicit/expiring;
19. test networking cannot target production destructively;
20. tenant routes/connectors are isolated;
21. revocation is enforced;
22. federation obeys residency;
23. air-gap means no public egress;
24. network enforcement failure blocks high-trust jobs;
25. network observations avoid payload capture by default;
26. violations are observable;
27. connectivity failures are classified;
28. HA state is reconciled;
29. DR restores identity/connectivity deliberately;
30. Forgeyard dogfoods its own network capability model.

---

# 350. Final Target Architecture

```text
                     Job / Pipeline IR
                           │
                           ▼
                  Network Requirements
                           │
                           ▼
                     Policy Decision
                           │
                           ▼
                 NetworkCapabilityId
                           │
                           ▼
                 Ephemeral Job Grant
                           │
             ┌─────────────┼─────────────┐
             ▼             ▼             ▼
        Forgeyard-only   Public Allowlist  Private Service
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                  Sandbox Enforcement
                           │
                           ▼
                Authenticated Target Use
```

Private service access:

```text
Job Sandbox
  ↓
NetworkCapabilityGrant
  ↓
Scoped tunnel/proxy
  ↓
Authenticated connector
  ↓
Private resource endpoint
  ↓
service credential / IAM
  ↓
application authorization
```

The key guarantee is:

> **Forgeyard can connect jobs to public services, private VPCs, on-prem networks, device labs, and remote sites without treating network placement as trust. Connectivity is declared, policy-checked, scoped, short-lived, identity-authenticated, observable, and enforced at the sandbox boundary, while application credentials and authorization remain separate control layers.**

---

# 351. Extended Architecture Sequence

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
12 Secrets / Trust
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
26 Self-Hosting / Bootstrap / Release Forgeyard
27 Multi-Tenancy / Quotas / Resource Governance
28 Audit / Compliance / Security Governance
29 Notifications / Alerting / Human Workflow
30 Entitlements / Licensing / Subscription
31 Search / Indexing / Operational Analytics
32 Test Results / Quality / Coverage / Flaky Intelligence
33 Benchmark / Performance / Load / Capacity
34 Monorepo / Dependency Graph / Affected Work
35 Developer Experience / Local Dev
36 Dependency / Registry / Mirror Governance
37 Static Analysis / Security Findings
38 Cache / Remote Cache / Correctness
39 Configuration / Feature Flags / Runtime
40 Security / Threat Model / Incident Response
41 Release Distribution / Update Delivery
42 Workflow Templates / Golden Paths
43 Runner Fleet Autoscaling / Provisioning
44 Pipeline Triggers / Schedules / Dispatch
45 Cost Accounting / FinOps
46 Data Lifecycle / Retention / Privacy
47 CI/CD Migration / Compatibility
48 Failure Diagnosis / Reproduction / Bisect
49 Service Catalog / Ownership / Developer Portal
50 Reliability / SLO / Error Budget / Resilience
51 Multi-Region Federation / Edge / Disconnected
52 Artifact Registry / OCI / Package Distribution
53 Infrastructure-as-Code / Preview Environments / Drift
54 Merge Queue / Speculative Integration / Batch Validation
55 AI-Assisted CI Optimization / Engineering Copilot
56 Test Data / Fixtures / Ephemeral Databases / Service Virtualization
57 API / ABI / Schema / Protocol Compatibility / Contract Evolution
58 Runner Image Factory / Golden Images / Patch Management / Baseline Attestation
59 Network Connectivity / Private Resource Access / Egress / Tunneling / Zero-Trust Connectivity
```
