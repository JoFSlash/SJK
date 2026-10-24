# Status and priorities

The current scope, priorities, baseline verification and open limits. This page
is read at the start of every task, so it stays short: per-change verification
belongs in the change description and in the subsystem page that owns the
behavior, not here. Earlier per-change notes, up to the release 2026.1017.2,
are kept in the [status history](history/status-2026.md); it is reference
material, not required reading.

## Current priorities

1. Evaluate dark-area readability while preserving the accepted lighting style
   (owner priority, 2026-10-03). A local, default-preserving indirect gain and fill
   occlusion experiment passed workspace and external GPU checks; see
   [rendering](rendering.md#indirect-lighting-and-dark-area-readability). Owner
   acceptance and populated-match measurements are pending. The marked custom-map
   ceiling lights were recognized but underpowered; an explicit-glow inference
   refinement passed source-policy and GPU checks on that map plus two stock maps
   ([fixture inference](rendering.md#inferring-fixture-light-from-legacy-materials)).
   That refinement also awaits owner acceptance. Static model fixtures were also
   missing from source extraction: both `t2_rancor` marks now gain local illumination,
   with unchanged stock-map appearance in the sampled `ffa1`/`ffa3` views. The
   Rancor checks add approximately 0.017–0.144 ms of GPU work; see
   [static model fixtures](rendering.md#static-model-fixtures) for evidence and
   remaining visibility limits. Owner acceptance is pending. A subsequent GPU audit
   confirmed a sign error in GI voxel traversal. The corrected forward distances
   and range checks pass 8,302 external GPU/reference cases and workspace checks.
   Both Rancor captures remain byte-identical, so the correction has not solved
   their low visibility. Readback confirms nonzero live probe lighting; receiver
   coverage and effective bounce strength remain to investigate. See
   [GI traversal correction](rendering.md#gi-traversal-correction).
   Dust remains parked.
2. Improve populated-match release frame times while preserving the owner-accepted
   appearance (owner priority, 2026-10-02). Target below 2 ms with 31 players;
   reaching that target is not a reason to stop investigating useful savings.
3. Stabilize normal client and dedicated-server use with reproducible local reports.
4. Audit compatibility gaps by subsystem and scenario; preserve exact combat and wire behavior.
5. Broaden community-content and platform validation.

These priorities guide requested work; they do not authorize an assistant to
start an unrelated task. Update this page when evidence or agreed priorities
change. For a new result, record the source revision, environment, scenario,
reference and limits. Keep resolved change history in Git rather than appending
session-by-session notes here.

## Implemented scope

- PK3/loose-file content, BSP maps/collision, legacy models and shader scripts.
- Protocol-26 client/server sessions, shared JKA game rules and client prediction.
- Graphical client with browser, menus/settings, HUD, console, audio, screenshots,
  demos and a Create game flow.
- wgpu BSP renderer with optional modern lighting and post processing.
- Offline generator of local rend2-convention material maps and SJK emission maps
  (`sjk-materialgen`).
- Dedicated-server game integration, console/configuration, stock game-type
  options, map entities, bots/NPCs and script integration.

Entry points and ownership are linked from [architecture.md](architecture.md).
The lists above describe source coverage; they do not close the validation gaps below.

## Verification recorded for this baseline

| Check | Result and scope |
| --- | --- |
| Linux workspace build, Cargo tests and formatting | Passed; no bundled regression tests currently run |
| Optimized client and dedicated-server build | Passed with Rust 1.96.1 on Linux |
| External OpenJK movement reference checks | 560 cases / 72,275 commands at 8/7/4/3 ms; movement, animation, events and compared wire fields matched |
| External player-angle checks | 125 samples matched the OpenJK reference |
| Local OpenJK client against the dedicated server | Joined `mp/ffa3`, walked, jumped and turned; no prediction misses observed in that run |
| Native Vulkan rendering | The local client and dedicated server completed joined-map rendering on `mp/ffa3` with defaults and with day/night + lighting tier 2 + HDR; each continued for 15 seconds without panic or GPU validation error |

The reference checkout used for the external checks was OpenJK
`1a6a643427aa347553e9073dac5570b33337c4d9`, multiplayer `codemp`.
The external harnesses and raw reports are not part of this repository, so these
are recorded maintainer results, not checks reproducible by running `cargo test`
alone. The GPU runs used debug builds and establish startup/integration only;
they do not establish visual parity or release performance.

Dated verification notes recorded after this baseline are in the
[status history](history/status-2026.md#baseline-verification-notes).

## Open validation and limitations

Volumetric silhouette correction (local changes based on `a993436`): excluded
depth samples no longer dilute the visible-air lighting estimate. External
Linux/RADV release checks reproduced and removed the sampled FFA5 player fringe
without increasing grid resolution. Paired 720p/4K timings showed no material
change in the tested scene; captures also cover FFA3, a Rancor interior and 24
camera turns. See [volumetric coverage](rendering.md#volumetric-silhouette-coverage)
for measurements and reproduction limits. Formatting, workspace build/tests and
the production release build passed. Owner acceptance and broad content coverage
remain open.

- Complete server/gameplay parity remains unverified. Audit concrete scenarios
  across game types, combat, vehicles, NPCs, scripting and map transitions before
  marking individual capabilities complete.
- Model animation sounds now follow `animevents.cfg` frames. Local release checks
  against OpenJK `3e465e7c`'s extracted `CG_PlayerAnimEvents` predicate matched
  238,328 frame-crossing cases. Walk/run and blue-style gesture cues were stable
  at 8/7/4/3 ms steps; a six-second gesture produced its ten authored spin cues
  at every cap, and held frames did not replay them. Include overrides, material
  selection and missing voice-family fallbacks passed external checks. A
  32-actor cursor-only microbenchmark averaged 0.44 microseconds per iteration
  with zero measured heap allocations; this excludes bone queries, collision
  traces, mixing and rendering.
  An isolated loopback TaystJK `5802c99` run produced stone/metal running steps
  and blue-taunt spin cues through a real decoder and null-output mixer, with
  zero decode failures or missing handles. Authored custom saber sound fields
  also passed an external parsing check. Formatting, locked workspace build/tests
  and the optimized Linux client build passed. Native owner listening, broader
  custom-model coverage remain open; footstep dust and prints followed on 13/10/2026.
- Snapshot entities draw a model from `modelindex` only for the entity types
  whose codemp cgame function does so; the per-type rules and their reference
  are in [entity_models.rs](../crates/sjk-client/src/entity_models.rs). An
  `ET_GENERAL` model takes the server's `iModelScale` (a percentage) as
  `CG_General` does, so a prop a server spawns with `modelscale` has its size
  (SJK, `presentation.rs`). A mover's secondary `modelindex2` model is drawn since
  13/10/2026 ([client.md](client.md#mover-models)). Models codemp draws that the client
  still does not: force holocrons, non-brush `ET_MOVER` models, and the portable shield
  (`ET_SPECIAL`) and `ET_BEAM` effects.
- Mod compatibility is scoped by explicit profiles; broad BaseJKA/JA+/TaystJK
  feature parity is not established by profile detection.
- Community PK3 compatibility needs broader map/model coverage. One retail map
  cannot establish every shader, animation or content combination.
- Windows runtime behavior is largely unverified. Do not infer platform support
  from source conditionals alone. Windows reserves 1 MiB for the main thread and
  the client overflowed it after loading `mp/ffa3`; [the viewer build
  script](../crates/sjk-viewer/build.rs) now links Windows binaries with the
  8 MiB Linux size. On Windows 11 (Rust 1.96, MSVC), release and debug clients
  then loaded `mp/ffa3`, and a release client joined a local dedicated server and
  completed its map load. Longer play, other maps and the GNU toolchain are unchecked.
- DX12 rendering is unverified. The client does not select DX12 itself (Vulkan is preferred
  where present); with `WGPU_BACKEND=dx12` and no `dxcompiler.dll`, wgpu compiles
  shaders with FXC. A headless pipeline build of every entry point of the 39 viewer
  shader modules on DX12/FXC (Windows 11, RTX 5080, 2026-10-02) fails only for the
  HUD fragment program (error X3507: the function ends in `discard` without a
  return) and the GI probe update (error X4026: a workgroup barrier after
  storage-dependent early returns). The stage programs' skinning loop no longer
  fails with X3511.
- The 500+ FPS / roughly 2 ms frame target is not certified. Measure representative
  release workloads, including populated matches and chosen graphics settings.
- SJK material-map work (2026-10-04, Windows 11, change based on `024c22a`): the
  lamp/bounce direction target, reflection probes, material-mapped floor mirrors,
  `r_normalMapStrength` and materialgen generation 2 pass the workspace build and
  tests, and naga validates every new or changed program; a dry run of the generator
  on `mp/ffa3`/`mp/duel1` picked 19 metal textures and one polished texture. None
  of it has run on a GPU: appearance, wgpu validation at run time, capture load time
  and frame cost in a match are unverified (estimates are in
  [rendering](rendering.md#reflection-probes)).
- The repository does not bundle a regression suite. Required reference evidence
  must be supplied externally until an in-repository verification approach is agreed.

Sky/terrain correction (2026-10-03, local preview based on `8f692ac`): the three
`t1_danger` marks exposed sky draw-buffer reuse between views and smoothed normals
flipping across visible hills. Both terrain bands are removed in fixed GPU views;
a 24-turn capture reproduced sky disappearance in 12 old-reset frames and verified
the corrected path against direct draws. Workspace checks and owner release build
passed. Native owner playtesting remains pending; see
[rendering](rendering.md#sky-scenery-and-hillside-orientation) for scope and timings.

## Current transition policy

Local gameplay continuation during match-end intermission and server map changes
is suspended at the owner's request. Intermission uses the real server's camera,
scores, chat and ready controls; map changes show a loading notice with gameplay
paused. The native continuation adapter and the viewer's dedicated-server
dependency have been removed. Background loading, shared GPU context, archive
inventory optimization, gate-world adoption and matching same-map reuse remain.
Earlier continuation results below describe the historical implementation, not
current enabled behavior. Fast joining and map loading are the current priority;
no universal loading-time target has been verified.

The verification of this policy and of the loading work is in the
[status history](history/status-2026.md#current-transition-policy-verification).
