#!/bin/env bash

# Prevents launching twice.
if [ $1 == "run" ]; then exit; fi

tooldir="$(realpath "$(dirname "$0")")"

export XL_SECRET_PROVIDER=FILE
$tooldir/xlcm launch --install-directory $tooldir/xlcore
