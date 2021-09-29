## 1.4.9
### Bugfixes
- Fixed another bug where walls to the right of a token could become see-through if one of it's end points was hidden due to the token's vision angle

## 1.4.8
### Bugfixes
- Fixed a bug where walls to the right of a token could become see-through if one of it's end points was hidden due to the token's vision angle


## 1.4.7
### Bugfixes
- Fixed a bug where a small vision angle could allow the vision to penetrate through walls


## 1.4.6
### Bugfixes
- Fixed a bug that could cause walls to become see-through if their walls aren't snapped to the grid (or the subgrid)


## 1.4.5
### New features
- Small speed improvement for sight calculation when using a restricted vision angle

### Bugfixes
- Fixed a bug that could cause the vision calculation to crash in specific wall constellations
- Fixed multiple bugs that could cause incorrectly calculated sight area when a restricted vision angle was being used (especially if that angle is bigger than 180°)


## 1.4.4
### Bugfixes
- The vision calculation algorithm no longer crashes when a token's vision radius is set to 0.


## 1.4.3
### Bugfixes
- Fixed a bug where the fog of war calclation could fail after switching between maps of vastly different sizes


## 1.4.2
### Bugfixes
- Fixed a bug where the background image of a map could occasionally flicker on small maps with For of War enabled


## 1.4.1
### Bugfixes
- The vision calculation algorithm no longer crashes when stepping under a roof that has no walls below it


## 1.4.0
### New features
- The cache that was introduced in 1.3.0, but disabled in 1.3.2 due to severe bugs is now fixed. It's now enabled again. This change makes the vision calculation about 20% faster than it was in 1.3.4.
- Lichtgeschwindigkeit now properly separates walls at different elevations that were placed with the Wall Height or Levels module, leading to a potentially huge boost in performance for those maps. The vision calculation within a level will now have nearly the same speed as if the level was placed on a entirely separate scene.
- Lichtgeschwindigkeit now calculates polygons of type `movement`, which increases compatibility with other modules.

### Bugfixes
- Light sources withing buildings that have a wall now shine through the windows, as they are supposed to.
- Fixed a bug that wrongly rendered walls as if they had a height while the "Wall Height" module is disabled.


## 1.3.4
### Bugfixes
- Fixed a bug that could cause the vision calculation to crash when a token or light was placed exactly on the endpoint of a wall [#19](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/19)


## 1.3.3
### Bugfixes
- Fixed a bug that could cause the vision calculation to crash when walls were paralell to a sight ray [#17](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/17)


## 1.3.2
### Bugfixes
- Fixed a bug that would cause invisible walls to block light if they were set to block sound [#16](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/16)
- Fixed a bug that caused walls below roofs to have the wrong visibility [#16](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/16)

### Performance
The bugs listed above were caused by the cache that was introduced in Lichtgeschwindigkeit 1.3.0. Unfortunately fixing the cache isn't simple, so it's being disabled for now to get rid of those bugs. This drops Lichtgeschwindigkeits speed back to the speed it had in Version 1.2.2.


## 1.3.1
### Bugfixes
- Fixed a bug that caused an error to be printed to the console in certain situations (initial load, scene switching, editing walls)


## 1.3.0
### New features
- Lichtgeschwindigket now caches the scenes walls in it's wasm memory. This makes the light calculation algorithm about 20% faster than it has been before.

### Compatibility
- Lichtgeschwindigkeit is now compatible with the [Wall Height module](https://foundryvtt.com/packages/wall-height). Wall Height version 3.5.3.9 or newer is required for compatibility.


## 1.2.2
### Bugfixes
- Fixed a bug that would cause the vision calculation to crash when walls were to the right of the token and touching the range of the fov ([#14](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/14), [#15](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/15))


## 1.2.1
### Bugfixes
- Vision with restricted angle is now calculated at the correct angle (0° now means down, as in vanilla foundry. Before 0° meant left in Lichtgeschwindigkeit).
- Fixed a bug that would stop a tokens movement mid-animation on scenes with Fog of War enabled when using Foundry version 0.8.6 or older.

## 1.2.0
### New features
- Lichtgeschwindigkeit now ships an improved, faster version of `PIXI.Polygon`. This improves the speed of lighting calculation and potentially improves speed in other areas in Foundry that make use of Polygons as well.


## 1.1.3
### Bugfixes
- Fixed a bug that caused the vision calculation to crash if a token was sitting precisely on top of a wall ([#10](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/10))

### Compatibility
- Lichtgeschwindigkeit is confirmed to work with Foundry 0.8.8


## 1.1.2
### Bugfixes
Fixed bugs that could cause the vision to be calculated incorrectly in scenes with
- Directional Walls ([#4](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/4))
- Walls arranged as t-junctions ([#5](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/5))
- Walls that have no length at all - meaning their start point is identical with their end point ([#6](https://github.com/manuelVo/foundryvtt-lichtgeschwindigkeit/issues/6))


## 1.1.1
### Bugfixes
- Fixed a bug that caused the vision calculation to crash if a wall was positioned at a very specific angle to a token/light source.


## 1.1.0
### New features
- Lichtgeschwindigkeit now speeds up a fog of war related calculation, reducing stutter during token animations on large maps that have fog of war enabled (this feature is only availabe if you use Foundry 0.8.7 or newer)

### Bugfixes
- Fixed a bug that caused the vision calclation to crash if tokens/lights with limited vision angle were placed into a scene with no walls

### Compatibility
- Lichtgeschwindigkeit is confirmed to work with Foundry 0.8.7


## 1.0.0
Initial release
