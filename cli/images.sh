#!/bin/bash
set -x;
cd images;
magick mogrify -background white -flatten -alpha off -format jpg "*.png"
mv *.jpg ../out_images/;
cd ../out_images/;
mogrify -gravity South -chop 0x1 "*.jpg"; 
ffmpeg -framerate 30 -pattern_type glob -i '*.jpg' -r 30 -pix_fmt yuv420p out.mp4;
open out.mp4;
set +x;