# Third-party and asset notices

Doubao Skin is an independent project and is not affiliated with or endorsed by ByteDance. “豆包”, “豆包工作”, “Doubao”, and related names and marks belong to their respective owners. This repository does not redistribute the official DoubaoWork application or its proprietary resources.

## Code licenses

- `crates/skin-core`, `apps/web`, project documentation, and original theme definitions are licensed under the repository [MIT License](LICENSE), unless a file or theme manifest says otherwise.
- `apps/desktop` and distributed desktop binaries are licensed under [GPL-3.0-or-later](LICENSES/GPL-3.0-or-later.txt). The pinned Zed dependency declares GPUI and `gpui_platform` as Apache-2.0, while the resolved desktop dependency graph includes `ztracing` and `zlog`, both declared GPL-3.0-or-later.
- Dependency versions are fixed by `Cargo.lock` and `apps/web/pnpm-lock.yaml`. Each dependency remains subject to its own license and notices.

## Themes and artwork

Theme manifests record palette sources and licenses when a theme derives from an existing color system. A source link is attribution and inspiration metadata; it does not imply affiliation.

Project-authored CSS, JSON, previews, icons, and generated backgrounds are covered by the component license unless their manifest states another license. Contributors must not add copied brand art, fonts, characters, screenshots, or other assets without explicit redistribution rights.
