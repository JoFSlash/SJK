//! Build settings of the client: the build's version, commit and commit time
//! (`scripts/build_version.rs`, shared with the dedicated server), and on
//! Windows the main thread's stack and the program's icon and version strings.
//!
//! Windows reserves 1 MiB for a program's main thread where Linux reserves
//! 8 MiB; the client overflows 1 MiB while loading a map, so Windows binaries
//! are linked with the Linux size. `sjk.exe` carries SJK's icon
//! (`assets/branding/sjk.ico`, which Explorer and shortcuts show) and names
//! itself "SJK" in its version strings, which Task Manager shows. Other
//! platforms have no executable icons.
//!
//! On Windows `sjk.exe` also exports NVIDIA's and AMD's hints to run on the fast
//! graphics card of a laptop that has two (`src/gpu_hints.c`, compiled here: the
//! client's Rust forbids the unmangled statics they need). `/INCLUDE` keeps the
//! otherwise unreferenced object in the link.

#[path = "../../scripts/build_version.rs"]
mod build_version;

const MAIN_STACK_BYTES: u32 = 8 * 1024 * 1024;
/// SJK's icon set, relative to this crate.
const ICON: &str = "../../assets/branding/sjk.ico";
/// The fast-card hints' source, relative to this crate, and the symbols it exports.
const GPU_HINTS: &str = "src/gpu_hints.c";
const GPU_HINT_SYMBOLS: [&str; 2] = [
    "NvOptimusEnablement",
    "AmdPowerXpressRequestHighPerformance",
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={ICON}");
    println!("cargo:rerun-if-changed=../../scripts/build_version.rs");
    println!("cargo:rerun-if-changed={GPU_HINTS}");
    let version = build_version::emit();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    match target_env.as_str() {
        "msvc" => println!("cargo:rustc-link-arg-bins=/STACK:{MAIN_STACK_BYTES}"),
        "gnu" => println!("cargo:rustc-link-arg-bins=-Wl,--stack,{MAIN_STACK_BYTES}"),
        _ => {}
    }
    cc::Build::new().file(GPU_HINTS).compile("sjk_gpu_hints");
    for symbol in GPU_HINT_SYMBOLS {
        match target_env.as_str() {
            "msvc" => println!("cargo:rustc-link-arg-bins=/INCLUDE:{symbol}"),
            "gnu" => println!("cargo:rustc-link-arg-bins=-Wl,--undefined={symbol}"),
            _ => {}
        }
    }
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon(ICON)
        .set("ProductName", "SJK")
        .set("FileDescription", "SJK")
        .set("ProductVersion", &version)
        .set("OriginalFilename", "sjk.exe");
    // A missing resource compiler costs the icon, not the build.
    if let Err(error) = resource.compile() {
        println!("cargo:warning=sjk is built without its icon: {error}");
    }
}
