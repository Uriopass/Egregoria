This document describes the high-level architecture of Egregoria. If you want to contribute to the project, you are in the right place!

# Bird's Eye View

Egregoria's core is a fixed-time "tick" update and uses an [ECS](https://en.wikipedia.org/wiki/Entity_component_system).
The whole simulation is advanced by one step depending on the current state, it is pure and deterministic.

The simulation is composed of many systems which acts upon the different entities and singletons.
For example, there's the `kinematics` system which says `position += velocity * DT`.
There's also the `market_update` which updates the markets and determines which trades are to be made.

To handle user interactions, Egregoria uses a Server-Client model.
This ensures Egregoria's state cannot be easily corrupted and enables [Deterministic Lockstep](https://gafferongames.com/post/deterministic_lockstep/) networking.
The [`WorldCommand`](https://github.com/Uriopass/Egregoria/blob/master/egregoria/src/engine_interaction.rs#L32) enum encodes theses possible mutations.

For rendering+audio, all the render state is specifically NOT contained along the simulated entities.
Instead of saying "this road has this mesh" with the rest of the `Road`'s definition, it is entirely determined on the rendering side and cached.
Simplified, instead of a `mesh: RoadMesh,` field in the `Road` struct, the renderer would contain a `HashMap<RoadID, RoadMesh>`.

Decoupling the rendering from the simulation really helps to separate concerns and keep related invariants at the same place in the code.

# Codemap

![](assets/crates_architecture.jpg)

This is a codemap to showcase the different crate's usages and modules within some crates.

Approximately sorted by importance.

## `map_model`

Contains all the map related data. It contains the data about:
 - Roads/intersections/lanes/lots
 - Buildings
 - Terrain
 - Trees

It is only raw data and operators (e.g. build this road here), but it doesn't contain any logic per se.

## `egregoria`

The main crate, which contains all the simulation logic.
It is itself composed of the following subsystems:

### `egregoria/economy`

Everything related to the market. Doesn't contain the economic actors.

### `egregoria/map_dynamic`

This contains all the dynamism around the map, like the pathfinding, routing, parking and itinerary systems.

### `egregoria/pedestrians`

This module handles pedestrians, that is bodies walking around the world.

### `egregoria/physics`

This module handles everything related to physics, that is the velocity systems but also the spatial structures.

### `egregoria/souls`

This module contains all the AI related to the companies and the humans in the world.  
This is where companies decide to employ people, and where people decide to buy some bread.

### `egregoria/vehicles`

This module handles vehicles, it contains all the complex rules around traffic and how to handle intersections.  

## `wgpu_engine`

This crate contains almost all of the wgpu related code. That is, all the low-level graphics stuff like connecting to the gpu, setting up pipelines, sending textures and render meshes.  
All the shaders are in the assets/shaders folder.

It is a simple Forward renderer with the following passes:
 - Opaque depth prepass
 - SSAO with depth reconstruction using the prepass
 - Shadow map pass for the sun
 - Main forward/color pass
 - UI pass

It does not use PBR, only basic albedo textures. Objects are loaded using the gltf format.

## `native_app`

This crate is the binary for desktop applications. It ties together ui+rendering+audio+simulation.
It also contains all the rendering state like the meshes and terrain systems.

## `networking`

This crate is standalone and contains all the client+server code for deterministic lockstep.  
It only takes in a world and world commands and synchronizes them between clients.

It implements basic connection, authentication, catching up mechanism and input handling.

See [this blog post](http://douady.paris/blog/egregoria_8.html) for more details.

## `flat_spatial`

This is a forked and specialized version of this crate for Egregoria, but a more general version and description can be found on the project's page [here](https://github.com/Uriopass/flat_spatial).  

## `geom`

As most of Rust's math libraries lack some methods or are far too generic, I prefered to just recode one for my usecase. It contains the basic vector types, some matrix math and a lot of geometry primitives like `Circle`, `Segment`, `Polyline` and `Polygon`.

## `headless`

This crate is a binary to be used as a server. It doesn't contain any ui/rendering code, only the simulation. 

## `common`

Some tools shared between the crates.

# I want to contribute to...

This section talks about "where to start with" if you want to contribute about a specific aspect.  
Sub-sections are not in any particular order.

When you have decided what you would like to contribute, please come chat about your needs and wishes in [the official discord](https://discord.gg/CAaZhUJ) or create a new issue. This helps with coordination.

## Audio/Art

Egregoria uses the GLTF format for meshes and ogg for audio files.  
At the moment, the renderer is pretty limited as it only supports one material per mesh.  
Modding support is nonexistent so everything goes through static links in the code.  

## UI

All the UI related code is in the `native_app` crate, more specifically in the `gui` module. It contains code for the road editor, building placement, inspect window, top gui and others.
topgui.rs contains most of the imgui code.

The other files follow one file = one system.

## Simulation/Gameplay

All the simulation code is in the `egregoria` crate. The different modules of this crate are explained in the codemap.  
Try to keep the different aspects of the simulation decoupled so that it is easier to reason about.

## 3D Graphics

Most of the 3D graphics code is in the `wgpu_engine` crate. It uses a basic forward renderer.  
Some notable features you could add would be cascaded shadow maps, PBR and clustered lighting.

As the name hints, it uses wgpu as the rendering backend which is multi-backend (vulkan, dx12 and metal).
