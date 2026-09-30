# Exo Colony
A real-time simulation game about building a colony on an exoplanet for terminal lovers.

Inspired by Dwarf Fortress: the build cursor and the camera are two different things.

## Run

```
cargo run --release --bin exocolony
cargo run --release --bin exocolony -- --seed foothold
exocolony --help
```

`--seed` / `-s` fixes world generation. Omit it, and a random 8-character seed
is chosen and shown in the console and map title.

## Controls

| Input                                       | Action                                           |
|---------------------------------------------|--------------------------------------------------|
| Arrows / WASD                               | Move the build cursor                            |
| Alt+Arrows / Alt+WASD                       | Pan the camera (turns follow off)                |
| Alt+Shift+Arrows / Alt+Shift+WASD           | Page-pan half a view (turns follow off)          |
| F4                                          | Toggle camera-follow-cursor (edge slack)         |
| F3                                          | Pin home at the cursor                           |
| Home                                        | Return cursor and camera to the pinned home      |
| `n` / `N`                                   | Next / previous structure (camera follows jump)  |
| `b` / `B`                                   | Next / previous Base                             |
| Tab / Shift+Tab, `[` `]`, PageUp / PageDown | Cycle build menu groups                          |
| `;` `'` (also Tab / `[` `]`)                | Cycle build menu groups                          |
| `-` `=` / `,` `.` / End                     | Cycle structure variants                         |
| PageUp / PageDown                           | Scroll the console log                           |
| Enter                                       | Place structure (then it builds for a few ticks) |
| Delete                                      | Remove structure on the cursor tile              |
| `?`                                         | Reprint controls in the console                  |
| Esc                                         | Quit                                             |

The console keeps the last **256** lines, oldest first and newest at the bottom.
While you are pinned to the tail, new messages keep the view following.
Page Up looks back; Page Down returns to the tail and resumes follow.

The first Base you place is pinned as home automatically if none is set.

With follow-on, the camera only shifts when the cursor hits a margin inside the view (about three tiles).
Jumps (`n`/`N`, Home) still recenter.

On the map, lowercase structure glyphs are still under construction.
Bright glyphs are active this tick; dark glyphs are idle.
Deposits and mines are tinted by ore: rust Iron, silver Aluminum, pale Silica,
cyan Water, green Uranium, dark Carbon.

## Screenshots

### Map with resources and built structures

![Alt text](doc/exo-colony-screenshot-map.png "Map")

### Colony information widgets

![Alt text](doc/exo-colony-screenshot-colony-info.png "Colony Information")

### Console / Log

![Alt text](doc/exo-colony-screenshot-console.png "Game Console")

### Build menu with structures and variants

![Alt text](doc/exo-colony-screenshot-build-menu.png "Build Menu")

## Alpha Roadmap
This game is currently in the *alpha* stage.
We will move to *beta* once all game mechanics are implemented.

### 0.4 — Economy and the spaceport

* ExoCoin wallet and running costs
* Spaceport structure group (export commodities, import rare goods)
* Construction that consumes stockpiled resources (tick delay still applies)
* Cancel construction / refund
* Persist seed and colony to a save file

### Later alpha

* Building activity that drains local deposits (`available` on tiles) — mines (0.3.1)
* Power / logistics radius so a 256² map has local grids
* Fog of exploration

## Beta Roadmap
...
