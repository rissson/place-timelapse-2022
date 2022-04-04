# [rplace.space](https://rplace.space)

[This scrapper](https://github.com/ProstoSanja/place-2022) made by
[ProstoSanja](https://github.com/ProstoSanja) is used to get images of the
canvas every 30 seconds. Have a look over there for how to run it.

Then every 5 minutes, this tool is run to merge the four parts of the canvas
together. Usage is `place <list of images to merge>`. The list of images must
follow this format: `[canvas id]-[timestamp].png`. Images from the scrapper are
already named in the correct format. Merged images are outputted in the current
directory at `[timestamp].png`.

And then, `ffmpeg` is used to create a time lapse every 5 minutes:

```sh
ffmpeg -y -framerate 60 -pattern_type glob -i "*.png" -s:v 2000x2000 -c:v libx264 -crf 17 -pix_fmt yuv420p video.mp4
```

### Building this tool

`cargo build --release`
