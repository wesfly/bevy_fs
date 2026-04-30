# bevy_fs

This is a little flight sim made in Bevy.

![(Stored in Git LFS)](screenshots/1.png)
![(Stored in Git LFS)](screenshots/2.png)

[Discord server](https://discord.gg/epMBz5m2Ad)

## Highlights

- Support for both keyboard and gamepad, but gamepad is recommended
- Terrain with real-world elevation data and collisions
- Loading data from GLTF with custom properties
- 3D cockpit
- Collisions using `avian3d`
- Water with screen space reflections
- A settings file (settings.json)

## Running `bevy_fs`

### Prerequisites

- **Rust toolchain:** [Install Rust](https://rust-lang.org/tools/install)
- **Git LFS:** Used for large file assets.
  - [Git LFS install guide](https://git-lfs.github.com/)

### Installation steps

1. Clone the repository and get large assets
   ```sh
   git clone https://github.com/wesfly/bevy_fs.git
   cd bevy_fs
   git lfs pull
   ```

2. Build and run the project
   ```sh
   cargo run --release
   ```

   This will build and launch bevy_fs. The first startup fetches terrain data and can take some time.

## Flying

### Helicopter

You can start the engines by pressing `M`. If you now bring the throttle to 100%, you will start flying.

### Aeroplane

You will start in the air, you just need to throttle up.

## Settings

The settings are located in `settings.json` at the root of this project.

## Terrain

Terrain data will be stored in the `terrain_cache` folder after the first fetch (the game window will be unresponsive while fetching). To change the coordinates or the resolution, change `terrain.coordinates` or `terrain.level_of_detail` in the settings. Note that the general maximum level of detail is 15, but in most regions 14 or even less. Here is an interactive map for different resolutions: https://mapterhorn.com/coverage/

Note that high resolutions will result in high RAM usage, to mitigate this issue, turn down the maximum render distance in the settings.

You can control the sun position using `HJKL`.

## Controls

You can change input methods in the settings.

- `F3` to take a screenshot (output: `screenshots/user`)
- `RMB + drag` to orbit camera
- `C` to switch the camera view
- Mouse wheel to zoom camera
- `R` to reset camera
- `G` to raise or lower landing gear
- `Z` to brake
- `=` to toggle parking brake
- `1` for position lights
- `2` for strobe lights
- `3` for formation lights
- `4` for anti-collision lights

### Some additional bindings for different configs

#### Controller

- Left stick to throttle and yaw
- Right stick to pitch and roll

#### HOTAS

- Left stick to steer
- `DPadUp` and `DPadDown` to throttle up and down respectively

#### Keyboard

- `WASDQE` to steer
- `PgUp` and `PgDown` to throttle up and down respectively

---

## Contributing

PRs welcome.

If you have any problems regarding this repository, please report them as an issue. Thanks!
