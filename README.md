This basic cheat is the result of a side-project I did for fun. It is an Internal Hack for AssaultCube on Linux written in Rust.

## Features

* Infinite Ammo
* Invincibility

## Usage

### Building

The build system of this cheat is cargo. External library dependencies are libGL headers and libSDL.

On Ubuntu, you can install all prerquisites with:

```bash
#/bin/bash
# install openGL header files
sudo apt install libgl-dev -y

# you will probably need to install libSDL-image to run the game
sudo apt-install libsdl-image1.2-dev -y
```

Then, simply run

```bash
cargo build --release # 
# or under nixos
nix build
```

### Loading the cheat

This cheat uses the Linux `LD_PRELOAD` technique to load the binary into the target process and to hook
`SDL_GL_SwapBuffers()`. 

After building the cheat, run the following command from root directory of AssaultCube run:

```bash
LD_PRELOAD=/path/to/libac_rhack.so PATH/TO/AC/ac_client
```

For LD_PRELOAD use the full path. 

## Documentation

A documentation can be generated via

```bash
cargo doc
```

Alternatively, just read the source code. I made lots of comments.
