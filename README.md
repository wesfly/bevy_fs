# bevy-fs

This is a little flight sim made in Bevy.

![](Screenshot.png)

## Features

- Support for both keyboard and gamepad, but gamepad is recommended
- A display for throttle and altitude
- Procedural terrain
- Loading data from GLTF with custom properties
- 3D cockpit with clickable buttons
- Collisions with `avian3d`
- Water with screen space reflections
- Screen space ambient occlusion
- FPS counter in when in `Debug`
- A settings file (settings.json)

## Installation

```sh
git clone https://codeberg.org/wesfly/bevy-fs.git
git lfs pull # Pull the big files
```

If you have Rust installed, use `cargo run (--release)` to run the program.
If not, you can install it here: https://rust-lang.org/tools/install/

## Controls

To switch between gamepad and keyboard, manipulate the `gamepad -> enabled` field in settings.json.

### Gamepad

- Left stick to steer
- `DPadUp` and `DPadDown` to throttle up and down respectively
- `RMB + drag` to orbit camera
- `C` to switch the camera view
- Mouse wheel to zoom camera
- `R` to reset camera

### Keyboard

- `WASDQE` to steer
- `PgUp` and `PgDown` to throttle up and down respectively
- `RMB + drag` to orbit camera
- `C` to switch the camera view
- Mouse wheel to zoom camera
- `R` to reset camera
