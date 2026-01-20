# bevy-fs

This is a little flight sim made in Bevy.

![](Screenshot.png)

## Features

- Support for both keyboard and gamepad, but gamepad is recommended
- An altitude display
- 3D cockpit with clickable buttons
- Laggy collisions that make low FPS (I hope this will fix itself in the next `avian_3d` version, there is a weird bug since Bevy 0.18.0)
- Water with screen space reflections
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

To switch between gamepad and keyboard, manipulate the `gamepad_enabled` field in settings.json.

### If gamepad_enabled is true

- Left stick to steer
- `DPadUp` and `DPadDown` to throttle up and down respectively
- `RMB + drag` to orbit camera
- `C` to switch the camera view
- Mouse wheel to zoom camera
- `R` to reset camera

### If gamepad_enabled is false

- `WASDQE` to steer
- `PgUp` and `PgDown` to throttle up and down respectively
- `RMB + drag` to orbit camera
- `C` to switch the camera view
- Mouse wheel to zoom camera
- `R` to reset camera
