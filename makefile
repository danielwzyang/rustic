RUN ?= WINIT_UNIX_BACKEND=x11 cargo run
DEFAULT ?= scripts/3dface.mdl
CUSTOM ?= scripts/dino.mdl

default:
	${RUN} ${DEFAULT}

custom:
	${RUN} ${CUSTOM}

# run with make run SCRIPT="path"
run:
	${RUN} ${SCRIPT}

clean:
	rm *.ppm *.png *.gif **/*.ppm **/*.png **/*.gif

animate:
	animate temp_frames/*.png

gif:
	convert -delay 1.7 temp_frames/*.png output.gif
