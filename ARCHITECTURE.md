This document describes the high-level architecture of Egregoria. If you want to contribute to the project, you are in the right place!

# Bird's Eye View

Egregoria's core is a fixed-time "tick" update.
The whole simulation is advanced by one step depending on the current state, it is pure and deterministic.

The simulation is composed of many systems which acts upon the different entities and singletons.
There's also the `market_update` which updates the markets and determines which trades are to be made.

To handle user interactions, Egregoria uses a Server-Client model.
This ensures Egregoria's state cannot be easily corrupted and enables [Deterministic Lockstep](https://gafferongames.com/post/deterministic_lockstep/) networking.
The [`WorldCommand`](https://github.com/Uriopass/Egregoria/blob/master/simulation/src/engine_interaction.rs#L32) enum encodes theses possible mutations.

For rendering+audio, all the render state is specifically NOT contained along the simulated entities.
Instead of saying "this road has this mesh" with the rest of the `Road`'s definition, it is entirely determined on the rendering side and cached.
Simplified, instead of a `mesh: RoadMesh,` field in the `Road` struct, the renderer would contain a `HashMap<RoadID, RoadMesh>`.

Decoupling the rendering from the simulation really helps to separate concerns and keep related invariants at the same place in the code.

# Codemap

![](assets/crates_architecture.jpg)

This is a (not up-to-date) codemap to showcase the different crate's usages and modules within some crates.

Approximately sorted by importance.

## `simulation`

The main crate, which contains all the simulation logic.
It is itself composed of the following subsystems:

### `simulation/economy`

Everything related to the market. Doesn't contain the economic actors.

## `simulation/map`

Contains all the map related data. It contains the data about:
- Roads/intersections/lanes/lots
- Buildings
- Terrain
- Trees

It is only raw data and operators (e.g. build this road here), but it doesn't contain any simulation logic per se.

### `simulation/map_dynamic`

This contains all the dynamism around the map, like the pathfinding, routing, parking and itinerary systems.

### `simulation/transportation`

This module handles vehicles and pedestrians, it contains all the complex rules around traffic and how to handle intersections.

### `simulation/souls`

This module contains all the AI related to the companies and the humans in the world.  
This is where companies decide to employ people, and where people decide to buy some bread.

## `engine`

This crate contains almost all of the wgpu related code. That is, all the low-level graphics stuff like connecting to the gpu, setting up pipelines, sending textures and render meshes.  
All the shaders are in the assets/shaders folder.

It is a simple Forward renderer with the following passes:
 - Opaque depth prepass
 - SSAO with depth reconstruction using the prepass
 - Cascaded shadow map pass for the sun
 - Main forward/color pass using Physically Based Rendering (PBR)
 - UI pass

Objects are loaded using the gltf format.

## `native_app`

This crate is the binary for desktop applications. It ties together ui+rendering+audio+simulation.
It also contains all the rendering state like the meshes and terrain systems.

## `networking`

This crate is standalone and contains all the client+server code for deterministic lockstep.  
It only takes in a world and world commands and synchronizes them between clients.

It implements basic connection, authentication, catching up mechanism and input handling.

See [this blog post](http://douady.paris/blog/egregoria_8.html) for more details.


## `geom`

As most of Rust's math libraries lack some methods or are far too generic, I preferred to just recode one for my usecase. It contains the basic vector types, some matrix math and a lot of geometry primitives like `Circle`, `Segment`, `Polyline` and `Polygon`.

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
The assets are in the `assets` folder.
You can change assets for companies in the `assets/companies.json` file.

A dedicated Asset Manager is in construction to help the process.

## UI

All the UI related code is in the `native_app` crate, more specifically in the `gui` module. It contains code for the road editor, building placement, inspect window, top gui and others.
`topgui.rs` contains most of the egui code.

## Simulation/Gameplay

All the simulation code is in the `simulation` crate. The different modules of this crate are explained in the codemap.  
Try to keep the different aspects of the simulation decoupled so that it is easier to reason about.

## 3D Graphics

Most of the 3D graphics code is in the `engine` crate.
It contains all the low-level graphics code like connecting to the gpu, setting up pipelines, sending textures and render meshes.
It is a forward renderer with SSAO, cascaded shadow maps, PBR and world-space lights (not screen space clustered).

It uses wgpu for the rendering backend which is multi-backend (vulkan, dx12 and metal).

# Economy

The economy is a very central part of this city building game. A model must be chosen to know how companies and individuals engage in trade.

A simple "graph model" ala Factorio where commodities are directly moved from producer to consumer works well if the system is closed. As we want external trading we need some way of introducing scarcity to regulate who gets what.

I think people are more interested in capitalist models where objects are priced based on a free market since that's the world we live in.  
However we need to make sure that the model does not go out of control. If the farms go bankrupt in real life, the government will give subsidies and try to keep the economy running instead of making everyone starve to death.

I am not really well versed in economics so I'm going to try to do something interesting gameplay wise while still trying to keep some form of realism.

## External trade based market

The whole city only has one capital: The money of the player is supposed to be "equally distributed" among its city.

If a bakery wants to buy flour, it either tries to fetch it from a local flour industry or it gets it from external trade.

Getting through local economy doesn't involve money, it is basically "free".
Getting through the external trade removes money from the government capital.

If a flour company makes too much flour, it can sell to the external trading platform, this surplus is where you get money to invest in the rest of the city.

Theoretically, if you produce all your goods locally you don't need to pay for anything and money is not an issue,
but big projects (big buildings, roads, electricy...) involve external workers that do cost money.

That way the player does manage "everything", but you still have incentive to produce locally.

A path for implementation:

The external market has fixed (maybe varying in later time) trading costs.

Local markets can trade, but surplus wins money and shortage costs money.

Buildings things usually costs less than buying from trade (because of the transportation costs) and that's how you can expand.

External value can be calculated from the Goods graph.

This is a good "simple" model than stills instills progress and is quite easy to simulate.
Basically a hybrid of factorio and free market, but simple because there is only one economic actor.

It does seem less interesting from the "citizens" point of view that don't need to make individual decisions.
However each citizen thinking is just too hard for me, maybe later.