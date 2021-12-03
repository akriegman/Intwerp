# This makefile is for cropping screenshots.
# This depends on Imagemagick for the `convert` command.
# The numbers used here are based on Ubuntu's
# screenshot of a window functionality. YMMV.
# There was an additional step involved of
# replacing spaces with underscores in the filenames.


targets = $(shell echo Screenshot*.png | sed 's/Screenshot/Cropped/g')

all: $(targets)

$(targets): Cropped%: Screenshot%
	convert $< -crop 900x900+10+45 $@

