# Development

Read [AGENTS.md](../AGENTS.md) and [architecture.md](architecture.md) first.
Linux is the verified development platform; Windows runtime validation remains
open. The manifests declare Rust 1.88 or newer and edition 2024.

## Build and checks

Install Rust/Cargo, a C/C++ toolchain, CMake and pkg-config. Linux also needs
ALSA, Wayland and XKB development libraries and a working graphics driver.
SJK builds no video codec: chat MP4 clips use Windows Media Foundation at run time
([sjk-mediafoundation](../crates/sjk-mediafoundation/src/lib.rs), the one crate
allowed `unsafe`), and WebP is decoded in Rust. No FFmpeg installation or runtime
codec download is required. Crates that omit license files receive their declared SPDX
license text and original author/license manifest; canonical texts are kept under
`scripts/licenses` (the MPL-2.0 wording comes from
[SPDX license-list-data](https://github.com/spdx/license-list-data/tree/main/text)).
Cargo metadata is read as UTF-8 on Windows as on Linux.
See [workspace dependencies](../Cargo.toml) and the
[viewer manifest](../crates/sjk-viewer/Cargo.toml) when diagnosing missing libraries.

From the repository root:

```sh
cargo fmt --all --check
cargo build --locked --workspace
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets
cargo build --locked --release -p sjk-viewer -p sjk-dedicated
```

The first formatting command requires the rustfmt component and the clippy
command the clippy component. Clippy fails only on the lints the workspace
manifest denies; its warnings do not fail the check. Use `-j2` to limit
build parallelism on constrained machines. `CARGO_TARGET_DIR` can put build
artifacts outside the checkout; do not commit binaries or generated output.

On Windows, Git's `core.autocrlf` checks text out with CRLF line endings.
[.gitattributes](../.gitattributes) keeps WGSL programs LF; a checkout made before
it existed keeps CRLF `.wgsl` files until they are checked out again, for example
by deleting them and running `git checkout -- '*.wgsl'`. The viewer accepts either.

SJK's Windows builds embed the programs' icon and version information with the
Windows SDK's resource compiler (`rc.exe`, installed with the MSVC build tools;
`windres` for the GNU toolchain), through the `winresource` build dependency.
Without one the build prints a warning and the programs have no icon.

The repository currently has no bundled regression suite. `cargo test` checks
its test/doc-test build targets but is not evidence of gameplay or wire parity.
Some source comments refer to external reference harnesses; those paths are not
available in this checkout. Do not silently skip required compatibility evidence
or claim those checks ran. Record a verification gap if the reference is unavailable.

### Continuous integration

[The CI workflow](../.github/workflows/ci.yml) runs on every pull request and
every push to `main`. It checks formatting on Linux, then runs the workspace
build, `cargo test` and clippy on Linux and Windows with the latest stable
Rust. It does not run the optimized build, check the declared minimum Rust
version or perform any of the evidence checks below, so a passing run does not
replace them.

## Verification appropriate to the change

| Change | Evidence required in addition to workspace checks |
| --- | --- |
| Movement, combat or animation | OpenJK `codemp` reference comparison; command steps 8/7/4/3 ms |
| Wire encode/decode | Captured or reference-emitted bytes, including affected edge cases |
| Rendering or shaders | Actual GPU startup and affected map/pass; compare images when appearance changes |
| Frame-loop performance | Release frame-time measurements with hardware, map, settings and workload |
| Server behavior | Isolated local server and relevant native/legacy client scenarios |
| Documentation | Check commands against source, relative links and the accuracy of status claims |

Use a server you control with no human players. Keep evidence under
`target/parity-reports/`, delete large scratch reports afterwards, and record a
compact account of the scenario, revision, expected result and observed result.
Never bundle retail assets in evidence or source commits.

## Change workflow

Check `git status` and existing work before editing. Follow the requested scope,
inspect the affected module and reference behavior, then implement and verify.
Update the relevant wiki pages in the same change. Record durable decisions and
limitations; use commit descriptions for the change narrative.

A handoff should state what changed, the exact checks and their results, and
what remains unverified. Do not label a feature complete merely because it builds.
Use [status.md](status.md) for current agreed priorities, without treating that
list as authorization to start unrelated work.

## Branches, commits and pull requests

Contributions reach `main` through pull requests, one topic per pull request.

- Start each change on its own branch from the current `main` of this repository,
  named for its topic with a type prefix: `fix/`, `feat/`, `perf/`, `refactor/`
  or `docs/` (for example `fix/windows-main-stack`). Do not commit to `main`; in a
  fork, keep `main` a fast-forward copy of this repository's `main`.
- Keep one topic per branch. Formatting sweeps, renames and unrelated fixes found
  along the way get their own branch and pull request, however small.
- Write commit subjects in the imperative mood, at most 72 characters, with no
  trailing period. Use the body for the reason and the verification account.
  Every commit should build; fold fixups into their commit before review.
- Before opening a pull request, rebase onto the current `main` and run the
  workspace checks above. Include the wiki updates the change requires.
- A change that adds something players see or use includes its SJK UI version
  ([the rule](sjk-ui.md#sjk-ui)); a classic+ view alone is not enough.
- The description states what changed and why, the exact checks and their
  results, the platform they ran on and what remains unverified, as in a handoff.
- A change players can see (menus, HUD, effects, models, maps) comes with
  screenshots in its pull request whenever you can take them (Sol's rule,
  18/10/2026): the new look, and before/after when something changed. A shot from
  the game or an off-screen world shot (`world_shot` tests) both do. Sol posts the
  good ones to the Discord `#screenshots` channel, so pick clear, full-frame shots
  with no personal details in them.
- Assistants push only to the contributor's fork or a branch they were
  authorized to use, never to `main` of this repository. Once review has started,
  add commits rather than rewriting the branch unless a reviewer asks for a rebase.
- Personal launchers, local paths and editor files stay out of commits and out of
  the shared `.gitignore`; put local ignores in `.git/info/exclude`.
