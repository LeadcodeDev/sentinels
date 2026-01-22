# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
cargo build          # Build the project
cargo run            # Run the game (opens 1200x800 window)
```

No tests are configured. The project uses Rust edition 2024.

## Key Dependencies

- **gpui 0.2.2** - Zed's GPU-accelerated UI framework (desktop-only, macOS primary)
- **gpui-component 0.5.0** - Component library (Button, Tooltip, Root)
- **core-text = "=21.0.0"** - Pinned to avoid core-graphics 0.24/0.25 version conflict

## Architecture

**Game loop**: `PlayScreen` spawns an async loop via `cx.spawn()` with `Timer::after(16ms)` for ~60 FPS. Each tick calls `game_state.tick(dt)` then `cx.notify()`.

**State management**: Centralized `GameState` struct in `src/game/mod.rs` owns all game entities. No distributed GPUI entities for game objects.

**Screen navigation**: Enum state machine in `SentinelsApp` (Welcome → Lobby → Shop/Play). Cross-screen communication uses `EventEmitter` pattern.

**Rendering**: Single `canvas()` element in `src/render/mod.rs`. All shapes drawn via `PathBuilder` polygons and `PaintQuad`. Game uses center-origin coordinates (player at 0,0), translated to screen pixels during render.

**Sidebar**: 200px right panel (`src/ui/hud.rs`) with stats + tower grid + selected tower upgrades. Game canvas occupies remaining width.

## Module Responsibilities

- `src/game/mod.rs` - GameState + tick() with 11-step update order
- `src/game/elemental.rs` - 5 elements, 6 reactions (damage/slow/stun effects)
- `src/game/tower.rs` - Tower entities with 4 upgrade types (Damage/Range/Speed/AoE)
- `src/game/enemy.rs` - Enemy movement, attack AI, damage/element application
- `src/game/wave.rs` - Wave spawning logic, difficulty scaling (+15% HP/wave)
- `src/data/mod.rs` - SaveData persistence to `~/.sentinels/save.json`
- `src/render/shapes.rs` - All 2D drawing primitives (polygons, circles, HP bars)
- `src/screens/play.rs` - Game loop host, input handling, layout composition
- `src/ui/hud.rs` - Sidebar with stats, tower icons, selected tower panel
- `src/ui/popover.rs` - Tower detail popover positioned at tower location

## UI Guidelines

Prefer using `gpui-component` widgets (Button, Tooltip, Root, etc.) over building custom UI elements from raw `div()`. The component library provides consistent styling, accessibility, and interaction patterns. Only use raw canvas drawing (`PathBuilder`, `PaintQuad`) for game-world rendering (shapes, projectiles, HP bars).

## GPUI Patterns & Gotchas

- **Listener signature**: `cx.listener(|this, event, window, cx|)` — always 4 args
- **Rust 2024 captures**: Functions returning `impl Trait` that borrow `cx` need `+ use<>` suffix
- **`Pixels` field is private**: Use `f32::from(pixels)` to extract value
- **`gen` is reserved in Rust 2024**: Use `rng.r#gen_range(...)` for rand calls
- **`FluentBuilder` trait**: Must be imported (`use gpui::prelude::FluentBuilder`) for `.when()` / `.when_some()`
- **`PaintQuad`**: Requires `border_style: BorderStyle::default()` field
- **Event propagation**: Add `on_mouse_down` handlers on overlay divs to prevent clicks from reaching elements underneath
- **Tooltip**: Use `Tooltip::new(text).build(window, cx)` to get `AnyView` for `.tooltip()` callback
- **Spawn closures**: Use `async |this: WeakEntity<T>, cx| { }` syntax; `WeakEntity::update` takes `cx` directly

## Dual Currency

- **Gold** (yellow): In-game currency, earned from kills, spent on towers/upgrades
- **Pepites** (purple): Persistent currency, random drops (10% per enemy, 3-5 per boss), used in shop for permanent upgrades
