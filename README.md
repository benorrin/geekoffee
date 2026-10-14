<div align="center">

<img src="icons/coffee-on.svg" width="96" alt="Geekoffee icon">

# Geekoffee

**Keep your PC awake from the KDE Plasma system tray.**

A tiny Rust tray app for openSUSE. One click stops the screen from locking and
the machine from sleeping. Click again and everything goes back to your normal
settings.

![Rust](https://img.shields.io/badge/Rust-2024-b7410e?logo=rust)
![KDE Plasma](https://img.shields.io/badge/KDE_Plasma-6-1d99f3?logo=kde)
![openSUSE](https://img.shields.io/badge/openSUSE-73ba25?logo=opensuse&logoColor=white)
![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue)

</div>

---

## ☕ Usage

| Icon | State | What happens |
|:---:|---|---|
| <img src="icons/coffee-off.svg" width="32"> | **Off** | Your normal power settings apply |
| <img src="icons/coffee-on.svg" width="32"> | **On** | The screen won't dim or lock, and the PC won't sleep |

- **Left-click** the tray icon to switch keep-awake on or off.
- **Right-click** for a menu with **Keep awake** and **Quit**.
- **Hover** to see the current state.

> [!TIP]
> If you can't see the icon, Plasma may have put it behind the **^** arrow.
> Right-click the arrow and choose **Configure System Tray… → Entries**, then
> set Geekoffee to **Always shown**.

## 📦 Install

You need a Rust toolchain:

```sh
sudo zypper install cargo        # or use rustup
```

Build it and install it for your user:

```sh
cargo build --release

install -Dm755 target/release/geekoffee        ~/.local/bin/geekoffee
install -Dm644 icons/coffee-on.svg             ~/.local/share/icons/hicolor/scalable/apps/geekoffee.svg
install -Dm644 geekoffee.desktop               ~/.local/share/applications/geekoffee.desktop
```

To **start it on login**:

```sh
install -Dm644 geekoffee.desktop ~/.config/autostart/geekoffee.desktop
```

To **start it now**, without logging out:

```sh
geekoffee &
```

or search for *Geekoffee* in the app launcher.

## ⚙️ How it works

Geekoffee **never changes your power settings.** When you switch it on, it asks
KDE over D-Bus to hold off for a while:

| D-Bus API | Blocks |
|---|---|
| `org.freedesktop.ScreenSaver.Inhibit` | Screen dimming, blanking and locking |
| `org.freedesktop.PowerManagement.Inhibit` | Automatic suspend when idle |

- **Switching it off** cancels both requests, and KDE goes back to your
  configured behaviour.
- **If Geekoffee quits or crashes,** the session bus cancels them automatically,
  so the PC can't get stuck awake.
- While it's on, Geekoffee is listed in Plasma's **Power and Battery** applet.

> [!NOTE]
> KDE waits a few seconds before a new request takes effect. This is normal.

## 🗂️ Project layout

```
src/main.rs       Tray icon, menu and main loop
src/inhibit.rs    D-Bus requests to KDE to block and unblock
src/icon.rs       Renders the embedded SVG icons for the tray
icons/            Coffee-cup icons (on/off)
geekoffee.desktop Launcher and autostart entry
```

## 🗑️ Uninstall

```sh
pkill -x geekoffee
rm ~/.local/bin/geekoffee \
   ~/.local/share/icons/hicolor/scalable/apps/geekoffee.svg \
   ~/.local/share/applications/geekoffee.desktop \
   ~/.config/autostart/geekoffee.desktop
```

## 📄 License

[GPL-3.0](LICENSE)
