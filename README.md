# rpi-music-visualizer
Raspberry Pi Music Visualizer written in Rust 

## Roadmap

[x] Accept microphone input

[x] Plug in LEDs

[x] LED communication

[x] Battery setup

[x] Wiring diagram

[ ] How to wear it (Comfortably)

[ ] Improve visualizer data quality

[ ] More visualizers

# How to run on Raspberry Pi

mkdir /tmp/xdg
export XDG_RUNTIME_DIR=/tmp/xdg

sudo -E startx $(pwd)/target/debug/rpi-music-visualizer --visualizer power_circles --fake --screen hardware
sudo -E startx $(pwd)/target/release/rpi-music-visualizer --visualizer power_circles --fake --screen hardware
