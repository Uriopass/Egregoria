#!/bin/bash
mkdir -p compiled_shaders
cd shaders || exit
for file in *.frag; do
  glslc -g -O -o "../compiled_shaders/$file.spirv" "$file" &
done
for file in *.vert; do
  glslc -g -O -o "../compiled_shaders/$file.spirv" "$file" &
done
wait
