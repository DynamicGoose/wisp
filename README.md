# Wisp
Wisp is the voxel-renderer for the Magma3D engine. It uses raymarching to render the voxels.
## Features
- [ ] raymarcing voxels with smooth min (very lightweight)
- [ ] materials/shading
- [ ] per-voxel global illumination
- [ ] reflections
- [ ] async reprojection
- [ ] VR-support
## How it works
Voxels in Wisp are represented as points. All voxels have the same radius and a smooth minimum function is used to interpolate between them. Wisp supports per-voxel global illumination. More detailed reflections are also supported for increased resolution of reflections. Async reprojection is used to make Wisp able to run on almost any hardware. 
## How to use it
### Credits
- Vulkan
- ash-rs
