//! Checks against JoF's real repository; they need the network, so they are
//! ignored by default: `cargo test -p sjk-jof-content --test live -- --ignored`.
//! `JKA_GAME_DATA` (the `GameData` folder) adds a look at the game's `base`.

use sjk_jof_content::{AssetState, Target, download, source, store::Store};
use std::sync::atomic::AtomicBool;

#[test]
#[ignore = "needs the network"]
fn both_hosts_give_the_manifest_and_a_file_with_an_awkward_name() {
    let agent = download::agent("test");
    for host in source::SOURCES {
        let (manifest, from) = download::manifest(&agent, &[*host]).unwrap();
        assert_eq!(from.name, host.name);
        assert!(!manifest.groups.is_empty(), "{}: no groups", host.name);
        // The smallest file whose name needs encoding (braces, quotes, spaces).
        let smallest = manifest
            .assets()
            .map(|(_, asset)| asset)
            .filter(|asset| asset.target == Target::Base)
            .filter(|asset| asset.name.contains(['{', '\'', ' ', '&', '(']))
            .min_by_key(|asset| asset.size)
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = download::file(
            &agent,
            &[*host],
            smallest,
            directory.path(),
            &AtomicBool::new(false),
            &mut |_| {},
        )
        .unwrap_or_else(|error| panic!("{}: {}: {error}", host.name, smallest.name));
        println!(
            "{}: {} groups, {} files skipped; fetched {} ({} bytes) to {}",
            host.name,
            manifest.groups.len(),
            manifest.skipped,
            smallest.name,
            smallest.size,
            path.display()
        );
    }
}

#[test]
#[ignore = "needs the network and JKA_GAME_DATA"]
fn the_game_folder_files_are_recognised() {
    let Some(game_data) = std::env::var_os("JKA_GAME_DATA") else {
        return;
    };
    let agent = download::agent("test");
    let (manifest, _) = download::manifest(&agent, source::SOURCES).unwrap();
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(
        directory.path(),
        Some(std::path::Path::new(&game_data).join("base")),
    );
    let snapshot = store.scan(manifest, &AtomicBool::new(false), &mut |_, _| {});
    for (index, group) in snapshot.manifest.groups.iter().enumerate() {
        let states: Vec<AssetState> = (0..group.assets.len())
            .map(|asset| snapshot.state(index, asset))
            .collect();
        let here = states
            .iter()
            .filter(|state| **state == AssetState::InGameFolder)
            .count();
        println!(
            "{} ({}): {here} of {} in the game folder",
            group.name,
            group.kind.label(),
            group.assets.len()
        );
    }
}
