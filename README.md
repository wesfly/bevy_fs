# bevy_fs

This is a little flight sim made in Bevy.

## Highlights

- Support for both keyboard and gamepad (gamepad is recommended).
- Terrain with real-world elevation data and collisions.
- Loading data from glTF with custom properties.
- 3D cockpit.
- Collisions using `avian3d` and the flight dynamics model crate.
  [`avian_fdm`](https://github.com/viccuad/avian_fdm).
- Water with screen space reflections.
- A settings file - `settings.json`.

## Preview

![(bevy_fs: preview 1)](./docs/preview-1.png)
![(bevy_fs: preview 2)](./docs/preview-2.png)
![(bevy_fs: preview 3)](./docs/preview-3.png)

## Community

- [Discord](https://discord.gg/epMBz5m2Ad)
- [Fluxer](https://fluxer.gg/6OosduvZ)

## Run

### Prerequisites

- **Rust toolchain:** [Install Rust](https://rust-lang.org/tools/install)

### Installation steps

1. Clone the repository

   ```bash
   git clone https://github.com/wesfly/bevy_fs.git && cd bevy_fs
   ```

2. Build and run the project

   ```bash
   cargo run --release
   ```

   This will build and launch `bevy_fs`. The first startup fetches terrain data
   and can take some time.

## Terrain

Terrain data will be stored in the `terrain_cache` folder after the first fetch
(the game window will be unresponsive while fetching). To change the coordinates
or the resolution, change `terrain.coordinates` or `terrain.level_of_detail` in
the settings. Note that the general maximum level of detail is 15, but in most
regions 14 or even less. Here is an interactive map for different resolutions:
<https://mapterhorn.com/coverage>

Note that high resolutions will result in high RAM usage, to tackle this issue,
turn down the maximum render distance in the settings.

## Controls

### Tips

#### Helicopter

You can start the engines by pressing `M`. If you now bring the throttle to
100%, you will start flying.

#### Aeroplane

You will start in the air with the engine already on, you just need to throttle
up.

### Pause

Press `P` to unpause the sim.

### General

- `F3` to take a screenshot (output: `screenshots/user`)
- `ESC` to return to menu
- `HJKL` to control the sun position

### Camera

- `RMB + drag` to orbit camera
- `C` to switch the camera view
- Mouse wheel to zoom camera
- `R` to reset camera

### Aircraft

- `G` to raise or lower landing gear
- `Z` to brake
- `=` to toggle parking brake
- `1` for position lights
- `2` for strobe lights
- `3` for formation lights
- `4` for anti-collision lights

You can change the steering device in the settings.

### Some additional bindings for different steering devices

#### Controller

- Left stick to throttle and yaw
- Right stick to pitch and roll

#### HOTAS

- Left stick to steer
- `DPadUp` and `DPadDown` to throttle up and down respectively

#### Keyboard

- `WASDQE` to steer
- `PgUp` and `PgDown` to throttle up and down respectively

## Troubleshooting

### Broken Git LFS on Codeberg

```bash
Error downloading object: <some object>: Smudge error: Error downloading <some file path>: [<some commit>] Not Found: [404] Not Found
```

```bash
GIT_LFS_SKIP_SMUDGE=1 git reset --hard HEAD
git clean -fd
git lfs pull # You might need to download git-lfs first
```

## Contributing

PRs welcome.

If you have any problems regarding this repository, please report them as an
issue. Thank you!
