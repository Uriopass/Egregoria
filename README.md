![Screenshot of Egregoria 1](assets/screen2.jpg)
![Screenshot of Egregoria 2](assets/screen3.jpg)
![Screenshot of Egregoria 3](assets/screen5.jpg)
![Screenshot of Egregoria 3](assets/screen1.jpg)

[![Build status](https://github.com/Uriopass/Egregoria/workflows/rust-build/badge.svg)](#)
[![Discord](https://img.shields.io/discord/709730057949544488?label=discord)](https://discord.gg/CAaZhUJ)

# Egregoria

Egregoria is an indie city builder, mostly inspired by Cities:Skylines.  
Each individual has its own thought model, meaning every action has its importance and influences the environment.  
Egregoria is focused on the socio-economical aspect of a city, with a logistics element.
The game is still in early development, but you can already play it and give some feedback through
[issues](https://github.com/Uriopass/Egregoria/issues) or on [discord](https://discord.gg/CAaZhUJ).  

By being open source, the hope is to get more people involved in the development of the game.  
Mod support is wanted but the design has not been found yet.

## How to play

A small tutorial is available on the [github wiki]((https://github.com/Uriopass/Egregoria/wiki/Introduction-Guide)) to get you started.

## Devblog  

I keep a blog about Egregoria's development [here](http://douady.paris/blog/index.html).

## Building the project

### Git LFS

This project uses Git LFS to track assets, therefore if you want to build your own copy you will need to install [Git LFS](https://git-lfs.github.com/).

Once installed, you should be able to clone the repo and fetch the assets:

```bash
git clone https://github.com/Uriopass/Egregoria
cd Egregoria
git lfs pull
```

### Windows/Mac
I personally use Windows 10 and Mac OS 10.11, and it compiles fine once the [rust toolchain is installed](https://www.rust-lang.org/tools/install).
```bash
cargo run --release
```

Don't forget to add the `--release` flag, otherwise the game will be very slow.  
Don't forget to pull the lfs files, otherwise the game will crash with a file not found error.

### Ubuntu/Debian on x11
There are a few libraries to install that some of my dependencies need:

```
sudo apt-get install libasound2-dev libudev-dev pkg-config libx11-dev
cargo run --release
```

Don't forget to add the `--release` flag, otherwise the game will be very slow.  
Don't forget to pull the lfs files, otherwise the game will crash with a file not found error.

A GitHub Action tests the builds on Ubuntu.

## Why Egregoria ?

An Egregor is an autonomous psychic entity made up of, and influencing, the thoughts of a group of people.  
It represents emergence at its purest form, where individuals come together to become a collective force.

## Credits

- [`@dabreegster`](https://github.com/dabreegster): For inspiration on the map model
- PBR Shaders are adapted from [LearnOpenGL](https://learnopengl.com/PBR/Theory)
