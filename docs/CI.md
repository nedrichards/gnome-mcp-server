# Continuous integration

Actions use immutable action revisions, read-only repository permissions by
default, bounded jobs, cancellation of obsolete development runs, and grouped
Dependabot action updates. Workflow syntax and shell snippets are checked with
checksum-pinned actionlint on native x86_64 and aarch64 runners. Dependency repair
jobs retain their existing narrowly scoped write access and trusted generators.

Tested development artifacts are for review, with source provenance and checksums;
diagnostics are retained even when tests fail. Reports and captures must use
fixtures, never account credentials, personal data or hardware identifiers.
Where a release workflow is configured, it attests verified files and prepares
drafts for manual publication. Native Flatpak release packaging requires both targets. CI signing establishes
build provenance; it does not replace release signing or human acceptance.

Stable Rust checks, locked builds and tests run on native x86_64 and aarch64. Weekly/manual beta compatibility repeats the suite. A private D-Bus session exercises the built binary through MCP initialization, tool listing and resource listing. Tested native tarballs include source/toolchain provenance and checksums. No desktop or keyring mutation is exercised.

Workflow syntax and shell snippets are checked locally before publication. Hosted
execution provides the architecture and installed-artifact evidence; generated
captures still require human visual review. No release or tag is created by this
maintenance change.


## Upstream refresh validation (15 November 2026)

The GNOME 51 SDK with Rust 1.98 passed 54 unit tests and the strict stdio
integration test. Tests cover the Mutter wire signature, an isolated DisplayConfig
peer, workspace listing through an isolated Shell peer, monitor configuration and
serial omission, and portal cancellation. Clippy passed all targets/features with
warnings denied. Cargo Deny passed advisories, bans, licenses and sources with no
advisory exceptions. The production executable is built with the locked graph.

A read-only invocation of the new monitor tool against the actual Mutter service
also passed. No real keyring operation, screenshot, input injection or desktop
setting mutation was performed. The Shell workspace test uses a fixture peer;
this does not establish live Shell Eval permission or human desktop acceptance.
Hosted native x86_64/aarch64 results are recorded after publication.
