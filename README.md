![Screenshot of Egregoria 1](assets/screen2.jpg)
![Screenshot of Egregoria 2](assets/screen3.jpg)
![Screenshot of Egregoria 3](assets/screen4.jpg)

[![Build status](https://github.com/Uriopass/Egregoria/workflows/rust-build/badge.svg)](#)
[![Discord](https://img.shields.io/discord/709730057949544488?label=discord)](https://discord.gg/CAaZhUJ)

# Egregoria

Egregoria is a simulation of modern day society, from the bottom-up. 
Each individual has its own thought model, meaning every action has its importance and influences the environment.  

#### Why Egregoria ?

An Egregor is an autonomous psychic entity made up of, and influencing, the thoughts of a group of people.  
It represents emergence at its purest form, where individuals come together to become a collective force.  

## How ?  

This is of course very ambitious, so a minimal viable product will be made where features are increasingly added.

The first [milestone](https://github.com/Uriopass/Egregoria/projects/1) will introduce humans into the world. They will have their own homes and a workplace, traveling by foot or using the road system.

As of January 2021, this milestone is now achieved :-) I'm focusing on making more interesting interactions now, but I don't have the next milestone well-defined yet.

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
cargo run
```

### Ubuntu/Debian on x11
There are a few libraries to install that some of my dependencies need:

```
sudo apt-get install libasound2-dev libudev-dev pkg-config libx11-dev
cargo run
```

A GitHub Action tests the builds on Ubuntu.



## Special thanks to

- [`@shika-blyat`](https://github.com/shika-blyat): For his work on the wgpu renderer
- [`@dabreegster`](https://github.com/dabreegster): For inspiration on the map model
- [`@kosuru`](https://soundcloud.com/kosuru-980687955): For his wonderful ambient music
