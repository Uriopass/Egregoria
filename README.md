![Screenshot of scale](resources/screen1.png)

[![Build status](https://github.com/Uriopass/Scale/workflows/rust-build/badge.svg)](#)

# Scale

Scale is a simulation of modern day society, from the bottom-up. 
Each individual has its own thought model, meaning every action has its importance and influences the environment.  
Scale is not a video game, but rather a live artwork. The world itself won't be generated or created by the user, but is part of the project.  
That way, the focus is on the world itself and not on the tools to build it. 

## How ?  
This is of course very ambitious, so a minimal viable product will be made where features are increasingly added.

The first [milestone](https://github.com/Uriopass/Scale/projects/1) will introduce humans into the world. They will have their own homes and a workplace, traveling by foot or using the road system.

## Building the project

### Windows/Mac
I personally use Windows 10 and Mac OS 10.11 and it compiles fine once the rust toolchain is installed.
```bash
cargo run
```

### Ubuntu/Debian
There are a few libraries to install that some of my dependencies need:

```
sudo apt-get install libasound2-dev libudev-dev pkg-config
cargo run
```

A Github Action tests the builds on Ubuntu.

## Devblog

I will try to keep a blog about Scale's development [here](http://douady.paris/blog/index.html).


## Special thanks to

- [`@shika-blyat`](https://github.com/shika-blyat): For his work on the wgpu renderer
- [`@dabreegster`](https://github.com/dabreegster): For inspiration on the map model