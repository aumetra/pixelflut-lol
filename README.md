# riptide

Stupidly overengineered pixelflut client produced on more caffeine than sleep at 38C3.

## Architecture

`riptide-process` processes a directory of frames into their RGB representation with their preproccessed hex code representation.  
It also does some differential analysis and only draws pixels that changed.

The file is stored using [`rkyv`](https://rkyv.org/) which allows us to just mmap the preprocessed file and have no deserialization step.

The actual `riptide` binary then takes various input parameters to configure how the frames are displayed. Most important here is the framerate parameter.  
Make that match the framerate of the images you previously processed. Otherwise it will look weird.

The binary will then establish connections with Naegle's algorithm disabled, and run a monoio runtime on each available CPU core.  
There are no repeated allocations in the hotloop, it's all either stack allocated or reusing allocations.

The I/O is handled through the io_uring driver monoio provides. We do some whacky unsafe stuff, but it _should_ all be sound.  
Miri has obviously not been run over it because we wanted results and we wanted them fast.
