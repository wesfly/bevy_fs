# bevy-fs

This is a little flight sim made in bevy. Currently it doesn't do much, but I plan to make some more commits.
The game has support for both keyboard and gamepad input.

![](Screenshot.png)

### Controls

To switch between gamepad and keyboard, manipulate the `ENABLE_GAMEPAD` constant in main.rs

#### If ENABLE_GAMEPAD is true

- Left stick to steer
- `RMB + drag` to orbit camera
- `R` to reset camera
- `DPadUp` and `DPadDown` to throttle up and down respectively

#### If ENABLE_GAMEPAD is false

- `WASDQE` to steer
- `RMB + drag` to orbit camera
- `R` to reset camera
- `PgUp` and `PgDown` to throttle up and down respectively
