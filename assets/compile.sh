#!/bin/bash
mkdir -p compiled_shaders
cd shaders || exit
for file in *; do
  glslc -O -o "../compiled_shaders/$file.spirv" "$file" &
done
sleep 0.3
