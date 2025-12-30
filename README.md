# Flappy Bird (Bevy)

A small Flappy Bird remake in Rust/Bevy with multiple modes, difficulties, themes, and save slots.

![Gameplay](https://github.com/AmerBidzevic/Flappy-Bird---Rust-/blob/main/assets/Gameplay.png?raw=true)

## Current features

- Modes: Endless (classic scoring) and Time Attack (60s countdown). Checkpoints is listed but not implemented yet.
- Difficulties: Easy / Normal / Hard adjust gaps, gravity, speed, and flap force.
- Themes: Classic uses a background texture; HighContrast and Minimal use flat colors.
- Saves: Three slots, selectable on start. Slots can be deleted with Ctrl+1/2/3. Per-run scores update the profile high score and averages.
- UI: Menu flow (Main → Save → Mode → Difficulty → Theme), in-game score/best/time, Game Over screen with score and best, background resets in menus so theme colors do not bleed.
- Audio: Flap/point/die/swoosh effects. Menu music loads `assets/35-Lost-Woods.ogg` on loop.

## Controls

- SPACE: Start in menus, flap in-game, and return to Main Menu from Game Over.
- ESC: Go back one menu (Save/Mode/Difficulty/Theme screens).
- Ctrl + (1/2/3): Delete the corresponding save slot in the Save Select menu.

## Saving

- Files live in `saves/slot_<n>.json` (1–3). Each run updates high score, total games, and average score for the selected slot. Saves are simple JSON; you can delete them manually or via the in-game shortcut.

## Assets needed

Place these in `assets/` (names must match exactly):

## How to run

- `cargo run`
- If assets are missing, Bevy will log a path-not-found error.


## Rust Analyzer warning (path mismatch)

Need to find the reason for the bug but it does not affect the game in any way.

- Checkpoints mode is a stub.
- `GameState::Paused` is unused.
- Time Attack duration is fixed at 60s.
