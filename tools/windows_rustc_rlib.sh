#!/usr/bin/env bash
# iroh 1.5 also requests a cdylib when used as a Rust dependency. MinGW
# cannot export that unused DLL's >65,535 symbols. Keep its Rust library.
set -euo pipefail
rustc=$1
shift
args=("$@")
iroh=0
mingw=0
for ((i=0; i+1<${#args[@]}; i++)); do
    if [[ ${args[i]} == --crate-name && ${args[i+1]} == iroh ]]; then iroh=1; fi
    if [[ ${args[i]} == --target && ${args[i+1]} == x86_64-pc-windows-gnu ]]; then mingw=1; fi
done
if ((iroh && mingw)); then
    output=()
    for ((i=0; i<${#args[@]}; i++)); do
        if [[ ${args[i]} == --crate-type && ${args[i+1]} == cdylib ]]; then
            i=$((i+1))
        else
            output+=("${args[i]}")
        fi
    done
    exec "$rustc" "${output[@]}"
fi
exec "$rustc" "$@"
