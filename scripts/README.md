# MDL Documentation
MDL is originally designed by Mr. DW.

Different commands and syntax changes have been made by me.

## Commands
Parentheses () indicate that a parameter is mandatory.

Brackets [] indicate that a parameter is optional.

Tokens not enclosed in anything are required tokens pertaining to the command syntax.

`display`
- open window to display current picture

`save (file_path)`
- save current picture under file_path

`camera (eye_x) (eye_y) (eye_z) (aim_x) (aim_y) (aim_z)`
- move the camera to eye coords and look at aim coords
- eye position acts as translation for the scene

`composite (name) begin (...) end`
- create a composite command
- essentially definition for function without parameters
- (...) indicates any list of commands

`run_composite (name)`
- run composite command

`clear`
- clear current picture

`push`
- push clone of current transformation matrix to stack

`pop`
- pop top of transformation stack

`move (a) (b) (c) [knob]`
- apply translation matrix to current transformation matrix
- knobs will be explained in the animation commands later

`scale (a) (b) (c) [knob]`
- apply dilation matrix to current transformation matrix

`rotate (x | y | z) (degrees) [knob]`
- apply rotation matrix to current transformation matrix

`save_coord_system (name)`
- save clone of current transformation matrix in symbol table

`line (x0) (y0) (z0) (x1) (y1) (z1)`
- draw line from point 0 to point 1

`circle (x) (y) (z) (r)`
- draw circle with center at xyz and radius r

`hermite (x0) (y0) (x1) (y1) (rx0) (ry0) (rx1) (ry1)`
- draw hermite curve given two points and the rate of change at each point

`bezier (x0) (y0) (x1) (y1) (x2) (y2) (x3) (y3)`
- draw cubic bezier curve given four points

`polygon [constants] (x0) (y0) (z0) (x1) (y1) (z1) (x2) (y2) (z2) [coord_system]`
- draw a triangle
- constants will be explained later in the constants command

`box [constants] (x) (y) (z) (w) (h) (d) [coord_system]`
- draw a box with width w, height h, and depth d
- xyz specifies the top left corner of the front side

`sphere [constants] (x) (y) (z) (r) [coord_system]`
- draw a sphere with center at xyz and radius r

`torus [constants] (x) (y) (z) (r0) (r1) [coord_system]`
- draw a torus with center at xyz
- r0 is the radius of the circle that makes up the torus
- r1 is the length from the center of the torus to the center of the circle

`cylinder [constants] (x) (y) (z) (r) (h) [coord_system]`
- draw a cylinder with base center at xyz, radius r, height h

`cone [constants] (x) (y) (z) (r) (h) [coord_system]`
- draw a cone with base center at xyz, base radius r, height h

`mesh [constants] (file_path) [coord_system]`
- draw a mesh loaded from file_path (obj or stl)

`light (r) (g) (b) (x) (y) (z)`
- add a light with color rgb with direction xyz
- light vectors are calculated as pointing out of the surface not into it

`clear_lights`
- remove all lights from the scene

`ambient (r) (g) (b)`
- set the ambient light color to rgb

`constants (name) (kar) (kdr) (ksr) (kag) (kdg) (ksg) (kab) (kdb) (ksb)`
- define a set of lighting constants under name
- k[a/d/s][r/g/b] are the ambient, diffuse, and specular constants for each color

`shading (wireframe | flat | gouraud | phong)`
- set the shading mode for subsequent shapes

`basename (name)`
- set the base filename used when saving animation frames

`set (name) (value)`
- set a knob to a specific value
- knobs will have a different value throughout different frames that allow for animated transformations

`save_knobs (name)`
- save the current state of all knobs under a knob list

`tween (start_frame) (end_frame) (knoblist0) (knoblist1) [easeInCubic | easeOutCubic | easeInExpo | easeOutExpo]`
- interpolate between two knob lists across a range of frames
- optionally use built in easing functions

`frames (num_frames)`
- set the total number of frames for the animation

`vary (knob) (start_frame) (end_frame) (start_val) (end_val) [easeInCubic | easeOutCubic | easeInExpo | easeOutExpo]`
- animate a knob from start_val to end_val over the given frame range

`setknobs (value)`
- set all knobs to the same value

`generate_rayfiles`
- not implemented

`focal (length)`
- not implemented

### Animation

Please note that some commands are only enabled during animation while others are disabled.

Animation only:
- basename
- frames
- vary
- tween
- save_knobs

Disabled in animation:
- display
- save

