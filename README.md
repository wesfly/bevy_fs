# bevy-fs

This is a little flight sim made in bevy. Currently it doesn't do much, but I plan to make some more commits.
The game has support for both keyboard and gamepad input.

![](Screenshot.png)

### Controls

To switch between gamepad and keyboard, manipulate the `gamepad_enabled` field in settings.json.

#### If gamepad_enabled is true

- Left stick to steer
- `DPadUp` and `DPadDown` to throttle up and down respectively
- `RMB + drag` to orbit camera
- Mouse wheel to zoom camera
- `R` to reset camera

#### If gamepad_enabled is false

- `WASDQE` to steer
- `PgUp` and `PgDown` to throttle up and down respectively
- `RMB + drag` to orbit camera
- Mouse wheel to zoom camera
- `R` to reset camera
