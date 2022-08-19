#!/bin/sh
mkdir build
cp -r assets build
rm -f build/screen*.png
rm -f build/crate_architecture.png
cp target/release/native_app build/egregoria_bin
mv build egregoria_build
tar -czvf goria.tar.gz egregoria_build
rm -rf egregoria_build
