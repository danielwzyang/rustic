RUN ?= RUST_BACKTRACE=1 WINIT_UNIX_BACKEND=x11 cargo run
DEFAULT ?= scripts/3dface.mdl
CUSTOM ?= scripts/dino.mdl

default:
	${RUN} ${DEFAULT}

custom:
	${RUN} ${CUSTOM}

# run with make run S="path"
run:
	${RUN} ${S}

clean:
	rm *.ppm *.png *.gif **/*.ppm **/*.png **/*.gif

animate:
	animate temp_frames/${B}*.png

gif:
	convert -delay 1.7 temp_frames/${B}*.png output.gif
