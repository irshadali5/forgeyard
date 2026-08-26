# Forgeyard Assembly & Native Object Toolchain System Architecture

**Document type:** Native low-level subsystem System & Architecture  
**Project:** Forgeyard CI/CD  
**Subsystem:** First-class Assembly, object-file, ABI, linker, startup-code, low-level verification, and native-toolchain integration  
**Implementation direction:** Rust-first Forgeyard core with explicit integration to C, C++, Rust, Objective-C, embedded, kernel, firmware, platform SDK, and cross-compilation workflows  
**Status:** Target production architecture  
**Position in Forgeyard:** Assembly is not a separate mainstream application ecosystem. It is a first-class low-level language/toolchain subsystem inside Forgeyard's native build foundation.

## 1. Purpose

Forgeyard needs explicit Assembly support because assembly often appears invisibly inside otherwise high-level projects:

```text
Rust crate -> build.rs -> cc/nasm/clang -> assembly object
C/C++ project -> .S/.s/.asm -> assembler -> object -> link
kernel/firmware -> startup assembly -> linker script -> image
```

Assembly is important for startup code, context switching, SIMD, cryptography, OS kernels, firmware, bootloaders, interrupt handlers, optimized routines, embedded systems, language runtimes, FFI/JNI boundaries, and platform integration.

> **An Assembly build is identified by source + assembler implementation + syntax + architecture + ABI + CPU feature contract + preprocessing + object format + linker/toolchain + sysroot/platform contract + controlled environment.**

## 2. Architectural Position

```text
Forgeyard Native Toolchain Foundation
├── C
├── C++
├── Objective-C
├── Assembly
├── Linkers
├── Object formats
├── ABI/calling conventions
├── Sysroots
├── libc/runtime
└── Platform SDKs
```

It integrates upward into Rust, C/C++, Swift, Java/JNI, Python native extensions, Node native addons, Dart/Flutter FFI, embedded/firmware, and OS/kernel projects.

## 3. Objectives

Forgeyard Assembly MUST support GNU as, LLVM integrated assembler, NASM, YASM where needed, MASM, architecture-specific adapters, `.s`, `.S`, `.asm`, preprocessing, Intel/AT&T/architecture-native syntax, x86/x86_64, ARM/AArch64, RISC-V, embedded targets, ELF, PE/COFF, Mach-O, raw binaries, ABI modeling, calling conventions, linker scripts, startup objects, CPU feature constraints, cross-compilation, hermetic execution, disassembly verification, reproducibility, remote execution, and integration with Forgeyard C/C++ and Rust.

## 4. Non-Goals

Forgeyard does not replace GNU `as`, LLVM assembler, NASM, MASM, linkers, objdump, llvm-objdump, or platform toolchains. Forgeyard makes them locked, typed, hermetic, observable, reproducible, and policy-controlled.

## 5. Suggested Crates

```text
crates/
├── forgeyard-asm/
├── forgeyard-asm-model/
├── forgeyard-asm-detect/
├── forgeyard-asm-toolchain/
├── forgeyard-asm-preprocess/
├── forgeyard-asm-abi/
├── forgeyard-asm-object/
├── forgeyard-asm-link/
├── forgeyard-asm-layout/
├── forgeyard-asm-verify/
├── forgeyard-asm-disasm/
├── forgeyard-asm-cross/
└── forgeyard-asm-embedded/
```

## 6. Core Domain Model

```rust
pub struct AssemblyUnit {
    pub source: SourceRef,
    pub assembler: AssemblerSpec,
    pub syntax: AssemblySyntax,
    pub architecture: CpuArchitecture,
    pub abi: AbiSpec,
    pub cpu_features: CpuFeatureSet,
    pub preprocessing: PreprocessSpec,
    pub object_format: ObjectFormat,
    pub output: AssemblyOutputKind,
}
```

## 7. Assembler Model

```rust
pub enum AssemblerKind {
    GnuAs,
    LlvmIntegrated,
    Nasm,
    Yasm,
    Masm,
    Custom(AssemblerId),
}
```

## 8. Syntax

```rust
pub enum AssemblySyntax {
    Att,
    Intel,
    Nasm,
    Masm,
    Arm,
    AArch64,
    RiscV,
    ArchitectureNative,
}
```

Syntax is explicit because the same instruction intent does not imply interchangeable source syntax.

## 9. Source Types

```text
.s    assembler source
.S    preprocessed assembler source
.asm  assembler-specific source
.inc  included fragment
```

Detection uses extension, syntax markers, build-system configuration, compiler-driver flags, assembler selection, and target architecture.

## 10. Toolchain Identity

Assembler identity includes assembler binary, exact build/version, driver mode, target support, resource files, and default object-format behavior.

```text
AssemblerId = H(tool closure)
```

Compiler-integrated assemblers such as Clang's integrated assembler are distinct from GNU `as`.

## 11. Architecture and CPU Features

```rust
pub enum CpuArchitecture {
    X86,
    X86_64,
    Arm,
    AArch64,
    RiscV32,
    RiscV64,
    Mips,
    PowerPc,
    Embedded(String),
}
```

Feature contracts can include SSE2, SSE4.2, AVX, AVX2, AVX-512, AES-NI, SHA extensions, NEON, SVE/SVE2, and RISC-V extension sets.

Portable releases must not silently use host-native CPU detection. Prefer declared baselines plus optional optimized variants and runtime dispatch.

## 12. ABI Model

```rust
pub struct AbiSpec {
    pub calling_convention: CallingConvention,
    pub stack_alignment: u16,
    pub red_zone: RedZonePolicy,
    pub symbol_prefix: SymbolPrefixPolicy,
    pub unwind: UnwindPolicy,
}
```

Support System V AMD64, Windows x64, AAPCS/AAPCS64, RISC-V psABI, cdecl, stdcall, fastcall, vectorcall, and platform-specific conventions.

Forgeyard can validate object metadata, symbols, relocations, unwind sections, and layout constraints, but cannot prove arbitrary handwritten assembly semantically correct.

## 13. Object Formats

```rust
pub enum ObjectFormat {
    Elf,
    Coff,
    PeCoff,
    MachO,
    WasmObject,
    RawBinary,
}
```

For ELF inspect class, machine, endianness, sections, symbols, relocations, notes, and unwind data. For PE/COFF inspect machine, sections, symbols/imports, relocations, and unwind metadata. For Mach-O inspect CPU type/subtype, sections, symbols, relocations, and load-command implications.

Raw binaries require explicit load address, entry offset, alignment, target, and memory layout.

## 14. Preprocessing

`.S` inputs generally use a C preprocessor/compiler driver. Preprocessor identity includes compiler/preprocessor, defines, include paths, target, and sysroot.

All includes and definitions are content-addressed build inputs. Distinguish CPP macros, GAS macros, NASM macros, MASM macros, and architecture-specific macro systems.

## 15. Derivation

```text
AssemblyDerivation =
H(
  source tree,
  assembler,
  syntax,
  architecture,
  ABI,
  CPU features,
  preprocessing,
  includes,
  defines,
  flags,
  object format,
  target,
  sysroot/platform contract
)
```

```rust
pub struct AssemblyAction {
    pub unit: AssemblyUnitId,
    pub toolchain: AssemblerId,
    pub target: NativeTarget,
    pub flags: Vec<AssemblyFlag>,
    pub output: OutputSpec,
}
```

Outputs may be Object, StaticArchiveMember, RawBinary, or StartupObject.

## 16. Link Integration

Assembly objects feed Forgeyard's native linker layer:

```text
Assembly object -> Native LinkPlan -> linker -> binary/library
```

Assembler and linker identities are separate. Linkers may include ld.bfd, gold, lld, link.exe, Apple ld, or custom embedded linkers.

```rust
pub struct NativeLinkPlan {
    pub objects: Vec<StoreObjectId>,
    pub libraries: Vec<NativeLibraryRef>,
    pub linker: LinkerId,
    pub linker_script: Option<StoreObjectId>,
    pub target: NativeTarget,
    pub entry: Option<SymbolName>,
}
```

Linker scripts are immutable inputs, especially for embedded, kernels, bootloaders, firmware, and special layout control.

## 17. Startup Code and Layout

Startup objects such as `_start`, `crt0`, vector tables, reset handlers, and boot entry code are explicit runtime inputs.

Forgeyard can validate expected entry address, sections, segment permissions, alignment, memory addresses, and symbol positions.

## 18. Symbols and Visibility

```rust
pub struct NativeSymbol {
    pub name: SymbolName,
    pub binding: SymbolBinding,
    pub visibility: SymbolVisibility,
    pub kind: SymbolKind,
}
```

Policy may require expected exports, forbid unexpected globals, or reject duplicate public symbols.

For integration with Rust/C/C++, prefer explicit stable C-compatible symbol boundaries rather than hard-coding unstable language-specific mangled names.

## 19. Rust Integration

Inline `asm!` and `global_asm!` remain part of rustc derivations, but their architecture/feature assumptions should be surfaced in low-level audit metadata where detectable.

If `build.rs` invokes `cc`, `nasm`, `as`, or `clang`, Forgeyard maps those tool accesses into this Assembly subsystem.

```text
Cargo
  ↓
build.rs
  ↓
declared assembler requirement
  ↓
Forgeyard AssemblyAction
  ↓
object
  ↓
rustc/Cargo link input
```

## 20. C/C++ Integration

CMake, Meson, Make, and Ninja projects can create AssemblyAction nodes from assembly sources.

Compiler-driver flows such as `clang -c foo.S` or `gcc -c foo.S` include both driver and underlying assembler behavior in derivation identity.

## 21. Kernel, Firmware, and Embedded

Support boot assembly, interrupt stubs, syscall entry, context switching, vector tables, reset handlers, DSP routines, and memory barriers.

Bare-metal target contracts include CPU, board/machine, memory map, endianness, ABI, linker script, and entry point.

Potential firmware artifacts include ELF, BIN, HEX, UF2 adapters, or project-defined images.

## 22. Architecture-Specific Validation

For x86/x86_64 verify mode, instruction-set assumptions, relocations, calling convention, and stack alignment.

For ARM/AArch64 verify ISA mode, Thumb where applicable, AAPCS, NEON/SVE features.

For RISC-V verify XLEN, extension set, ABI, and relaxation settings.

Endianness is explicit where relevant.

## 23. PIC / PIE / TLS / Unwind

Assembly may explicitly target PIC, PIE, or absolute/static relocation models.

Thread-local storage usage must match target ABI/linker/runtime.

Handwritten assembly crossing unwind boundaries may require CFI, SEH, or platform unwind metadata. Forgeyard can validate presence of required metadata where inspectable.

Default policy: do not unwind through unannotated assembly.

## 24. SIMD and Cryptographic Assembly

Optimized variants should carry explicit CPU feature requirements.

For cryptographic assembly, recommended additional gates include known-answer tests, cross-implementation differential tests, architecture-specific vectors, and optional constant-time analysis adapters.

Forgeyard must never infer constant-time behavior merely because code is assembly.

## 25. Disassembly and Instruction Policy

Store normalized disassembly evidence using llvm-objdump, objdump, or platform equivalents.

Disassembly is evidence, not artifact identity.

Useful policies include:

```text
forbidden instruction
required instruction
no AVX above baseline
no privileged instruction in user-space target
```

Kernel/firmware targets may allow privileged instructions.

## 26. Reproducibility

```text
AssemblyDerivation D
  ↓
Runner A -> Object X
Runner B -> Object Y
  ↓
X == Y
```

Common nondeterminism sources: timestamps, debug paths, assembler version, preprocessor macros, host include paths, linker IDs, archive timestamps, random metadata.

Use stable logical `/source`, `/build`, `/store` paths and path remapping where supported.

## 27. Archive Integration

When objects enter `.a`/`.lib`, normalize archive member ordering, metadata, and timestamps.

## 28. Verification Levels

```rust
pub enum AssemblyVerificationLevel {
    None,
    ObjectMetadata,
    AbiMetadata,
    Disassembly,
    Layout,
    FullConfiguredPolicy,
}
```

Object verification checks format, architecture, endianness, sections, symbols, and relocations.

Layout verification is especially important for kernel, bootloader, and firmware builds.

## 29. Testing

Assembly is usually tested through its caller/application, but dedicated tests may include ABI harnesses, instruction tests, known-vector tests, emulator runs, and device runs.

An ABI harness may verify preserved registers, return values, stack behavior, and calling convention.

Differential tests can compare assembly against portable C/Rust reference implementations.

## 30. Hermeticity

Assembler and preprocessor run with network denied, controlled source/include roots, controlled temp storage, declared toolchains, and declared sysroots.

Strict builds reject host include leakage such as `/usr/local/include` unless explicitly part of the platform contract.

## 31. Cross-Assembly

Cross assembly is modeled as build-host tool producing target object:

```text
build host
+
assembler target mode
+
target architecture
+
ABI
+
object format
+
sysroot
```

Host and target are always distinct concepts.

## 32. Platform Notes

### Windows
Support MASM + MSVC, NASM + COFF + MSVC/lld-link, Clang integrated assembler, and GNU assembler + MinGW as distinct toolchain contracts.

### macOS/iOS
Mach-O assembly uses Apple target conventions and Apple linker/Clang/SDK contracts when platform symbols/frameworks are needed.

### Linux
Typically ELF + System V ABI, with explicit libc/runtime/linker contracts.

### Android
Support ARM64/ARMv7/x86_64 according to Android NDK ABI/API-level policy.

### WASM
WAT is a separate low-level textual representation adapter, not native Assembly syntax.

## 33. Build-System Integration

CMake may use ASM/ASM_NASM languages and custom commands. Meson may expose assembly sources/custom targets. Make/Ninja provide explicit assembler command graphs. Cargo integrates through build.rs/cc/custom build actions.

Detection should consider actual build graph/tool invocation, not just source file extension.

## 34. Capability Registry

```rust
pub struct AssemblerCapability {
    pub assembler: AssemblerId,
    pub architectures: BTreeSet<CpuArchitecture>,
    pub object_formats: BTreeSet<ObjectFormat>,
    pub syntaxes: BTreeSet<AssemblySyntax>,
}
```

## 35. Scheduler

```rust
pub struct AssemblyRunnerCapabilities {
    pub assemblers: Vec<AssemblerId>,
    pub targets: Vec<NativeTarget>,
    pub linkers: Vec<LinkerId>,
    pub sysroots: Vec<SysrootId>,
    pub emulators: Vec<EmulatorId>,
    pub devices: Vec<DeviceCapability>,
    pub sandbox: SandboxCapabilities,
}
```

Hard constraints: assembler, architecture, object format, linker, sysroot, SDK, emulator/device, trust tier.

Score by toolchain locality, input locality, native dependency locality, queue delay, and device availability.

## 36. Remote Execution

Assembly is a strong remote-execution candidate because actions usually have small inputs/outputs:

```text
source/include closure + assembler + target settings -> object
```

Linking remains a separate action because its closure and platform requirements differ.

## 37. Cache Key

```text
H(
  source,
  includes,
  assembler,
  syntax,
  target,
  ABI,
  CPU features,
  preprocessing,
  flags,
  object format
)
```

## 38. CAS Objects

Store source snapshot, assembler toolchain, optional preprocessed source, object, disassembly evidence, and verification reports.

Preprocessed output is useful for diagnostics but may contain sensitive paths/constants, so normal artifact access policy applies.

## 39. CLI

```text
forgeyard asm detect
forgeyard asm toolchain
forgeyard asm assemble
forgeyard asm preprocess
forgeyard asm verify
forgeyard asm disasm
forgeyard asm symbols
forgeyard asm relocations
forgeyard asm layout
forgeyard asm reproduce
forgeyard asm explain
forgeyard asm explain-rebuild
```

## 40. Dioxus UI

Panels:

```text
Assembler
Target
ABI
CPU features
Object
Symbols
Sections
Relocations
Disassembly
Layout
Reproducibility
Integration owner
```

Integration owner identifies whether the object came from C/C++, Rust build.rs, Swift native target, Python extension, Node addon, kernel/firmware, or standalone assembly.

## 41. Failure Types

```rust
pub enum AssemblyFailure {
    DetectionFailure,
    ToolchainFailure,
    PreprocessFailure,
    AssembleFailure,
    UnsupportedSyntax,
    UnsupportedTarget,
    AbiMismatch,
    ObjectFormatMismatch,
    CpuFeatureViolation,
    SymbolViolation,
    RelocationViolation,
    LayoutViolation,
    LinkFailure,
    HermeticityViolation,
    ReproducibilityFailure,
}
```

Examples:

```text
Assembly target mismatch
expected: x86_64 ELF
produced: i386 ELF
```

```text
Forbidden instruction detected
instruction: vaddps
required: AVX
release baseline: SSE2
```

```text
Firmware layout violation
section: .vectors
expected: 0x00000000
actual:   0x00000100
```

## 42. Production Defaults

```text
locked assembler
explicit syntax
explicit architecture
explicit ABI
explicit CPU baseline
explicit object format
network denied
isolated includes
explicit sysroot
deterministic archive/link policy
object verification enabled
reproduction for release-critical objects
```

Development may permit audited host assemblers with lower reproducibility status.

For crypto/kernel/firmware, enable stronger object metadata, symbol, layout, disassembly, independent reproduction, and device/emulator gates.

## 43. Threat Model

Key risks:

```text
wrong architecture
wrong ABI
host assembler drift
unexpected CPU instruction
hidden include
malicious build script
incorrect symbol export
layout corruption
nondeterministic linker metadata
```

## 44. Implementation Phases

### Phase 1 — Core Model
Implement AssemblerId, syntax, CPU architecture, ABI, object format, AssemblyAction.

### Phase 2 — GNU/LLVM
Implement GNU as and LLVM integrated assembler with ELF/COFF/Mach-O support.

### Phase 3 — NASM/MASM
Add NASM, MASM, and Windows integration.

### Phase 4 — Object Inspection
Implement ELF, PE/COFF, Mach-O parsing for symbols, sections, and relocations using permissively licensed Rust crates where appropriate.

### Phase 5 — ABI Verification
Add target/architecture, symbol, unwind, and layout policy checks.

### Phase 6 — Link Integration
Unify with Forgeyard native LinkPlan.

### Phase 7 — Rust/C++ Integration
Bridge Cargo build.rs, CMake/Meson, and native tool capability graphs.

### Phase 8 — Embedded/Kernel
Add raw binaries, linker-script validation, memory layout, QEMU/emulator/device tests.

### Phase 9 — Reproducibility
Add stable paths, preprocessed artifact capture, independent rebuilds, object diffs.

### Phase 10 — Advanced Security/Performance
Add instruction policy, SIMD validation, constant-time adapters, disassembly diffs, performance harnesses.

## 45. Acceptance Tests

1. Assembler version change changes derivation.
2. Syntax change changes derivation.
3. CPU feature change changes derivation.
4. ABI change changes derivation.
5. Include change changes derivation.
6. Preprocessor define change changes derivation.
7. Object format change changes derivation.
8. Locked assembler works without host assembler.
9. Undeclared host include fails strict build.
10. AVX emitted under SSE2 baseline fails policy.
11. Wrong object architecture fails verification.
12. Missing required symbol fails verification.
13. Wrong firmware section address fails layout check.
14. Rust build.rs invoking undeclared assembler fails.
15. CMake assembly action uses locked toolchain.
16. Independent rebuild produces identical object.
17. Reproducer mismatch yields object-level diagnostics.
18. Native linker consumes only verified object according to strict policy.

## 46. Architectural Invariants

1. Assembly is a first-class native subsystem, not a mainstream app ecosystem.
2. Assembler version string alone is not toolchain identity.
3. Syntax is explicit.
4. Architecture is explicit.
5. ABI is explicit.
6. CPU baseline/features are explicit.
7. Object format is explicit.
8. Preprocessor and include closure are explicit.
9. Host includes are denied in strict mode.
10. Object outputs are content-addressed.
11. Linker identity is distinct from assembler identity.
12. Linker scripts are immutable inputs.
13. Build/host/target are distinct.
14. Cross-assembly is explicit.
15. Rust/C/C++ integrations cannot invoke undeclared assemblers.
16. Inline assembly remains owned by its compiler ecosystem.
17. Native assembly objects receive object-format verification.
18. Portable releases cannot silently target host CPU features.
19. Reproducibility compares actual object bytes.
20. Disassembly is evidence, not artifact identity.
21. Embedded/kernel layouts are validated separately.
22. Signing/package operations never alter source assembly derivation.
23. C/C++ native subsystem remains the broader linker/sysroot authority.
24. Correctness and ABI safety take priority over optimization convenience.

## 47. Final Architecture

```text
Assembly Source
    ↓
Forgeyard ASM Detector
    ↓
AssemblyUnit
    ↓
Assembler + ABI + Target + CPU + Preprocessor + Sysroot
    ↓
AssemblyDerivation
    ↓
Hermetic Runner
    ↓
Assembler
    ↓
Object / Raw Binary
    ↓
Object Verification
    ├── Symbols
    ├── Relocations
    ├── ABI metadata
    └── Layout
    ↓
Object CAS
    ↓
Native LinkPlan
    ↓
Binary / Library / Firmware
    ↓
Independent Reproduction
```

## 48. Final Architectural Position

A Forgeyard Assembly derivation is:

```text
Source
+
Assembler
+
Syntax
+
Architecture
+
ABI/calling convention
+
CPU feature contract
+
Preprocessor
+
Include closure
+
Defines
+
Assembler flags
+
Object format
+
Target
+
Sysroot/platform contract
+
Hermetic environment
=
AssemblyDerivation
```

Then:

```text
AssemblyDerivation
  ↓
object/raw binary digest
  ↓
object metadata verification
  ↓
symbols / relocations / layout / ABI checks
  ↓
independent reproduction
  ↓
native linker
  ↓
final executable/library/firmware
```

This keeps Assembly exactly where it belongs in Forgeyard: a deep, explicit native capability used by higher-level ecosystems without duplicating the broader C/C++ ecosystem architecture or hiding low-level ABI/toolchain assumptions.
