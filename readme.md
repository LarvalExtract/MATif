## About
MATif is a command-line tool used to convert images from common image formats into MechAssault's proprietary TIF files compliant with both games. This allows for custom textures to be modded in to each game. This tool should be used with [MGF-Packer](https://github.com/LarvalExtract/MGF-Packer) to package textures into the game for use.

## Usage

Convert an image to a DXT5 texture for MechAssault 1:
```
matif ma1 "C:\path\to\image.png" --format dxt5
```
This will write `C:\path\to\image.tif` to disk

Convert an image to an ARGB8888 texture for MechAssault 2:
```
matif ma2 "C:\path\to\image.png" --format argb8888
```
This will write `C:\path\to\image.tif` to disk