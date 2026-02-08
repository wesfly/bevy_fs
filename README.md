# bevy-fs

This is a little flight sim made in Bevy.

![](Screenshot.png)

## Features

- Support for both keyboard and gamepad, but gamepad is recommended
- A display for throttle and altitude
- Terrain with real-world elevation data
- Loading data from GLTF with custom properties
- 3D cockpit with clickable buttons
- Collisions with `avian3d`
- Water with screen space reflections
- Screen space ambient occlusion
- FPS counter in when in `Debug`
- A settings file (settings.json)

## Installation

First you need to install Rust from [here](https://rust-lang.org/tools/install/), if you haven't already, and `git-lfs` by installing it with your package manager and then running `git lfs install`.

```sh
git clone https://codeberg.org/wesfly/bevy-fs.git
git lfs pull # Pull the big files
```

Use `cargo run --release` to run the program from the project root folder.

## Flying

To start flying, you need to go to the cockpit view and turn on the engine. The switch is located in the center console and is labelled `ENG`.
You should start flying immediatly. If that is not the case and the performance is bad, turn off the terrain collisions in the settings.

## Terrain

Terrain data will be stored in terrain.json after the first fetch. If you want a different location or resolution, you need to delete it and refetch it.

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
