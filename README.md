# rpi-music-visualizer
Raspberry Pi Music Visualizer written in Rust 

## Roadmap

[x] Accept microphone input

[ ] Plug in LEDs

[x] LED communication

[ ] Battery setup

[ ] Wiring diagram

[ ] How to wear it (Comfortably)

[ ] Improve visualizer data quality

[ ] More visualizers

## Visualizer Ideas:
* Lip-syncing Smiley Face visualizer


# How to run on Raspberry Pi

mkdir /tmp/xdg
export XDG_RUNTIME_DIR=/tmp/xdg

sudo -E startx $(pwd)/target/debug/rpi-music-visualizer --visualizer smiley --fake --screen hardware
