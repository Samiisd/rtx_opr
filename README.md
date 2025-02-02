# Introduction
test
Minecraft-like game with infinite maps rendered using path-tracing techniques and NVIDIA's RTX technology (Vulkan).
This project is a rewrite of a [project we did with OpenGL and raytracing without hardware acceleration](https://github.com/AudranDoublet/glopr).

This project is originally an assignment for our **advanced introduction to OpenGL** course, nothing serious.

The project is entirely written in Rust and GLSL (for Vulkan), using [Ash](https://github.com/MaikKlein/ash) for Vulkan bindings.

## Samples

Images are more talkative than long texts, so here are few samples of what our engines are capable of:

<img src="/data/samples/1.png" width="384" height="216"> <img src="/data/samples/4.png" width="384" height="216">
<img src="/data/samples/2.png" width="384" height="216"> <img src="/data/samples/5.png" width="384" height="216">
<img src="/data/samples/3.png" width="384" height="216"> <img src="/data/samples/6.png" width="384" height="216">

Here is a demonstration video that shows some scenes captured from the game (*you have to click on the image below*):

[![Demonstration Video](http://img.youtube.com/vi/OUKqvlPS1nk/0.jpg)](http://www.youtube.com/watch?v=OUKqvlPS1nk "Demonstration Video")

## Disclaimer

We made every resources used by the project except for the Textures that we took from the internet, a **big thanks** to people who contributed to those **beautiful** and **free** textures! :heart:

This project was written in a rush, there may be some mistakes and imperfections, although we put a lot of effort and the results are satisfactory.

# Functionalities

RTX-GlOPR is a simple Minecraft-like game, which use path tracing algorithms for rendering.

We implemented a map generator:
* generation of different biomes with a global coherence (big oceans, warm / cold / temperate zones, beaches ...)
* generation of columns with coherent size (using perlin noise) and smooth transition between biomes (ex: between plains and mountains)
* generation of decorations: flowers, cactus, various trees, grass, ...

We implemented a minimalistic game engine using AABB collisions.

Finally, we implemented a rendering pipeline in Vulkan with a few steps:

 <img src="/data/samples/schema.png">

**Initial ray**, which is basically a ray casting for each screen pixel, to known the hitten object (and store the normal, material properties, material color, ...).
It can be noted that this could be replaced by traditional rasterization, which would probably be faster.

**Procedural skybox** is computed using Rayleigh diffusion, when a ray doesn't hit an object (initial ray and specular).

**Shadow ray** which just sends a ray from hitten point in initial ray towards the direction of the sun. This phase only handles sun shadows, not block lights.
In fact, these lights are not managed in raytracing but we "hope" to simply touch them with the path tracing phases.

The interest of this implementation is the performance: Minecraft is a game where light sources can be counted by hundreds or thousands, and it would be unthinkable to throw so many rays.

**Diffuse reflections** are implemented using  **Disney-Burley [1]** model, resulting in high quality results with simple textures, easily taking into account the roughness, metalness and specularity of a material.

**Specular reflections** are implemented using a **Microfacet-based BRDF [2]**, describing a material surface realistically as many micro-facets that deviate the direction of the reflected light rays.

This is done by calculating:
- Fresnel term: reflectance for a given normal et light direction
- **D** microfacet distribution term: distribution of surface normals for a given microfacet (how the microfacet normal is distributd around a given direction), e.g., [Blinn-Phong distribution](https://en.wikipedia.org/wiki/Blinn%E2%80%93Phong_reflection_model). *We use the GGX distribution*.
- **G** shadowing-masking term: probability that the microfacet is visible from the incoming/outgoing direction. *We use the Smith G*.

Roughly, by multiplying those three terms together (normalizing by other stuffs), we obtain the amount of light reflected by a given microfacet.

**Refraction** step sends rays through transparents surfaces (glasses and water). This step isn't done through path tracing for performance reasons.
A big flaw of this approach is that diffuse and specular lightning won't be seen behind a transparent surface, nevertheless it's still provides good results.

**Denoising** is extremly important for real-time path tracing. One of the state-of-the-art approach for this is SVGG and A-SVGF algorithms, which includes a temporal filtering
(accumulation of samples between each frames, using a reprojection) and a spatial filtering (à trous filter).

Our version implements only the temporal filter, because the rest of our graphics pipeline is not optimized enough, and we would probably have lost too much FPS. We didn't have time to optimize more for our assignment.

**Shadow maps** are used as part of the god rays rendering. It's basically implemented as a depth map of the scene from the sun view (orthographic projection). As for the initial ray, we implemented
this step using RTX, but using rasterization would probably have been better.

**God rays (atmospheric light scattering) [3]** are due to small particles in the light-transmitting medium. To simulate the effect we samples some air points between the camera and the initial hit point. For each of
these points, we need to know whether or not their are shadowed. If they are, the point doesn't provide illumination, otherwise it does. The shadowmap is then very useful, because it wouldn't be feasible to launch so many shadow rays.

This part could benefit from a less basic implementation of shadowmaps.

Note: in fact, for performance reasons, god rays are computed with a resolution halved, and then upsampled with a bilateral filter.

**Reconstruction** produce the final image by combining all lights sources using Fresnel's law: direct illumination, denoised diffuse, denoised specular, refraction and god rays.

# Performances

Having an optimized code wasn't the main focus of this project, however, the performances are still correct.

On an **NVIDIA RTX 3060 Ti**, with a **1080P** resolution, the performances are approximately:


|                                                                CONFIGURATION                                                               |  FPS |
|:------------------------------------------------------------------------------------------------------------------------------------------:|:----:|
| Ray Tracing: diffuse + shadow only                                                                                                         | >144 |
| Path Tracing: 3 samples per pixel, 4 bounces                                                                                               | ~80  |
| Path tracing: 1 sample per pixel, 4 bounces + Direct/Indirect specular + God rays (shadowmap 4k resolution) + Refraction (up to 8 bounces) | ~25  |

This is way better than the performances we got on our initial project: [Path-traced Minecraft using OpenGL compute shaders](https://github.com/AudranDoublet/glopr) where having optimized code was one of our main targets. Here are the performances on OpenGL with compute shaders (and less features):

|      Graphic Card      | Ray Tracing 540P (FPS) | Path Tracing 540P (FPS)  *diffuse only, up to 5 bounces* |
|:----------------------:|:----------------------:|:--------------------------------------------------------:|
| Intel UHD Graphics 620 |           30           |                            10                            |
| Nvidia GTX 1050        |           60           |                            20                            |
| Nvidia RTX 2060        |           200          |                            60                            |

 
# Compile me

1. Install rustup: https://rustup.rs/
2. Run: rustup install nightly
3. In this directory, run: rustup override set nightly
4. Run cargo build --release

# Usage

Example:
```
cargo run --release -- game \
          --view-distance 10 \
          --layout us
```

Main game parameters:
* view-distance: number of chunks seen in each direction
* layout: fr or us, main keyboard mapping
* world: world path to load
* flat: if presents, the map is flat
* seed: (number) world random seed; by default 0

# In game options

**Move** Z,Q,S,D (fr) or W,A,S,D (us)

**Break a block** Left click

**Place a block** Right click

**1,2,3,4,5,6,7,8,9,0** display pathtracing debug buffers

**Alt+1,2,3,4** change block in hand

**Toggle ambient light** L

**Set sun position** K

**Do daylight cycle** N

**Sneak** Left-shift (the player will be slower but won't fall)

**Toggle sprint** Left-control (the player will be faster)

**Toggle fly mode** Double click on space

# References

NVIDIA offers resources on RTX (including the official version of Minecraft RTX developed by them), which helped us a lot:
* [NVIDIA's Metalness-Emissivness-Roughness textures for Minecraft](https://www.nvidia.com/en-us/geforce/guides/minecraft-rtx-texturing-guide/)
* [technical presentation by members of the Minecraft RTX's team, which presents a pipeline similar to the one we use](https://www.youtube.com/watch?v=mDlmQYHApBU)
* [Q2RTX, the RTX version of Quake2 made by NVIDIA](https://github.com/NVIDIA/Q2RTX)
* [RTX tutorial for DX12](https://developer.nvidia.com/rtx/raytracing/dxr/DX12-Raytracing-tutorial-Part-1)


[1] [Burley, Brent, and Walt Disney Animation Studios. "Physically-based shading at disney." ACM SIGGRAPH. Vol. 2012. vol. 2012, 2012.](https://media.disneyanimation.com/uploads/production/publication_asset/48/asset/s2012_pbs_disney_brdf_notes_v3.pdf)

[2] [Walter, Bruce, et al. "Microfacet Models for Refraction through Rough Surfaces." Rendering techniques 2007 (2007): 18th.](http://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf)

[3] [GDC 2016: Fast, Flexible, Physically-Based Volumetric Light Scattering (presented by NVIDIA)](https://www.gdcvault.com/play/1023519/Fast-Flexible-Physically-Based-Volumetric)
