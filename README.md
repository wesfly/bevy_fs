# bevy_fs

This is a little flight sim made in Bevy.

![(Stored in Git LFS)](screenshots/1.png)
![(Stored in Git LFS)](screenshots/2.png)

[Discord server](https://discord.gg/epMBz5m2Ad)

## Highlights

- Support for both keyboard and gamepad, but gamepad is recommended
- Terrain with real-world elevation data and collisions
- Loading data from GLTF with custom properties
- 3D cockpit with clickable buttons
- Collisions with `avian3d`
- Water with screen space reflections
- A settings file (settings.json)

## Running `bevy_fs`

First you need to install Rust from [here](https://rust-lang.org/tools/install/), if you haven't already, and `git-lfs` by installing it with your package manager and then running `git lfs install`.

```sh
git clone https://github.com/wesfly/bevy_fs.git
cd bevy_fs
git lfs pull # Pull the big files
```

Use `cargo run --release` to run the program from the project root folder.

## Flying

### Helicopter

To start flying, you need to go to the cockpit view and turn on the engine. The switch is located in the center console and is labelled `ENG`.
If you now bring the throttle to 100%, you should start flying.

### Aeroplane

You will start in the air, you just need to throttle up.

## Settings

The settings are located in `settings.json` at the root of this project.

## Terrain

Terrain data will be stored in the `terrain_cache` folder after the first fetch (the game window will be unresponsive while fetching). To change the location or the resolution, change `terrain.coordinates` or `terrain.level_of_detail` in the settings. Note that the maximum level of detail is 14, in some regions even less.

## Controls

You can change input methods in the settings.

- `F3` to take a screenshot (output: `screenshots/user`)
- `RMB + drag` to orbit camera
- `C` to switch the camera view
- Mouse wheel to zoom camera
- `R` to reset camera
- `G` to raise or lower landing gear

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

## Contributions

Contributions are welcome.

If you have any problems regarding this repository, please report them as an issue. Thanks!
