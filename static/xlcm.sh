#!/bin/env bash

# Prevents launching twice.
if [ $1 == "run" ]; then sleep 1; exit; fi

tooldir="$(realpath "$(dirname "$0")")"

XL_SECRET_PROVIDER=FILE PATH=$PATH:$tooldir/xlcore $tooldir/xlcm launch --install-directory $tooldir/xlcore
