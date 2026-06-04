# soomfonLinux

> Open-source desktop app to drive **Soomfon** stream decks on **Linux**.

Soomfon only ships official software for Windows and macOS. **soomfonLinux** is a
community effort to give Linux users a first-class way to use their device —
trigger OBS scenes, control Spotify, run shortcuts and more.

> [!WARNING]
> **Early stage / work in progress.** The project scaffolding is in place; device
> support is being wired in. See the [roadmap](#roadmap) and
> [open issues](https://github.com/Traxxouu/soomfonLinux/issues).

## About the hardware

Soomfon "Stream Controller" devices are **OEM rebrands of Mirabox / Ajazz**
hardware (also sold as Mirabox, Ajazz, Vapourd, StreamDock…). The USB protocol
has been reverse-engineered by the community, so soomfonLinux builds on the
excellent [`mirajazz`](https://github.com/4ndv/mirajazz) crate rather than
reinventing it.

If you are looking for a more generic, plugin-based stream deck app today, see
[OpenDeck](https://github.com/nekename/OpenDeck) and the
[opendeck-soomfon](https://github.com/virlatinus/opendeck-soomfon) plugin.
soomfonLinux instead aims to be a focused, Soomfon-first app with batteries-included
integrations.

## Architecture

The codebase is a Cargo workspace with a clear separation of concerns:

| Crate / dir         | Responsibility                                                        |
| ------------------- | --------------------------------------------------------------------- |
| `crates/soomfon-device` | Hardware layer — wraps `mirajazz`, exposes a transport-agnostic API. |
| `crates/soomfon-core`   | App logic — device discovery, profiles, pages, action dispatch.     |
| `src-tauri`             | Tauri (Rust) backend — thin glue exposing the core to the frontend. |
| `ui`                    | Svelte + Vite frontend — the button grid and editor.                |
| `packaging`             | udev rules and (later) AppImage / Flatpak / AUR packaging.          |

The layers only depend downward (`ui → src-tauri → soomfon-core → soomfon-device`),
so USB details never leak into the UI.

## Building from source

**Prerequisites**

- Rust (stable) and Cargo — <https://rustup.rs>
- Node.js 18+ and npm
- Tauri Linux system dependencies — see the
  [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) (e.g.
  `webkit2gtk`, `gtk3`, `libappindicator`, `librsvg`).

**Run in development**

```sh
# install frontend deps once
npm --prefix ui install

# launch the app with hot reload (needs the @tauri-apps/cli devDependency)
npm --prefix ui run tauri dev
```

**Build a release bundle**

```sh
npm --prefix ui run tauri build
```

## Device access (udev)

To talk to the deck without root, install the udev rule:

```sh
sudo cp packaging/udev/70-soomfon.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Then unplug and replug the device.

## Roadmap

- [x] Project scaffolding (workspace, Tauri + Svelte, CI, packaging skeleton)
- [x] Device detection via `mirajazz` (read key/encoder input)
- [x] Render images and text to keys, set brightness
- [ ] Profiles, pages and button configuration with persistence
- [ ] Visual button-grid editor
- [ ] Actions: run command / hotkey / open app
- [ ] OBS integration (scene & slide control)
- [ ] Spotify integration (MPRIS / Web API)
- [ ] Packaging: AppImage / Flatpak / AUR

## Contributing

Contributions are very welcome — this project exists for the community. Please
read [CONTRIBUTING.md](CONTRIBUTING.md). If your specific Soomfon model is not
detected, open a
[device support issue](https://github.com/Traxxouu/soomfonLinux/issues/new/choose)
with the output of `lsusb`.

## License

Licensed under the **GNU General Public License v3.0 or later**. See
[LICENSE](LICENSE).

## Acknowledgements

- [`mirajazz`](https://github.com/4ndv/mirajazz) — the Mirabox/Ajazz/Soomfon USB protocol library.
- [OpenDeck](https://github.com/nekename/OpenDeck) and [opendeck-soomfon](https://github.com/virlatinus/opendeck-soomfon) — prior art and inspiration.
