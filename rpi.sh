#!/bin/bash -e

project_dir=$(dirname "$0")

mkdir -p /tmp/xdg
export XDG_RUNTIME_DIR=/tmp/xdg

sudo -E startx "${project_dir}/target/release/rpi-music-visualizer" $@
