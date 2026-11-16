#!/usr/bin/env python3
"""Package a clean, already-built SJK revision into a drop-in GameData ZIP."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import tomllib
import zipfile

# Where the source of a package is published.
DEFAULT_REPOSITORY = "https://github.com/Sol-Vulpes/SJK"


def bin_name(source, crate):
    """The [[bin]] name declared by crates/<crate>/Cargo.toml, else the crate name."""
    manifest = tomllib.loads((source / "crates" / crate / "Cargo.toml").read_text(encoding="utf-8"))
    bins = manifest.get("bin") or []
    return bins[0]["name"] if bins else crate


def command(source, *args):
    return subprocess.check_output(args, cwd=source, text=True, encoding="utf-8").strip()


def text_name(stem, name):
    """README-SJK.txt and LICENSES-SJK.txt, so they do not replace another program's
    README.txt or LICENSES.txt in the shared GameData folder."""
    return f"{stem}-{name}.txt"


def add_file(archive, source, name, executable=False):
    info = zipfile.ZipInfo(name)
    info.create_system = 3
    info.external_attr = (0o100755 if executable else 0o100644) << 16
    info.compress_type = zipfile.ZIP_DEFLATED
    archive.writestr(info, source.read_bytes())


def icns(source):
    """A macOS icon file holding the emblem at 512 and 1024 pixels (PNG payloads,
    which macOS 10.7+ reads; ic09 and ic10 are their sizes' type codes)."""
    branding = source / "assets/branding"
    entries = b"".join(kind + struct.pack(">I", 8 + len(data)) + data
                       for kind, data in ((b"ic09", (branding / "sjk-logo-512.png").read_bytes()),
                                          (b"ic10", (branding / "sjk-logo.png").read_bytes())))
    return b"icns" + struct.pack(">I", 8 + len(entries)) + entries


def info_plist(name, client, version):
    """The bundle's Info.plist: Finder starts the client itself (no Terminal) and
    the Dock shows SJK's name and icon."""
    version = version or "0.0.0"
    return f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key><string>en</string>
  <key>CFBundleDisplayName</key><string>{name}</string>
  <key>CFBundleExecutable</key><string>{client}</string>
  <key>CFBundleIconFile</key><string>{name}</string>
  <key>CFBundleIdentifier</key><string>io.github.sol-vulpes.{name.lower()}</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>CFBundleName</key><string>{name}</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>{version}</string>
  <key>CFBundleVersion</key><string>{version}</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.action-games</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
"""


def app_bundle(directory, binary, source, name, client, version):
    """Build `<name>.app` in `directory` around the client binary and return its
    files as (zip name, path, executable). On a Mac the bundle is ad-hoc signed as a
    whole, so its signature covers Info.plist and the icon."""
    bundle = directory / f"{name}.app"
    (bundle / "Contents/MacOS").mkdir(parents=True)
    (bundle / "Contents/Resources").mkdir()
    shutil.copy2(binary, bundle / "Contents/MacOS" / client)
    (bundle / "Contents/Info.plist").write_text(info_plist(name, client, version), encoding="utf-8")
    (bundle / "Contents/Resources" / f"{name}.icns").write_bytes(icns(source))
    if sys.platform == "darwin":
        subprocess.run(["codesign", "--force", "--sign", "-", str(bundle)], check=True)
    program = bundle / "Contents/MacOS" / client
    return [(path.relative_to(directory).as_posix(), path, path == program)
            for path in sorted(bundle.rglob("*")) if path.is_file()]


# A crate that declares one of these licenses without shipping its text gets the
# canonical text from scripts/licenses/<selected>.txt.
CANONICAL_TEXTS = {
    "Apache-2.0": "Apache-2.0",
    "MIT OR Apache-2.0": "Apache-2.0",
    # Older Cargo declarations use a slash for the same dual-license choice.
    "Apache-2.0/MIT": "Apache-2.0",
    "MIT/Apache-2.0": "Apache-2.0",
    "MPL-2.0": "MPL-2.0",
    # The macOS bindings (objc2, block2, dispatch...) ship no license file.
    "MIT": "MIT",
    "Zlib OR Apache-2.0 OR MIT": "Apache-2.0",
    "BSL-1.0": "BSL-1.0",
}


def dependency_notices(source, target, name="SJK"):
    """Yield original dependency license texts and their attribution inventory."""
    metadata = json.loads(command(source, "cargo", "metadata", "--locked",
                                  "--format-version", "1", "--filter-platform", target))
    resolved = {node["id"] for node in metadata["resolve"]["nodes"]}
    notices = []
    for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
        if package["source"] is None or package["id"] not in resolved:
            continue
        root = Path(package["manifest_path"]).parent
        files = [p for p in root.rglob("*") if p.is_file()
                 and p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE"))]
        if package.get("license_file"):
            files.append(root / package["license_file"])
        prefix = f'{name}-licenses/{package["name"]}-{package["version"]}'
        for path in sorted(set(files)):
            yield f"{prefix}/{path.relative_to(root).as_posix()}", path.read_bytes()
        selected_license = None
        if not files:
            # Some crates declare a license in Cargo.toml but omit its
            # text from their crate. Select that offered license and include the
            # canonical text plus the original package manifest (authors and license
            # declaration).
            selected_license = CANONICAL_TEXTS.get(package["license"])
            if selected_license is None:
                raise RuntimeError(f"No supplied license text for {prefix}: {package['license']}")
            yield (f"{prefix}/LICENSE-{selected_license.upper()}.txt",
                   (Path(__file__).parent / f"licenses/{selected_license}.txt").read_bytes())
            yield f"{prefix}/Cargo.toml", Path(package["manifest_path"]).read_bytes()
        notices.append({"name": package["name"], "version": package["version"],
                        "license": package["license"], "authors": package["authors"],
                        "repository": package["repository"],
                        "selected_license": selected_license,
                        "license_files": len(set(files)) or 1})
    yield "Dependency inventory", (json.dumps(notices, indent=2) + "\n").encode("utf-8")


def license_text(entries, name="SJK"):
    """Consolidate notices without altering or dropping their original bytes."""
    parts = [f"{name} - licenses and third-party notices\n".encode("utf-8"),
             b"Each section identifies its original source file.\n"]
    for label, content in entries:
        parts.extend([f"\n{'=' * 72}\n{label}\n{'=' * 72}\n".encode("utf-8"),
                      content, b"\n"])
    return b"".join(parts)


def instructions(platform, revision, name="SJK", repository=DEFAULT_REPOSITORY, version=None,
                 profile="SJK", client="sjk", server="sjk-server"):
    suffix = ".exe" if platform == "windows-x64" else ""
    # On a Mac the client is the app bundle, started from Finder.
    launch = f"{name}.app" if platform == "macos-arm64" else client + suffix
    requirements = {
        "windows-x64": "Windows 10/11 x64 and a working graphics driver. The MSVC runtime is statically linked.",
        "linux-x64": "Linux x64 with glibc 2.35+, ALSA, Wayland/X11 libraries and a Vulkan or OpenGL driver.\n"
                     f"If your archive extractor drops permissions: chmod +x {client} {server}",
        # Not signed with an Apple Developer ID: Gatekeeper blocks the programs until
        # the player allows them once.
        # "Open Anyway" alone would leave the quarantine flag, and macOS would run the
        # app from a random read-only copy, away from GameData.
        "macos-arm64": "macOS 11+ on Apple Silicon (M1 or later); the game draws through Metal.\n"
                       "The programs are not signed by Apple, so macOS blocks them at first. In Terminal,\n"
                       "in the folder you extracted them to, run once:\n"
                       f"  xattr -dr com.apple.quarantine {launch} {server}\n"
                       f"  chmod +x {launch}/Contents/MacOS/{client} {server}\n"
                       f"Then open {launch} from Finder (it can be kept in the Dock).\n"
                       "SJK also looks inside the Mac edition's app bundle in Steam's library. If it is\n"
                       "not found, extract into a GameData folder holding base/ copied from a Windows\n"
                       "installation (assets0.pk3 through assets3.pk3).\n"
                       "macOS builds are experimental: they have not been played on a Mac yet.",
    }[platform]
    label = f"{name} {version}" if version else f"{name} playtest build"
    return f"""{label} - {platform}
Source revision: {revision}

INSTALL
Extract ALL files from this ZIP directly into Jedi Academy's GameData folder,
beside its existing base folder. Do not put them inside base or an extra {name} folder.
The installed game must include base/assets0.pk3 through base/assets3.pk3.
Launch {launch}. No game-data path or environment variable is needed.
Keep {server}{suffix} beside it for Create game and local devmap.
An existing shortcut must point to this client, not an older named playtest binary.

YOUR FILES
Settings, marks, screenshots, demos, favorites and friends live in GameData/{profile}/.
Unwritable installations use the per-user profile instead.
The console command path shows the selected folder. Downloaded PK3s retain their
separate per-user cache. This archive contains no game assets or personal settings.
Close {name} before replacing its executables; retain your {profile} folder when updating.

REQUIREMENTS
{requirements}
This is a playtest build. Startup checks do not establish full GPU/gameplay
compatibility on every system. Windows graphical runtime testing is still pending.

SOURCE AND LICENSES
{repository}/tree/{revision}
GPL-2.0-only: see {text_name("LICENSES", name)}, including all bundled dependency notices.
The matching source snapshot is distributed separately as {name}-{version or revision[:7]}-source.zip.
Build metadata and SHA-256 checksums are distributed separately from this install ZIP.
"""


def smoke_check(package, platform, source, packaged, profile="SJK", client="sjk",
                server="sjk-server", name="SJK"):
    """`packaged` lists the ZIP's names; `client` and `server` are the programs'
    names in it (the client may be inside the macOS bundle)."""
    # All scratch stays under the repository target directory, never system /tmp.
    scratch = source / "target/parity-reports"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="package-", dir=scratch) as directory:
        root = Path(directory)
        game = root / "Jedi Academy space/GameData"
        game.mkdir(parents=True)
        with zipfile.ZipFile(package) as archive:
            assert archive.testzip() is None
            assert set(archive.namelist()) == set(packaged)
            archive.extractall(game)
        for program in (client, server):
            (game / program).chmod(0o755)
        started = subprocess.run([str(game / server),
                                 "--bind", "127.0.0.1:0", "--quit-on-eof"],
                                cwd=root, input="", text=True,
                                check=True, capture_output=True, timeout=30)
        assert "listening on 127.0.0.1:" in started.stdout, started.stderr
        (game / "base").mkdir()
        for number in (0, 3):
            (game / f"base/assets{number}.pk3").touch()
        env = os.environ.copy()
        for name in ("JKA_GAME_DATA", "DISPLAY", "WAYLAND_DISPLAY"):
            env.pop(name, None)
        env["XDG_CONFIG_HOME"] = str(root / "profile")
        env["APPDATA"] = str(root / "profile")
        # The start-failure dialog would wait for a click on a runner with a desktop.
        env["SJK_NO_START_DIALOG"] = "1"
        run = subprocess.run([str(game / client)], cwd=root,
                             env=env, capture_output=True, text=True, timeout=60)
        assert run.returncode == 1, run.stderr
        assert "was not found in the mounted game data" in run.stderr, run.stderr
        assert (game / profile / "config.cfg").is_file(), run.stderr
        print("Extracted client discovered adjacent synthetic assets and saved portable settings; loopback server startup/shutdown passed.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--target", required=True)
    parser.add_argument("--platform", choices=("linux-x64", "macos-arm64", "windows-x64"), required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--smoke-check", action="store_true")
    # Release builds name their files after the product and version and point the
    # bundled instructions at the repository that publishes the source.
    parser.add_argument("--name", default="SJK")
    parser.add_argument("--version")
    parser.add_argument("--repository", default=DEFAULT_REPOSITORY)
    # The client folder the packaged client creates in GameData.
    parser.add_argument("--profile-dir", default="SJK")
    # The client and server program names; by default the [[bin]] names the crates
    # declare (sjk and sjk-server).
    parser.add_argument("--client-bin")
    parser.add_argument("--server-bin")
    args = parser.parse_args()
    source, output = args.source.resolve(), args.output.resolve()
    args.client_bin = args.client_bin or bin_name(source, "sjk-viewer")
    args.server_bin = args.server_bin or bin_name(source, "sjk-dedicated")
    if command(source, "git", "status", "--porcelain", "--untracked-files=no"):
        raise SystemExit("Refusing to package a modified source checkout")
    revision = command(source, "git", "rev-parse", "HEAD")
    output.mkdir(parents=True, exist_ok=True)
    stem = f"{args.name}-{args.version or revision[:7]}"
    package = output / f"{stem}-{args.platform}.zip"
    suffix = ".exe" if args.platform == "windows-x64" else ""
    binaries = [source / "target" / args.target / "release" / (name + suffix)
                for name in (args.client_bin, args.server_bin)]
    # macOS disks ignore case by default: a program named `sjk` beside the `SJK`
    # client folder takes its name, and the client falls back to per-user storage.
    client = args.client_bin
    if args.platform == "macos-arm64" and client.lower() == args.profile_dir.lower():
        client += "-client"
    with tempfile.TemporaryDirectory(prefix="bundle-", dir=output) as staging, \
            zipfile.ZipFile(package, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        # On a Mac the client goes in an app bundle; elsewhere it is a bare program.
        if args.platform == "macos-arm64":
            programs = app_bundle(Path(staging), binaries[0], source, args.name, client,
                                  args.version)
        else:
            programs = [(client + suffix, binaries[0], True)]
        programs.append((args.server_bin + suffix, binaries[1], True))
        for packaged, path, executable in programs:
            add_file(archive, path, packaged, executable)
        client_entry = programs[0][0] if args.platform != "macos-arm64" else \
            f"{args.name}.app/Contents/MacOS/{client}"
        archive.writestr(text_name("README", args.name),
                         instructions(args.platform, revision, args.name, args.repository, args.version,
                                      args.profile_dir, client, args.server_bin))
        fonts = source / "crates/sjk-viewer/assets/fonts"
        # The copyright notice leads the file.
        notices = [(f"{args.name}/NOTICE", (source / "NOTICE").read_bytes()),
                   (f"{args.name}/LICENSE", (source / "LICENSE").read_bytes()),
                   ("Inter/LICENSE.txt", (fonts / "LICENSE.txt").read_bytes())]
        notices.extend((f"{name}/{path.name}", path.read_bytes())
                       for name, path in (("JetBrains Mono", fonts / "JetBrainsMono-OFL.txt"),
                                          ("Rajdhani", fonts / "Rajdhani-OFL.txt"),
                                          ("Exo 2", fonts / "Exo2-OFL.txt"),
                                          ("SJK fonts", fonts / "SJK-fonts.txt")))
        notices.extend(dependency_notices(source, args.target, args.name))
        archive.writestr(text_name("LICENSES", args.name), license_text(notices, args.name))
    manifest = output / f"{stem}-{args.platform}-build.json"
    manifest.write_text(json.dumps({
        "revision": revision, "target": args.target,
        "rustc": command(source, "rustc", "--version"),
        "profile": "release", "rustflags": os.environ.get("RUSTFLAGS", ""),
        "binaries": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in binaries},
    }, indent=2) + "\n", encoding="utf-8")
    if args.smoke_check:
        smoke_check(package, args.platform, source,
                    [packaged for packaged, _, _ in programs]
                    + [text_name("README", args.name), text_name("LICENSES", args.name)],
                    args.profile_dir, client_entry, args.server_bin + suffix, args.name)
    source_zip = output / f"{stem}-source.zip"
    subprocess.run(["git", "-c", "core.autocrlf=false", "archive", "--format=zip", f"--prefix={stem}/",
                    f"--output={source_zip}", revision], cwd=source, check=True)
    checksums = "".join(f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n"
                        for p in (package, source_zip, manifest))
    sums = f"{stem}-SHA256SUMS-{args.platform}.txt" if args.version else f"SHA256SUMS-{args.platform}.txt"
    (output / sums).write_text(checksums, encoding="utf-8")
    print(package)


if __name__ == "__main__":
    main()
