Normals From Shading
====================

Creates a normal map from a set of three or more images of
an object. Images must be the same resolution, have the
same perspective, and be lit from different angles.

Example
-------

### Inputs, Output, and Colorless Render

![image](https://raw.githubusercontent.com/Kenkron/normals_from_shading/refs/heads/main/sample_output/normal_map_demo.webp)

### Animations

[Colorless](https://raw.githubusercontent.com/Kenkron/normals_from_shading/refs/heads/main/sample_output/tile_untextured_small.mp4)

[With Color](https://raw.githubusercontent.com/Kenkron/normals_from_shading/refs/heads/main/sample_output/tile_untextured_small.mp4)

Usage
-----

    normals_from_shading [filename...]

The output will be stored in normal_map.png, and will be
in linear colorspace, not sRGB.

Methodology
-----------

This algorithm primarily works off the principle that
most light in most scenes is diffuse, so its intensity can
be computed with a simple dot product of the normal and
lighting direction.

In order to approximate both the lighting direction, and
the normal map, while being provided neither. An initial
normal map is created that is roughly domed, bending out
towards the edges.

Lighting directions are then computed via a least squared
solution for the given intensity. Since the dot product
operation is communicative, the lighting directions can
then be used to create a normal map using the same method.
This process can be repeated.

Because the lights and normals are only computed relative
to each other, its possible for them to drift such that
the normal map is greatly skewed in one direction. To fix
this, after each normal calculation, the normals are all
rotated together such that their average points directly
upwards along the z axis.

Finally, the result often bends outwards, akin to the dome
created initially. This can be due to the fact that the
diffuse lighting calculation assumes directional light,
while in reality, most scenes must use point light
sources, which provide more light to things that are
closer. To compensate for this, the image is assumed to be
flat in general, and an approximate normal is generated
for the area around each corner. These corners are then
rotated so that the approximate is facing upwards, and
every other point is rotated based on a linear
interpolation of the four corners.

Limitations
-----------

In diffuse lighting, a concave surface lit from one
direction appears identical to a convex surface lit from
the opposite direction, meaning there is an inherant
ambiguity, and all of the normals might be flipped
along the x/y axis. This would make bumps look like
dents, and vice-versa.

Because the initial normal map is convex, the algorithm
will tend to be convex overe all. This is often correct,
but not always.
