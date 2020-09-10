#!/bin/bash
mkdir compiled_shaders
cd shaders || exit
for file in *; do
  glslangValidator -V -o "../compiled_shaders/$file.spirv" "$file"
done