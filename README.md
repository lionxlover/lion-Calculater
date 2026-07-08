# Lion Calculator

A modern native calculator for **LionOS** built with Rust + Slint 1.17.

![Lion Calculator — 4 modes: Basic, Scientific, Financial, Programmer](./docs/screenshot.png)

## Requirements

| Tool | Minimum version |
|------|----------------|
| Rust / Cargo | **1.82** |
| Slint | **1.17** (fetched automatically) |
| OS | Linux (Wayland / X11), macOS, Windows |

## Build & Run

```bash
# Debug
cargo run

# Release (faster startup, smaller binary)
cargo build --release
./target/release/lion-calculator
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `0`–`9`, `.` | Input digits |
| `+`, `-`, `*`, `/` | Operators |
| `%` | Percent |
| `^` | Power (Scientific) |
| `(`, `)` | Parentheses (Scientific) |
| `=` or `Enter` | Evaluate |
| `Backspace` | Delete last digit |
| `Escape` | All-clear |
| `A`–`F` | Hex digits (Programmer) |

## Modes

### Basic
Standard four-function calculator with memory (M+, M−, MR, MC) and
chained operations. Percent key computes percentage of the pending value.

### Scientific
All Basic operations plus:
- Trigonometry: sin / cos / tan / asin / acos / atan (DEG or RAD toggle)
- Logarithms: log₁₀, ln
- Power: xʸ, √x, 1/x, x!
- Constants: π, e
- Extras: RND (random 0–1)

### Financial — TVM Solver
Enter any **4** of the 5 TVM variables (N, I/Y, PV, PMT, FV), then press
**CPT** and the target variable to solve for it using the standard ordinary
annuity formula:

```
PV·(1+i)ᴺ  +  PMT·[(1+i)ᴺ−1]/i  +  FV  =  0
```

Additional helpers:
- **Tax+** — add tax at the stored rate (default 20%)
- **Tax−** — remove tax from an inclusive total
- **Mar**  — gross margin % given cost (PV) and selling price

### Programmer
- Bases: **HEX / DEC / OCT / BIN** — live four-line conversion display
- Bit ops: AND, OR, XOR, NOT
- Shifts: `<<`, `>>`
- Hex digits A–F enabled when HEX base is active

## Architecture

```
lion-calculator/
├── build.rs                  Compiles Slint UI
├── ui/
│   └── app_window.slint      Complete UI: theme, all components, 4 pads
└── src/
    ├── main.rs               Entry point
    ├── app.rs                Slint ↔ Calculator bridge, callback wiring
    ├── calculator.rs         Unified state machine (all 4 modes)
    ├── parser.rs             Recursive-descent expression evaluator
    ├── history.rs            Bounded LIFO history store
    ├── memory.rs             M+/M−/MR/MC/MS register
    └── modes/
        ├── financial.rs      TVM solver (Newton-Raphson for I/Y), tax, margin
        └── programmer.rs     Base conversion, bitwise ops, shift
```

## Design System (Leonux)

All colours, radii, and spacings are defined in the `LT` global in
`ui/app_window.slint`. Key tokens:

| Token | Value | Role |
|-------|-------|------|
| `LT.card` | `#0d0e18` | Window background |
| `LT.btn`  | `#1b1d2b` | Button background |
| `LT.eq`   | `#2260c8` | Equals button |
| `LT.t-op` | `#58b8fc` | Operator text (blue) |
| `LT.t-dn` | `#f07878` | Danger text (AC / DEL) |
| `LT.t-acc`| `#f8a840` | Accent text (CPT / bitops) |
| `LT.r-btn`| `14px`    | Button corner radius |
