# Flappy Bird (Bevy)

Flappy Bird built in Bevy (using Rust language) with multiple modes, difficulties, themes, skins, saving feature.

![Gameplay](https://github.com/AmerBidzevic/Flappy-Bird---Rust-/blob/main/assets/Gameplay.png?raw=true)

## Current features

### Modes
- Endless - Classic style of playing.
- Time Attack - 60s time rush, survive until timeout.
- Checkpoints - Every 5th obstacle saves your spawn point.
### Difficulty
- Easy - Large Gaps, Slow, Low Gravity.
- Normal - Standard difficulty.
- Hard - Smaller Gaps, Fast, High Gravity.
### Themes
- Classic - Original Look (Like in Main Menu).
HighContrast - Enhanced Visibility / Dark mode.
Minimal - Basic flat gray color.
### Saves
- Three slots, selectable on start. Slots can be deleted. Per-run scores update the profile high score and averages(also visible in Leaderboard)
### User Interface 
- Start Game
- Options (Saves, Game Mode, Difficult, Theme, Skin)
- Leaderboard
### Audio
- Flap/point/die/swoosh effects. 
- Menu music loads `assets/35-Lost-Woods.ogg` on loop.
### Controls
- SPACE: Start in menus, flap in-game, and return to Main Menu from Game Over.
- (1/2/3/4/5): Number select for options.
- Ctrl + (1/2/3): Delete the corresponding save slot in the Save Select menu.
### Saving
- Files live in `saves/slot_<n>.json` (1–3). Each run updates high score, total games, and average score for the selected slot. Saves are simple JSON; you can delete them manually or via the in-game shortcut.

## Assets needed
Place these in `assets/` (names must match exactly):

## How to run

- In console: `cargo run`
