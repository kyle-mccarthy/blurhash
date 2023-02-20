# Blurhash

Blurhash encoding algorithm implemented in rust.

## Notes

**Why is the hash produced different https://blurha.sh?**

The typescript implementation leverages the Canvas API for getting the image's pixels. This is done by getting the 2D context from the canvas and drawing the image on the canvas. Then the image data can be extracted from the canvas and then used to derive the hash.

During the drawing step, `dx`, `dy`, `dWidth`, and `dHeight` values are provided which dictate the position of the image on the canvas as well as the width and height of the image on the canvas. The example on the website uses the __canvas's__ width and height during this process, which _can lead to the image being scaled_. After the image is scaled, the image drawn on the canvas will have different data than the original image, producing a different hash.

**How is this different from blurhash-rs?**

- This library only provides the encoding implementation
- During the linear to sRBG conversion, the other library incorrectly rounds rather than truncates. This produces slightly different values.
