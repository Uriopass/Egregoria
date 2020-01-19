[![Build status](https://github.com/Uriopass/Scale/workflows/rust-build/badge.svg)](#)

# Scale

Scale is a simulation of modern day society, from the bottom-up. 
Each individual has its own thought model, meaning every action has its importance and influences the environment.  
Scale is not a video game, but more of a zero-player game (like the game of life), in which there might be a small and very limited interaction.  
Thus, a lot of power could be invested into making Scale a unique persistent world,
where attention to detail is achievable. 

## How ?  
This is of course very ambitious, so inspired by the WASM project,
a minimal viable product should be made where features are increasingly added.

The first [milestone](https://github.com/Uriopass/Scale/projects/1) is to be able to bring humans to the world. They would have their own homes and a workplace, for which they could use their cars or simply walk to go there.  

## Building the project

### Windows/Mac
I personally use Windows 10 and Mac OS 10.11, and it works by simply compiling the project with the marvelous cargo.
```bash
cargo run
```

### Ubuntu/(Others?)
There are a few libraries to install, which some of my dependencies need:

```
sudo apt-get install libasound2-dev libudev-dev pkg-config
cargo run
```

## Devblog

I will try to keep a blog about Scale updates [here](http://douady.paris/blog/index.html).
