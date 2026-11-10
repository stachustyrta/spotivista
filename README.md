<p align="center">
  <img src="res/spotivista.ico" alt="SpotiVista logo" width="88" height="88">
</p>

<h1 align="center">SpotiVista</h1>

<p align="center"><strong>Spotify, native, lightweight, and authentic for Windows Vista and Windows 7.</strong><br>A tailored fork of Spotifast targeting Windows Vista SP2 (NT 6.0) and Windows 7 SP1 (NT 6.1).</p>

<p align="center">
  <a href="Cargo.toml"><img src="https://img.shields.io/badge/Built_with-Rust_1.75-1ed760" alt="Built with Rust 1.75"></a>
  <img src="https://img.shields.io/badge/Platform-Windows_Vista_SP2_%7C_Windows_7_SP1-0078D7" alt="Windows Vista SP2 / Windows 7 SP1">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-1ed760" alt="MIT license"></a>
</p>

SpotiVista is a lightweight, native Spotify client built with Rust, [egui](https://www.egui.rs), and native Win32/DWM shell integration. It delivers an authentic Windows Aero Glass look, native taskbar and tray media controls, and seamless Spotify playback on vintage hardware without heavy modern web browser wrappers.

**Playback requires Spotify Premium.** Free accounts can browse and search, but cannot stream through the native player.

---

## Highlights

- **Authentic Windows Aero Glass Integration**:
  - Dynamically extends DWM glass frame into the client area via `DwmExtendFrameIntoClientArea`.
  - Seamless fallback to solid borders when Desktop Window Manager (DWM) composition is disabled.
  - Transparent framebuffer clearing for hardware-accelerated blur-behind.
- **Native Shell Media Controls**:
  - **Windows Notification Area (System Tray)**: Dynamic tooltip showing `Track - Artist (Status)` and right-click Win32 popup menu (`TrackPopupMenuEx`) with Play/Pause, Next, Previous, and Restore controls.
  - **Windows 7 Taskbar Thumbnail Toolbar**: Live thumbnail playback buttons (`ITaskbarList3`) with dynamic playback state and track progress indication. Gracefully degrades to a clean no-op on Windows Vista SP2 (`E_NOINTERFACE`).
  - **Global Multimedia Keys**: Hardware multimedia keyboard keys handled directly via `WM_APPCOMMAND`.
- **Hardened OAuth PKCE Sign-In with Supermium**:
  - Secure loopback PKCE flow compatible with modern Spotify authentication.
  - Includes a prominent **"Copy Authorization Link (for Supermium)"** button in the sign-in screen, ensuring seamless login on vintage systems where Internet Explorer 7 or 8 cannot render modern web pages.
- **Hardware Compatibility & Pruned Bloat**:
  - Pinned to Rust `1.75.0` with static C runtime (`/MT`) and explicit subsystem target (`/SUBSYSTEM:WINDOWS,6.0`).
  - Completely excises MilkDrop/libprojectM C++ dependencies and Discord Rich Presence, eliminating high memory footprint and CPU wakeups on vintage Core 2 Duo processors.
  - DPAPI credential vault encrypts session tokens securely with `CryptProtectData`.

---

## System Requirements

- **Operating System**:
  - Windows Vista SP2 (x86 or x64) with Platform Update
  - Windows 7 SP1 (x86 or x64)
- **Processor**: Intel Core 2 Duo, AMD Athlon 64 X2, or newer (SSE2 baseline).
- **Graphics**: DirectX 9.0c / WDDM 1.0 capable graphics card supporting Desktop Window Manager (Aero Glass).
- **Web Browser for Sign-In**: [Supermium](https://win32subsystem.live/supermium/) (or another Chromium-based browser supporting modern TLS 1.3 and OAuth) to approve the one-time Spotify authentication prompt.

*(Note: Windows XP is intentionally unsupported; the PE loader immediately rejects the binary cleanly via `/SUBSYSTEM:WINDOWS,6.0`)*

---

## Build from Source

### Prerequisites

1. **Rust Toolchain**: Rust `1.75.0` (or `1.77.2`) via `rustup`:
   ```cmd
   rustup toolchain install 1.75.0-x86_64-pc-windows-msvc
   rustup default 1.75.0-x86_64-pc-windows-msvc
   ```
2. **Build Tools**: Visual Studio 2022 / C++ Build Tools with MSVC v143.

### Building

```cmd
cargo build --release
```

The resulting binary `target/release/spotivista.exe` has:
- Subsystem version `6.00` (Windows Vista / 7 native).
- Statically linked C runtime (`/MT`).
- Embedded Common Controls 6.0 manifest with Vista & 7 supported OS GUIDs.
- Embedded multi-resolution Aero icons.

---

## Getting Started

1. **Register a Spotify App**:
   - In the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard), click **Create app**.
   - Add the redirect URI: `http://127.0.0.1:8989/login`.
   - Select **Web API**.
2. **Launch SpotiVista**:
   - Paste your Client ID and click **Sign in with Spotify**.
   - Click **Copy Authorization Link (for Supermium)** and paste the link into Supermium.
   - Authorize access; SpotiVista will capture the callback and load your library immediately.
3. **Local Playback**:
   - In the player bar, choose **Set up playback here**.
   - Complete the playback authorization in Supermium. Your machine will appear as a native Spotify Connect device.

---

## License

SpotiVista is distributed under the [MIT License](LICENSE).  
Upstream Spotifast is created by Carmine Paolino.
SpotiVista is an independent project and is not affiliated with Spotify AB.
