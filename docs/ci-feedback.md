# CI feedback tiers and measurement

ASHA has one gate inventory with two selection tiers. Normal agent iteration uses:

```sh
./harness/ci/check-fast.sh
```

Campaign or release closure uses:

```sh
./harness/ci/check-all.sh
```

The fast selector keeps authority/dependency, no-Den-coupling, and vocabulary
rails on every change. It adds affected Rust, TypeScript, generated-border,
bridge, identity/conformance, replay, render, or native gates. Unknown and
cross-cutting paths expand to the full inventory. The full inventory includes
the native addon and browser-host gate.

These tiers schedule existing checks; they do not define a second proof system.
The selector is a small path classifier in `harness/ci/ci.py`, and its unit tests
cover representative classes, safe expansion, and injected failures.

## Reports

Each run writes a computed report under the ignored
`harness/smoke-out/ci/` directory. GitHub uploads that directory and the proof
execution receipts as run artifacts. The report records:

- the changed files and selected change classes;
- each gate's normalized command fingerprint and semantic claim consumers;
- per-gate and total wall time;
- unique and repeated gate commands; and
- proof-execution requests, actual executions, receipt reuses, and any duplicate
  actual fingerprint.

A run is invalid if one normalized command/input/toolchain fingerprint executes
more than once. Different suites may still consume the same receipt and retain
their own probe/assertion attribution.

## Baseline

The last successful pre-change GitHub job on 2026-07-16 was run
`29472070788` at commit `4fc7d5a810e102172fda87b32007b6259255fe8d`.
Its single `Verify ASHA` job ran from 04:43:57Z to 04:57:43Z: 826 seconds,
or 13.77 runner minutes. That workflow did not emit per-gate timing or execution
reuse data, which is itself part of the feedback problem.

A same-host warm local inventory sampled immediately before the scheduling
change established these available per-gate timings. Fingerprints are the
normalized-command identities now emitted by the shared inventory; commands
were unchanged for these sampled gates.

| Gate | Baseline seconds | Normalized command fingerprint | Claim consumer |
| --- | ---: | --- | --- |
| Rust | 40.450 | `sha256:00b2b047c1d7c5d49cbb8eb7ca55e95ead4acacfc16be181e9d4a45b4cdb59a3` | Rust format, compile, clippy, workspace tests |
| TypeScript | 46.158 | `sha256:eef77839702850a426c19d6aeafc560a88037ee86a17f9e2cc5efd3bda7c85ea` | build, typecheck, tests, lint, package boundaries |
| Contracts | 1.567 | `sha256:6947c574b4c107510b69a6cdfe852e020044d0d64c7fef4e9fd20fc8a3318cdc` | generated Rust-to-TypeScript border parity |
| Dependency/authority rails | 3.916 | `sha256:a02b875284b2c916753ee3448b9161b043b8b6dab5a2a7f59134d23f644294b7` | lanes, edges, source shape, committed paths |
| No Den coupling | 0.022 | `sha256:d39b0bba572a72dc8e8bb09aabd4cc1b92226bd27453955537048631f4cace12` | engine independence from Den runtime code |
| Vocabulary | 2.654 | `sha256:d31adccb18c335ff03ce94a5534c5aec46c998160fa6386c511ce4c9b7d9ddb4` | ECRP vocabulary and Rust authority naming |
| Harness identities | 1.618 | `sha256:8c6fb2bfb4baf863cd160eb95b3f90fef887e11c2fa9c8b9d205a16f46248f19` | stable public/provider/execution/evidence IDs |
| Consumer needs | 0.202 | `sha256:3469772132d9fbd1c53622bd2e9da7cdd586b61ec26231e1cdb4f9c908d384fc` | declared requirements and semantic delivery |
| Reachability | 0.273 | `sha256:b3ed96af15be212c647020ac3c28a3defa76bada8500399f216c14c6a9e37848` | joined public capability reachability |

The old conformance pass was stopped after 76 seconds while waiting on broad
live/product execution, so it is not presented as a successful timing. The old
workflow supplied no trustworthy unique-versus-total command count.

## Warm changed-surface measurements

Same-host measurements after the change were:

| Representative change | Selected gates | Wall time |
| --- | ---: | ---: |
| Documentation only | 3 | 3.563 s |
| Rust only, first warm pass | 4 | 25.353 s |
| Rust only, receipt-reuse pass | 4 | 4.725 s |
| TypeScript only | 4 | 27.271 s |

All are below the three-minute warm optimization budget. Cold GitHub timing is
reported separately because dependency restore, toolchain installation, and
first compilation are runner concerns rather than architecture requirements.

## Retained blocking safeguards

Fast selection remains fail-closed for the surfaces that can invalidate an
iteration:

- authority/dependency and generated-file governance always run;
- affected Rust and TypeScript compilation/tests run;
- protocol edits add generated-contract and strict bridge checks;
- native, replay, and render edits add their owning semantic gates;
- identity/conformance self-tests run when harness-policy surfaces change; and
- an unclassified path expands to the comprehensive inventory.

Broad harness negative fixtures are skipped for unrelated product edits, while
their actual dependency and authority guards still run. Product browser
acceptance remains owned by the consumer instead of running for every unrelated
engine change.

## Execution reuse and semantic isolation

The Rust workspace, TypeScript workspace, native-bridge library, and downstream
gameplay fixture now have one execution identity each instead of package-level
duplicates. Conformance and dedicated public gates consume the same receipts
while retaining separate claim attribution. Redundant direct reruns were
deleted from the gameplay conformance gate rather than cached behind another
layer.

The installed-addon runtime-bridge and browser-host suites have their own native
execution identities and reusable logs/receipts. Their post-install executions
remain separate from the earlier TypeScript workspace pass because installing
the compiled addon changes test semantics: native parity runs instead of being
absent or skipped. The composed provider release build also remains isolated
because it proves a downstream-shaped release artifact, not the workspace test
binary.

The final same-host warm comprehensive run selected all 16 gates, including
native acceptance, and completed in 99.618 seconds (1.660 runner minutes). Its
gate layer contained 16 unique commands and zero repeats. The proof scheduler
observed 14 requests for 11 unique fingerprints: 10 actual executions and four
receipt reuses, with three repeated requests and zero duplicate actual
fingerprints. The native gate accounted for 4.360 seconds and its Rust library
test reused the conformance receipt; the two post-addon TypeScript suites each
produced one reusable native-semantic receipt.

The first cold GitHub comparison used implementation run `29496495981` at
commit `6af53b04635ebeb977b1699b83648568568d2b53`. Because the change modified
the selector and execution harness themselves, the fast job correctly expanded
to all 16 gates. The measured command inventory took 769.986 seconds (12.833
runner minutes); the complete GitHub job, including checkout/toolchain/cache
setup and teardown, took 857 seconds (14.283 runner minutes). It recorded 16
unique gate commands, zero gate repeats, 13 proof requests for 10 unique
fingerprints, four receipt reuses, and zero duplicate actual fingerprints.

The equivalent pre-change all-gates job took 826 seconds (13.767 runner
minutes), so this worst-case cold run was 31 seconds, or 3.8%, slower. This is
reported deliberately rather than presented as a speedup: the material gain is
that ordinary narrow changes no longer pay for the all-gates/native path, while
cross-cutting CI changes retain the complete fail-safe path. The same-host warm
measurements above demonstrate the execution-reuse improvement independently
of cold runner setup and compilation.
