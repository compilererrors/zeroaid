## Fork Notes (zeROAId)

This fork is optimized for a lean default build.

### Why This Fork Exists

This fork was created to keep the project focused on the core text editor experience.
The goal is a smaller build with fewer dependencies by minimizing optional extras in the default build.

Current focus areas to keep minimized by default:

- AI features
- Collaboration and calls tooling
- Audio-related features

More areas may be trimmed over time if they are not needed for the core editor workflow.

### What Changed in This Fork

- `zed` now builds with `default = []` (no default features).
- AI is feature-gated behind `ai`.
- Audio is feature-gated behind `audio`.
- Collaboration/Calls are feature-gated behind `collab`.
- `collab` includes `audio` + `audio/webrtc`.
- Collaboration/calls/audio settings pages are hidden when `collab` is not enabled.
- Title bar initialization remains enabled so the top bar/UI does not disappear in lean builds.

### How to Run

Lean (default, without AI/collab/audio):

```bash
cargo run -p zed
```

With audio only:

```bash
cargo run -p zed --features audio
```

With AI:

```bash
cargo run -p zed --features ai
```

With collaboration/calls (includes audio + webrtc):

```bash
cargo run -p zed --features collab
```

With both AI + collaboration:

```bash
cargo run -p zed --features ai,collab
```

### Build DMG (macOS)

Lean DMG (default in this fork):

```bash
script/bundle-mac
```

DMG with features:

```bash
script/bundle-mac -f audio
script/bundle-mac -f ai
script/bundle-mac -f collab
script/bundle-mac -f ai,collab
```

Explicitly without default features (same effect as lean here):

```bash
script/bundle-mac -n
```

### Offline Build (optional)

```bash
CARGO_NET_OFFLINE=true cargo check -p zed --no-default-features
CARGO_NET_OFFLINE=true script/bundle-mac
```

# Zed

[![Zed](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/zed-industries/zed/main/assets/badge/v0.json)](https://zed.dev)
[![CI](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml/badge.svg)](https://github.com/zed-industries/zed/actions/workflows/run_tests.yml)

Welcome to Zed, a high-performance, multiplayer code editor from the creators of [Atom](https://github.com/atom/atom) and [Tree-sitter](https://github.com/tree-sitter/tree-sitter).

---

### Installation

On macOS, Linux, and Windows you can [download Zed directly](https://zed.dev/download) or install Zed via your local package manager ([macOS](https://zed.dev/docs/installation#macos)/[Linux](https://zed.dev/docs/linux#installing-via-a-package-manager)/[Windows](https://zed.dev/docs/windows#package-managers)).

Other platforms are not yet available:

- Web ([tracking issue](https://github.com/zed-industries/zed/issues/5396))

### Developing Zed

- [Building Zed for macOS](./docs/src/development/macos.md)
- [Building Zed for Linux](./docs/src/development/linux.md)
- [Building Zed for Windows](./docs/src/development/windows.md)

### Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for ways you can contribute to Zed.

Also... we're hiring! Check out our [jobs](https://zed.dev/jobs) page for open roles.

### Licensing

License information for third party dependencies must be correctly provided for CI to pass.

We use [`cargo-about`](https://github.com/EmbarkStudios/cargo-about) to automatically comply with open source licenses. If CI is failing, check the following:

- Is it showing a `no license specified` error for a crate you've created? If so, add `publish = false` under `[package]` in your crate's Cargo.toml.
- Is the error `failed to satisfy license requirements` for a dependency? If so, first determine what license the project has and whether this system is sufficient to comply with this license's requirements. If you're unsure, ask a lawyer. Once you've verified that this system is acceptable add the license's SPDX identifier to the `accepted` array in `script/licenses/zed-licenses.toml`.
- Is `cargo-about` unable to find the license for a dependency? If so, add a clarification field at the end of `script/licenses/zed-licenses.toml`, as specified in the [cargo-about book](https://embarkstudios.github.io/cargo-about/cli/generate/config.html#crate-configuration).

## Sponsorship

Zed is developed by **Zed Industries, Inc.**, a for-profit company.

If you’d like to financially support the project, you can do so via GitHub Sponsors.
Sponsorships go directly to Zed Industries and are used as general company revenue.
There are no perks or entitlements associated with sponsorship.
