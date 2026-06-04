# Contributing to soomfonLinux

Thanks for helping bring the Soomfon stream deck to Linux! This project is meant
to be approachable for new contributors. If anything here is unclear, open an
issue and ask.

## Ways to contribute

- **Report device support** — if your Soomfon model isn't detected, open a
  *Device support* issue with your `lsusb` output. This is hugely valuable.
- **Fix bugs / build features** — see the [roadmap](README.md#roadmap) and the
  `good first issue` label.
- **Improve docs** — README, this file, code comments.

## Development setup

```sh
git clone https://github.com/Traxxouu/soomfonLinux.git
cd soomfonLinux
npm --prefix ui install
npm --prefix ui run tauri dev
```

See the [README](README.md#building-from-source) for system prerequisites.

## Branching & commits

- Branch off `main` using a descriptive prefix:
  - `feat/…` new feature, `fix/…` bug fix, `chore/…` tooling/infra, `docs/…` docs.
- Use [Conventional Commits](https://www.conventionalcommits.org/) for messages,
  e.g. `feat(device): detect Soomfon Stream Controller SE`.
- Keep PRs focused; one logical change per PR makes review easier.

## Before opening a PR

Run the same checks CI runs:

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
npm --prefix ui run build
```

Then open a PR against `main` and fill in the template. CI must be green before
review.

## Code style

- Rust: `rustfmt` defaults, `clippy` clean (warnings are denied in CI).
- Keep the layering intact: the UI talks to `src-tauri`, which calls
  `soomfon-core`, which calls `soomfon-device`. Don't reach across layers.
- Prefer small, well-named functions over comments. Comment the *why*, not the *what*.

## License of contributions

By contributing, you agree that your contributions are licensed under the
project's [GPL-3.0-or-later](LICENSE) license.
