#!/bin/bash -e

project_dir=$(dirname "$0")

sudo -E startx "${project_dir}/target/release/rpi-music-visualizer" $@
