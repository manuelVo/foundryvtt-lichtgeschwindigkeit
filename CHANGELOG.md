## 1.1.3
### Bugfixes
- Fixed a bug that caused the vision calculation to crash if a token was sitting precisely on top of a wall

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
