# ASCII Deck

A compact, customizable control applet for the [COSMIC](https://system76.com/cosmic) desktop panel — built around ASCII-style battery and brightness controls instead of traditional icons.

```text
87% ⚡[█████████░]
```
![Uploading Screenshot_2026-09-04_09-25-29.png…]()

## Features

* **Live battery indicator** with:

  * Battery percentage
  * Charging indicator
  * ASCII block bar
  * Automatic panel sizing
  * Charge-level colors

* **Popup control deck** with:

  * Estimated time to full charge / time remaining
  * Battery capacity and health information
  * Power profile switching

    * Power Saver
    * Balanced
    * Performance
  * ASCII brightness slider
  * Screen brightness control

* **ASCII battery visualization**

  * Vertical ASCII battery
  * Charging lightning indicator
  * Adjustable display options

* **100-dot battery grid**

  * Displays battery charge as a 10×10 dot grid
  * Filled and empty dots show the current charge level
  * Adjustable dot size
  * Can be enabled or disabled with a toggle

* **Panel customization**

  * Toggle battery display
  * Toggle brightness display
  * Change panel text size
  * Adjust ASCII block count
  * Adjust spacing
  * Reorder panel modules

* **Appearance customization**

  * Dark Mode toggle
  * System accent color support
  * Custom accent colors
  * Optional monospace font

* **Automatic configuration saving**

  * Settings persist between launches

## Installation

### Via the COSMIC Flatpak repository

This applet is submitted to the COSMIC Flatpak repository and is pending review.

Once available:

```bash
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install --user cosmic com.github.pewmoe.cosmic-ext-ascii-dot-battery
```

Then add it to your panel through:

**Settings → Desktop → Panel → Add Applet**

### Building from source

Requires [`just`](https://github.com/casey/just), a Rust toolchain, and the COSMIC development libraries.

```bash
git clone https://github.com/pewmoe/Cosmic-Desktop-ASCII-Battery-Applet.git
cd Cosmic-Desktop-ASCII-Battery-Applet
just build-release
sudo just install
```

Restart COSMIC Panel and add the applet from the panel settings.

For example:

```bash
killall cosmic-panel
```

## Development

```bash
just build-debug     # Build a debug version
just build-release   # Build an optimized release
just run             # Run locally for testing
just check           # Run checks / clippy
```

## Known Issues

* The **notification applet disappearing** issue is related to the COSMIC notification applet rather than ASCII Deck. Logging out, rebooting, or restarting the relevant COSMIC components can restore it.
* The **Dark mode** isn't fully working yet.

## Roadmap

Planned features include:

* **open an issue for whatever feature you want and I'll think about it... or do it yourself and tell me**

## License

[MPL-2.0](LICENSE)

## Acknowledgments

Built with [libcosmic](https://github.com/pop-os/libcosmic) and originally scaffolded from [cosmic-app-template](https://github.com/pop-os/cosmic-app-template).

Special thanks to **KodeBarista** for their substantial help with development, troubleshooting, and getting ASCII Deck to where it is today.
