#!/usr/bin/env bash

watchman-make -p src/*.rs --run ./scripts/build_for_web.sh &
python -m http.server -d web