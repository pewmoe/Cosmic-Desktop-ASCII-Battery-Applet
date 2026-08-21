# Cosmic Ascii Battery

A compact battery indicator applet for the [COSMIC](https://system76.com/cosmic) desktop panel — shown as an ASCII block bar instead of a traditional icon.

```
87% ⚡[██████░░]
```

## Features

- **Battery percentage and charge status** as a live-updating block bar directly in the panel, alongside a charging indicator
- **Automatically resizes** with your configured panel size — no fixed/cramped layout
- **Click to open a popup** showing:
  - Estimated time to full charge / time remaining
  - **Power profile switching** (Power Saver / Balanced / Performance)
  - **ASCII brightness slider** — click any segment to set screen brightness
  - **Accent color picker** — choose a fixed color, or leave it on Auto to color-code by charge level (red/orange/yellow/blue)

## Installation

### Via COSMIC Flatpak repo (recommended)

This applet is submitted to [pop-os/cosmic-flatpak](https://github.com/pop-os/cosmic-flatpak) and pending review. Once merged:

```bash
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install --user cosmic com.github.pewmoe.cosmic-ext-ascii-dot-battery

```

Then add it to your panel via **Settings → Desktop → Panel (or Dock) → Add applet**.

### Building from source

Requires [`just`](https://github.com/casey/just), a Rust toolchain, and the COSMIC development libraries.

```bash
git clone https://github.com/pewmoe/Cosmic-Desktop-ASCII-Battery-Applet.git
cd Cosmic-Desktop-ASCII-Battery-Applet
just build-release
sudo just install
```

Restart the panel (`killall cosmic-panel`) (or what ever your kill command is) and add the applet from Settings as above.

## Known issues

- **Screen brightness control does not currently work when running as a Flatpak.** Reading/writing brightness relies on system paths and D-Bus calls that need further sandbox permission work. Power profile switching and everything else works correctly in the Flatpak build. im still trying to figure out a way to fix it
## Development

```bash
just build-debug     # debug build
just run             # run locally for testing
just check           # clippy
```

## License

[MPL-2.0](LICENSE)

## possible future features
-**ASCII sound and brightness bar show on panel toggle.**
-**custom color picker.** 

## Acknowledgments

Built with [libcosmic](https://github.com/pop-os/libcosmic) and scaffolded from [cosmic-app-template](https://github.com/pop-os/cosmic-app-template).
