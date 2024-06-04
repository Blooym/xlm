#!/bin/env bash

# Prevents launching twice.
if [ $1 == "run" ]; then sleep 1; exit; fi

tooldir="$(realpath "$(dirname "$0")")"

PATH=$PATH:$tooldir/xlcore $tooldir/xlcm launch --install-directory $tooldir/xlcore
