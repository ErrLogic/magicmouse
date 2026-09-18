# Magic Mouse Linux Suite

> Native Linux control suite for Apple Magic Mouse  
> Target platform: Linux / Wayland / Hyprland  
> Primary language: Rust

---

## 1. Project Vision

Magic Mouse Linux Suite adalah aplikasi native Linux yang memberikan pengalaman penggunaan Apple Magic Mouse yang lebih lengkap di Linux.

Project ini terinspirasi oleh konsep aplikasi seperti Magic Utilities, tetapi dirancang khusus untuk ekosistem Linux dan dibuat dengan pendekatan native, modular, configurable, dan extensible.

Tujuan utama:

- Menggunakan Magic Mouse secara proper di Linux.
- Mengakses multitouch data yang tersedia dari Linux input subsystem.
- Menyediakan konfigurasi pointer dan scrolling.
- Menyediakan gesture recognition.
- Menyediakan button mapping.
- Menyediakan device naming/alias.
- Menyediakan battery information.
- Menyediakan live touch visualization.
- Menyediakan background daemon.
- Menyediakan CLI control.
- Menyediakan GUI configuration.
- Menyediakan integrasi Hyprland.
- Menjaga core gesture engine agar tidak bergantung langsung pada Hyprland.
- Menjadikan Hyprland sebagai salah satu integration backend.

---

# 2. Current Hardware Evidence

Magic Mouse yang digunakan saat development terdeteksi Linux sebagai:

```text
Input device:
"Fikry’s Magic Mouse"

Bus:
0x5

Vendor:
0x4c

Product:
0x323

Version:
0x314
```

Linux input device menyediakan:

```text
EV_KEY
EV_REL
EV_ABS
EV_MSC
```

Multitouch capability yang tersedia:

```text
ABS_MT_SLOT
ABS_MT_TOUCH_MAJOR
ABS_MT_TOUCH_MINOR
ABS_MT_ORIENTATION
ABS_MT_POSITION_X
ABS_MT_POSITION_Y
ABS_MT_TRACKING_ID
```

Device mengekspos slot:

```text
0 - 15
```

dan pengujian aktual telah membuktikan minimal tiga simultaneous contacts.

Contoh:

```text
slot 2  → tracking ID 35
slot 6  → tracking ID 36
slot 11 → tracking ID 37
```

Ketiganya muncul dalam satu input frame sebelum `SYN_REPORT`.

Data tersebut menunjukkan bahwa Linux sudah melakukan decoding input Magic Mouse menjadi multitouch events yang dapat dibaca oleh userspace.

Raw event source:

```text
/dev/input/eventXX
```

Project tidak akan melakukan reverse engineering Apple HID protocol pada tahap awal.

---

# 3. High-Level Architecture

```text
┌───────────────────────────────┐
│        Apple Magic Mouse      │
│           Bluetooth           │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│       Linux HID Subsystem     │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│      Linux Input Subsystem    │
│                               │
│      /dev/input/eventXX       │
└───────────────┬───────────────┘
                │
                ▼
┌────────────────────────────────────┐
│               magicd               │
│                                    │
│  Device Detection                  │
│  Input Reader                      │
│  Touch Tracker                     │
│  Gesture Engine                    │
│  Sensitivity Engine                │
│  Scroll Engine                     │
│  Action Engine                     │
│  Configuration                     │
│  Battery                           │
│  IPC                               │
└───────────────┬────────────────────┘
                │
       ┌────────┼─────────┐
       │        │         │
       ▼        ▼         ▼
    magicctl   GUI     Integrations
                         │
                         ├── Hyprland
                         ├── Generic Linux
                         └── Future WMs
```

---

# 4. Core Design Principles

## 4.1 Native Linux First

Tidak menggunakan browser/Electron sebagai fondasi daemon.

Core implementation menggunakan Rust.

---

## 4.2 Daemon First, GUI Later

GUI bukan bagian dari core system.

Daemon harus dapat berjalan tanpa GUI.

```text
magicd
```

harus menjadi source of truth untuk device state dan configuration.

---

## 4.3 Hyprland Is An Integration

Core gesture engine tidak boleh memiliki dependency langsung terhadap Hyprland.

Contoh:

```text
Gesture
    ↓
Action
    ↓
Backend
    ↓
Hyprland
```

bukan:

```text
Gesture
    ↓
Hyprland command
```

---

## 4.4 Raw Input Must Remain Observable

Selama development, kita harus dapat melihat raw input dan normalized input.

Debugging tool adalah bagian resmi dari project, bukan temporary throwaway script.

---

## 4.5 Configuration Must Be Persistent

Setting user harus disimpan dan dapat digunakan kembali setelah daemon restart.

---

## 4.6 Hardware-Specific Logic Must Be Isolated

Magic Mouse-specific behavior harus dipisahkan dari generic gesture engine.

Tujuannya agar gesture engine suatu hari dapat menerima device lain.

---

# 5. Target Repository Structure

Initial repository:

```text
magicmouse/
├── README.md
├── LICENSE
├── Cargo.toml
├── Cargo.lock
│
├── docs/
│   ├── PROJECT_PLAN.md
│   ├── ARCHITECTURE.md
│   ├── GESTURE_ENGINE.md
│   ├── CONFIGURATION.md
│   ├── HYPRLAND.md
│   └── DEVELOPMENT.md
│
├── scripts/
│   ├── setup.sh
│   ├── install-deps.sh
│   ├── dev.sh
│   ├── check.sh
│   └── uninstall.sh
│
├── crates/
│   ├── magic-core/
│   ├── magic-linux/
│   ├── magic-hyprland/
│   ├── magicd/
│   └── magicctl/
│
├── config/
│
├── systemd/
│   └── magicd.service
│
├── tests/
│
└── assets/
```

Structure ini dapat berubah berdasarkan kebutuhan implementasi.

---

# 6. Development Phases

Project development dilakukan secara bertahap.

Setiap phase harus menghasilkan sesuatu yang dapat dijalankan atau diverifikasi.

Urutan:

```text
Phase 0  → Project Bootstrap
Phase 1  → Device Discovery
Phase 2  → Raw Input Reader
Phase 3  → Touch Tracking
Phase 4  → Input Normalization
Phase 5  → Gesture Engine
Phase 6  → Pointer & Scroll Engine
Phase 7  → Action System
Phase 8  → Hyprland Integration
Phase 9  → Configuration System
Phase 10 → magicd
Phase 11 → magicctl
Phase 12 → GUI
Phase 13 → Battery & Device Features
Phase 14 → Testing & Hardening
Phase 15 → Packaging & Distribution
```

---

# 7. PHASE 0 — Project Bootstrap

## Objective

Menyiapkan seluruh development environment dan repository sehingga phase berikutnya dapat langsung dimulai.

Phase 0 harus dimulai dari kondisi:

```text
directory belum ada
```

dan berakhir dengan:

```text
project berhasil dibuat
dependencies tersedia
workspace berhasil compile
development tooling tersedia
basic validation berhasil
```

---

## 7.1 Create Project Directory

Project dibuat melalui script shell.

Contoh:

```bash
mkdir magicmouse
cd magicmouse
```

Namun setup final harus dilakukan melalui:

```bash
./scripts/setup.sh
```

Tujuannya agar project dapat direproduksi pada installation baru.

---

## 7.2 Setup Script

`setup.sh` bertanggung jawab untuk:

1. Membuat directory project.
2. Membuat struktur repository.
3. Mengecek operating system.
4. Mengecek architecture.
5. Mengecek Rust.
6. Mengecek Cargo.
7. Mengecek system dependencies.
8. Menjalankan dependency installer jika diperlukan.
9. Membuat Cargo workspace.
10. Membuat initial crates.
11. Membuat basic documentation.
12. Menjalankan formatter.
13. Menjalankan compiler check.
14. Menampilkan hasil setup.

Setup script harus aman untuk dijalankan ulang.

Contoh prinsip:

```text
run setup.sh
     ↓
existing directory?
     ↓
yes → preserve existing files
no  → create structure
     ↓
check dependencies
     ↓
install missing dependencies
     ↓
cargo check
     ↓
READY
```

Script tidak boleh sembarangan menghapus file user.

---

# 8. PHASE 0 — Dependency Installation

Dependency dibagi menjadi beberapa kategori.

## Rust Toolchain

Minimal:

```text
rustc
cargo
rustfmt
clippy
```

Toolchain harus diverifikasi:

```bash
rustc --version
cargo --version
rustfmt --version
cargo clippy --version
```

---

## Linux Input Dependencies

Project membutuhkan akses terhadap Linux input subsystem.

Candidate stack:

```text
evdev
udev
```

Pemilihan crate final dilakukan saat Phase 1/2 setelah kebutuhan API dipastikan.

---

## IPC

Daemon membutuhkan mekanisme komunikasi dengan client.

Candidate:

```text
Unix Domain Socket
D-Bus
```

Keputusan final dilakukan setelah Phase 10 architecture review.

---

## Hyprland

Hyprland integration harus menggunakan interface resmi yang tersedia dari compositor.

Implementation tidak boleh membuat dependency Hyprland pada core crate.

---

## GUI

GUI dependencies **tidak perlu dipasang pada Phase 0 jika GUI belum ditentukan**.

Framework GUI akan dipilih setelah core functionality stabil.

Candidate:

```text
GTK4 / libadwaita
Iced
Slint
egui
```

Pemilihan dilakukan pada Phase 12.

---

# 9. PHASE 0 — Cargo Workspace

Workspace awal:

```text
crates/
├── magic-core/
├── magic-linux/
├── magicd/
└── magicctl/
```

`magic-hyprland` dapat ditambahkan ketika integration mulai diperlukan.

Dependency relationship awal:

```text
magic-core
    ↑
magic-linux
    ↑
magicd

magicctl
    ↓
IPC
    ↓
magicd
```

Core tidak boleh bergantung pada daemon.

---

# 10. PHASE 0 — Initial Validation

Setelah setup selesai:

```bash
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets
```

Expected result:

```text
0 compilation errors
0 formatting errors
```

Clippy warnings harus ditinjau, bukan otomatis diabaikan.

---

# 11. PHASE 0 — Development Commands

Project harus menyediakan command sederhana.

Contoh:

```bash
./scripts/setup.sh
./scripts/install-deps.sh
./scripts/check.sh
./scripts/dev.sh
```

`check.sh`:

```text
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets
```

`dev.sh` nantinya menjadi entry point untuk development workflow.

---

# 12. PHASE 0 — Definition of Done

Phase 0 dianggap selesai apabila:

- [ ] Repository structure terbentuk.
- [ ] `setup.sh` tersedia.
- [ ] `install-deps.sh` tersedia.
- [ ] Rust toolchain terdeteksi.
- [ ] Linux dependencies tervalidasi.
- [ ] Cargo workspace berjalan.
- [ ] Initial crates berhasil compile.
- [ ] `cargo fmt --check` berhasil.
- [ ] `cargo check --workspace` berhasil.
- [ ] `cargo clippy` dapat dijalankan.
- [ ] README awal tersedia.
- [ ] Development documentation tersedia.
- [ ] Setup dapat diulang tanpa merusak existing project.
- [ ] Tidak ada dependency GUI yang dipaksakan sebelum waktunya.

---

# 13. PHASE 1 — Device Discovery

## Objective

Menemukan Magic Mouse secara otomatis.

Output yang diharapkan:

```text
Magic Mouse detected

Name: Fikry’s Magic Mouse
Vendor: 0x04c
Product: 0x0323
Input device: /dev/input/event20
```

Device discovery tidak boleh bergantung pada nomor:

```text
event20
```

karena nomor event dapat berubah.

Discovery harus menggunakan stable device attributes.

---

# 14. PHASE 2 — Raw Input Reader

## Objective

Membaca raw Linux input events.

Input:

```text
/dev/input/eventXX
```

Output debug:

```text
EVENT
type: EV_ABS
code: ABS_MT_SLOT
value: 2
```

dan:

```text
EVENT
type: EV_ABS
code: ABS_MT_TRACKING_ID
value: 35
```

Tool:

```text
magicmouse-debug
```

harus dapat berjalan tanpa gesture engine.

---

# 15. PHASE 3 — Touch Tracking

## Objective

Mengubah raw events menjadi state touch yang dapat diproses.

Model awal:

```rust
Touch {
    slot,
    tracking_id,
    x,
    y,
    dx,
    dy,
    major,
    minor,
    orientation,
}
```

Frame:

```rust
TouchFrame {
    timestamp,
    touches,
}
```

Touch lifecycle:

```text
DOWN
  ↓
MOVE
  ↓
MOVE
  ↓
UP
```

Tracking ID digunakan sebagai identitas contact.

---

# 16. PHASE 4 — Input Normalization

Raw coordinates tidak boleh langsung digunakan gesture engine.

Pipeline:

```text
Raw Input
    ↓
Coordinate Normalization
    ↓
Noise Filtering
    ↓
Delta Calculation
    ↓
Velocity
    ↓
Gesture Engine
```

Parameter yang perlu diperhatikan:

```text
sensitivity
threshold
deadzone
velocity
acceleration
smoothing
```

---

# 17. PHASE 5 — Gesture Engine

Gesture engine menerima:

```text
TouchFrame
```

dan menghasilkan:

```text
GestureEvent
```

Contoh:

```rust
SwipeLeft
SwipeRight
SwipeUp
SwipeDown

TwoFingerTap
ThreeFingerTap

TwoFingerSwipe
ThreeFingerSwipe

PinchIn
PinchOut

Hold
```

Gesture engine harus bersifat deterministic dan testable.

Gesture recognition tidak boleh langsung mengeksekusi shell command.

---

# 18. PHASE 6 — Pointer & Scroll Engine

Fitur:

```text
Pointer speed
Pointer sensitivity
Pointer acceleration
Scroll speed
Scroll sensitivity
Natural scrolling
Smooth scrolling
Horizontal scrolling
```

Input:

```text
raw movement
```

output:

```text
normalized movement
```

dan:

```text
scroll delta
```

---

# 19. PHASE 7 — Action System

Gesture harus menghasilkan abstract action.

Contoh:

```rust
Action::WorkspaceNext
Action::WorkspacePrevious
Action::OpenLauncher
Action::Scroll
Action::Back
Action::Forward
Action::CustomCommand(...)
```

Architecture:

```text
Gesture
   ↓
Action
   ↓
Backend
```

Dengan demikian gesture engine tidak mengetahui bagaimana action dieksekusi.

---

# 20. PHASE 8 — Hyprland Integration

Hyprland menjadi integration backend pertama.

Contoh mapping:

```text
3-finger swipe left
    ↓
WorkspacePrevious
    ↓
Hyprland backend
```

Configuration:

```toml
[gestures.three_finger_swipe_left]
action = "workspace.previous"

[gestures.three_finger_swipe_right]
action = "workspace.next"

[gestures.three_finger_swipe_up]
action = "overview"
```

Hyprland integration tidak boleh masuk ke `magic-core`.

---

# 21. PHASE 9 — Configuration System

Configuration harus persistent.

Candidate location:

```text
~/.config/magicmouse/
```

Contoh:

```text
~/.config/magicmouse/config.toml
```

Configuration mencakup:

```text
device alias
pointer sensitivity
pointer acceleration
scroll sensitivity
natural scrolling
gesture mappings
button mappings
Hyprland settings
daemon settings
```

Configuration harus memiliki default yang aman.

---

# 22. PHASE 10 — magicd

Daemon:

```bash
magicd
```

Responsibilities:

```text
Device discovery
Input reading
Touch tracking
Gesture processing
Configuration
Action dispatch
IPC
Battery monitoring
Logging
```

Daemon harus dapat berjalan:

```bash
systemctl --user start magicd
```

dan:

```bash
systemctl --user enable magicd
```

---

# 23. PHASE 11 — magicctl

CLI client:

```bash
magicctl status
magicctl devices
magicctl gestures
magicctl reload
magicctl config
```

Contoh:

```text
$ magicctl status

Magic Mouse
────────────────────────

Status       Connected
Device       Fikry’s Magic Mouse
Battery      82%
Pointer      1.00x
Scroll       1.20x
Gestures     Enabled
Hyprland     Connected
```

`magicctl` berkomunikasi dengan daemon melalui IPC.

CLI tidak membaca raw device secara langsung ketika daemon sudah aktif.

---

# 24. PHASE 12 — GUI

GUI dibuat setelah core system stabil.

Fitur minimum:

### Device

```text
Connected
Name
Alias
Battery
Connection
```

### Pointer

```text
Speed
Sensitivity
Acceleration
```

### Scrolling

```text
Speed
Smooth scrolling
Natural scrolling
Horizontal scrolling
```

### Gestures

```text
2 finger
3 finger
4 finger
```

### Mapping

```text
Gesture → Action
Button → Action
```

### Debug

Live touch visualization:

```text
┌──────────────────────────┐
│                          │
│    ●               ●     │
│                          │
│            ●             │
│                          │
└──────────────────────────┘

Contacts: 3
```

---

# 25. PHASE 13 — Device Features

Additional features:

```text
Battery status
Connection status
Device alias
Device profiles
Multiple Magic Mouse support
Automatic reconnect
```

Profile example:

```text
Default
Development
Gaming
Hyprland
```

---

# 26. PHASE 14 — Testing & Hardening

Testing harus mencakup:

## Unit Tests

```text
coordinate normalization
velocity calculation
touch tracking
gesture classification
configuration parsing
action mapping
```

## Integration Tests

```text
raw input → touch frame
touch frame → gesture
gesture → action
action → backend
```

## Hardware Tests

```text
1 finger
2 finger
3 finger
4 finger
tap
hold
swipe
slow movement
fast movement
simultaneous contacts
finger replacement
```

Important cases:

```text
finger disappears
tracking ID changes
slot reuse
device disconnect
device reconnect
```

---

# 27. PHASE 15 — Packaging & Distribution

Target installation methods:

```text
cargo install
binary release
Arch Linux package
AUR
```

Potential future:

```text
.deb
.rpm
Flatpak
```

Systemd user service:

```text
magicd.service
```

Installation harus tidak membutuhkan root untuk penggunaan normal daemon.

Root privileges hanya digunakan jika memang diperlukan untuk system-level installation/configuration.

---

# 28. Security Considerations

Daemon akan membaca:

```text
/dev/input/eventXX
```

dan mungkin mengeksekusi actions.

Karena itu:

- Jangan menjalankan daemon sebagai root kecuali benar-benar diperlukan.
- Hindari arbitrary shell execution sebagai default.
- Custom command harus explicit opt-in.
- IPC socket harus memiliki permission yang tepat.
- Configuration tidak boleh dapat diubah user lain.
- Jangan menyimpan credential.
- Logging tidak boleh membocorkan data sensitif.
- Device access harus dibatasi pada device yang diperlukan.

---

# 29. Logging

Logging harus tersedia sejak daemon mulai dibuat.

Level:

```text
ERROR
WARN
INFO
DEBUG
TRACE
```

Contoh:

```text
INFO  device connected
INFO  magic mouse detected
DEBUG touch started
DEBUG gesture recognized
INFO  action dispatched
WARN  device disconnected
```

Raw event logging harus configurable karena event stream dapat sangat besar.

---

# 30. Observability / Debug Mode

Development mode:

```bash
magicd --debug
```

atau:

```bash
MAGICMOUSE_LOG=debug magicd
```

Debug output dapat menampilkan:

```text
touch count
tracking IDs
slots
coordinates
velocity
gesture state
recognized gesture
action
```

Tujuannya memudahkan debugging tanpa perlu kembali ke `evtest`.

---

# 31. Initial Feature Scope

### MVP

MVP harus fokus pada:

```text
Magic Mouse detection
        ↓
Raw multitouch reading
        ↓
Touch tracking
        ↓
3-finger gesture recognition
        ↓
Action mapping
        ↓
Hyprland integration
        ↓
Persistent configuration
        ↓
Background daemon
```

GUI belum menjadi requirement MVP awal.

---

# 32. Explicitly Out of Scope for Early Development

Jangan mengerjakan ini terlalu awal:

```text
Apple HID reverse engineering
custom kernel driver
GUI polishing
multi-distro packaging
Flatpak
multiple desktop environment support
cloud sync
mobile companion app
```

Fokus awal adalah membuktikan pipeline:

```text
Magic Mouse
    ↓
Linux Input
    ↓
Rust
    ↓
Touch Tracking
    ↓
Gesture
    ↓
Hyprland
```

---

# 33. Development Strategy

Setiap phase harus memiliki:

```text
Objective
Implementation
Validation
Definition of Done
```

Tidak boleh lompat terlalu jauh sebelum phase sebelumnya tervalidasi.

Contoh:

```text
Phase 2
Raw Input Reader
       ↓
VALIDATE
       ↓
Phase 3
Touch Tracking
       ↓
VALIDATE
       ↓
Phase 4
Normalization
```

Dengan pendekatan ini kita selalu memiliki working state.

---

# 34. First Development Milestone

Milestone pertama setelah Phase 0:

```text
magicmouse-debug
```

Expected behavior:

```text
$ magicmouse-debug

Device:
  Fikry’s Magic Mouse

Connection:
  Connected

Contacts: 3

[slot 2]
  tracking: 35
  x: -891
  y: -638

[slot 6]
  tracking: 36
  x: 847
  y: -877

[slot 11]
  tracking: 37
  x: 17
  y: -1031
```

Kemudian ketika finger bergerak:

```text
Contacts: 3

slot 2
  dx: +12
  dy: -4

slot 6
  dx: +14
  dy: -6

slot 11
  dx: +11
  dy: -5
```

Ini menjadi dasar seluruh gesture engine.

---

# 35. Final Target

Pada akhirnya user dapat:

```text
Connect Magic Mouse
        ↓
magicd automatically detects it
        ↓
Mouse works normally
        ↓
Touch data is tracked
        ↓
Gestures are recognized
        ↓
Settings are configurable
        ↓
Mappings are configurable
        ↓
Hyprland actions execute
        ↓
GUI provides configuration
```

Target experience:

```text
Install
  ↓
Connect Magic Mouse
  ↓
Open Magic Mouse Linux
  ↓
Configure
  ↓
Use
```

Tanpa perlu menjalankan script gesture manual.

---

# 36. Current Decision

Development dimulai dari:

```text
PHASE 0
Project Bootstrap
```

Urutan awal:

```text
1. Create project directory
2. Create setup.sh
3. Create repository structure
4. Detect Linux environment
5. Check Rust toolchain
6. Install/validate dependencies
7. Initialize Cargo workspace
8. Create initial crates
9. Add basic documentation
10. Add formatting/lint/check scripts
11. Run cargo check
12. Run clippy
13. Verify Phase 0
14. Move to Phase 1
```

**Tidak membuat gesture engine sebelum raw input dan touch tracking tervalidasi.**

---

# 37. Project Philosophy

Project ini bukan:

```text
"script untuk menjalankan gesture Magic Mouse"
```

Project ini adalah:

```text
A native Linux Magic Mouse control platform.
```

Dengan prinsip:

```text
Hardware
   ↓
Linux Input
   ↓
Rust Core
   ↓
Daemon
   ↓
Gesture / Action System
   ↓
Desktop Integration
   ↓
User Configuration
```

Hyprland adalah integration pertama, bukan batasan project.
