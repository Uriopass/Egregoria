#!/bin/bash
mkdir -p compiled_shaders
cd shaders || exit
for file in *; do
  glslangValidator -V -o "../compiled_shaders/$file.spirv" "$file" &
done
sleep 0.3
