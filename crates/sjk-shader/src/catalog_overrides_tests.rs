//! Synthetic JoF EJK #138 override scripts, without retail data or GPU requirements.
use super::*;

fn script(image: &str) -> Vec<u8> {
    format!("textures/test/wall\n{{\n {{\n map {image}\n }}\n}}\n").into_bytes()
}
fn image(catalog: &ShaderCatalog) -> &str {
    &catalog.get("textures/test/wall").unwrap().stage_images[0]
}

#[test]
fn overrides_win_independent_of_normal_filename_and_work_without_normal_scripts() {
    let mut vfs = VirtualFileSystem::new();
    vfs.mount_memory(
        "synthetic",
        [
            ("shaders/a.oshader", script("override")),
            ("shaders/z.shader", script("normal")),
            ("elsewhere/hidden.oshader", script("ignored")),
        ],
    )
    .unwrap();
    assert_eq!(image(&ShaderCatalog::load(&vfs).unwrap()), "override");
    assert_eq!(
        image(&ShaderCatalog::load_with_warnings(&vfs, |_, error| panic!(
            "{error}"
        ))),
        "override"
    );
    let mut only = VirtualFileSystem::new();
    only.mount_memory("synthetic", [("shaders/only.oshader", script("only"))])
        .unwrap();
    assert_eq!(image(&ShaderCatalog::load(&only).unwrap()), "only");
}

#[test]
fn first_override_path_and_first_definition_win_but_normal_order_is_unchanged() {
    let mut vfs = VirtualFileSystem::new();
    let mut first = script("first");
    first.extend(script("second"));
    vfs.mount_memory(
        "synthetic",
        [
            ("shaders/a.oshader", first),
            ("shaders/z.oshader", script("last")),
            ("shaders/a.shader", script("old")),
            ("shaders/z.shader", script("new")),
        ],
    )
    .unwrap();
    assert_eq!(image(&ShaderCatalog::load(&vfs).unwrap()), "first");
    let mut normal = VirtualFileSystem::new();
    normal
        .mount_memory(
            "synthetic",
            [
                ("shaders/a.shader", script("old")),
                ("shaders/z.shader", script("new")),
            ],
        )
        .unwrap();
    assert_eq!(image(&ShaderCatalog::load(&normal).unwrap()), "new");
}

#[test]
fn source_authority_still_protects_definitions_and_downloads_only_add() {
    let mut vfs = VirtualFileSystem::new();
    let trusted = vfs
        .mount_memory("trusted", [("shaders/retail.shader", script("trusted"))])
        .unwrap();
    vfs.mount_memory(
        "installed",
        [("shaders/custom.oshader", script("installed"))],
    )
    .unwrap();
    let protected = ShaderCatalog::load_with_authority(
        &vfs,
        |source| source.mount_id == trusted,
        |_, error| panic!("{error}"),
        |_, _, _| {},
    );
    assert_eq!(image(&protected), "trusted");
    let mut installed = ShaderCatalog::load(&vfs).unwrap();
    assert_eq!(image(&installed), "installed");
    let mut extra = script("download");
    extra.extend(b"textures/test/new\n{\n}\n");
    let downloaded = vfs
        .mount_memory("download", [("shaders/download.oshader", extra)])
        .unwrap();
    installed.add_unclaimed(
        &vfs,
        |source| source.mount_id == downloaded,
        |_| false,
        |_, error| panic!("{error}"),
        |_, _| {},
    );
    assert_eq!(image(&installed), "installed");
    assert!(installed.get("textures/test/new").is_some());
}

#[test]
fn malformed_override_does_not_discard_valid_normal_definitions() {
    let mut vfs = VirtualFileSystem::new();
    vfs.mount_memory(
        "synthetic",
        [
            ("shaders/good.shader", script("normal")),
            ("shaders/bad.oshader", b"broken\n{\n{\nmap nope\n".to_vec()),
        ],
    )
    .unwrap();
    let mut warnings = 0;
    let catalog = ShaderCatalog::load_with_warnings(&vfs, |_, _| warnings += 1);
    assert!(warnings > 0);
    assert_eq!(image(&catalog), "normal");
    assert!(ShaderCatalog::load(&vfs).is_err());
}

#[test]
fn a_normal_script_naming_a_shader_twice_keeps_the_first() {
    // `shaders/oot.shader` (maps/oot.bsp) names textures/oot/wtr_0_yuka_02_4 twice.
    let mut twice = script("first");
    twice.extend(script("second"));
    let mut vfs = VirtualFileSystem::new();
    let mount = vfs
        .mount_memory("synthetic", [("shaders/a.shader", twice)])
        .unwrap();
    assert_eq!(image(&ShaderCatalog::load(&vfs).unwrap()), "first");
    assert_eq!(
        image(&ShaderCatalog::load_with_warnings(&vfs, |_, error| panic!(
            "{error}"
        ))),
        "first"
    );
    for trusted in [true, false] {
        let catalog = ShaderCatalog::load_with_authority(
            &vfs,
            |source| trusted && source.mount_id == mount,
            |_, error| panic!("{error}"),
            |_, _, _| {},
        );
        assert_eq!(image(&catalog), "first", "authoritative {trusted}");
    }
    let mut added = ShaderCatalog::default();
    added.add_unclaimed(
        &vfs,
        |source| source.mount_id == mount,
        |_| false,
        |_, error| panic!("{error}"),
        |_, _| {},
    );
    assert_eq!(image(&added), "first");
}
