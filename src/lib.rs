/**
 * This hack is relatively simple. It is loaded into the AssaultCube process through
 * the LD_PRELOAD technique (e.g.) LD_PRELOAD=./hack.so ./assaultcube.sh in the main AC directory.
 * There is a constructor, which runs at load time. It is used to initialize the hack by
 *  - verifying this library is actually loaded into the game and not for example /bin/sh when
         launching AC through ./assaultcube.sh
 *  - finding offsets of code to patch
 *  - generating shellcode on the fly through nasm for hooks
 *  - prepares hooks
 *  - initialized the global AC_HACK variable
 *  - dynamically loads libSDL and obtains a pointer to the SDL_GL_SwapBuffers() function
 *
 *  By using the LD_PRELOAD technique, this hack hooks the SDL_GL_SwapBuffers() function.
 *  This function will then use the initialized, static variable AC_HACK to perform the logic
 *  it needs to do such as getting player positions, draw ESP boxes etc.
 *  The reason we use statics here is that we don't want to reload the entire hack
 *  for each frame
 */
use std::thread;
use std::time::Duration;

extern crate libloading;
extern crate ctor;
use ctor::ctor;
use libloading::{Library, Symbol};


// include all the different sub modules of this hack as pub for the documentation
pub mod process;
pub mod player;
pub mod aimbot;
pub mod esp;
pub mod util;

// make all their symbols available to the other submodules through 'crate::'
pub use esp::*;
pub use aimbot::*;
pub use player::*;
pub use process::*;
pub use util::*;

/// This is a static reference to the initialized hack. It is initialized at load time of the library
/// and used for every frame of the game (SDL_GL_SwapBuffers())
static mut AC_HACK: Option<AcHack> = None;

/// a reference to the dynamiclly loaded libSDL. We use this dynamically loaded library
/// to keep a ference to the real SDL_GL_SwapBuffers() so that the hack can call it after
/// the hook has finished.
static mut SDL_DYLIB: Option<libloading::Library> = None;

/// The main struct containing the current configuration of the cheat
struct AcHack {
    /// Exposes an interface to interact with the AC player struct
    pub player: Player,

    /// Enables GodMode (invincible, 1-shot-1kill
    pub god_mode: GodMode,

    /// Hooks the shooting function and enables infinite ammo
    pub infinite_ammo: InfiniteAmmo,

    /// Used to configure the aimbot
    pub aimbot: AimBot,

    /// Used to configure the ESP
    pub esp: ESP,
}


impl AcHack {
    /// Creates a new instance of the AcHack struct
    fn new() -> Self {
        // get a handle to the current process
        let player = Player::player1();
        AcHack {
            aimbot: AimBot::new(),
            esp: ESP::new(),
            god_mode: GodMode::new(),
            infinite_ammo: InfiniteAmmo::new(),
            player,
        }
    }

    /// Initializes default settings and launches a new thread that will listen for keyboard
    /// bindings
    fn init() ->Self {
        let mut hack = Self::new();

        // all the following are default settings for this hack
        hack.aimbot.enable();
        hack.aimbot.norecoil_spread.toggle();
        hack.aimbot.enable_autoshoot();
        hack.infinite_ammo.toggle();
        hack.god_mode.toggle();

        hack
    }
}

/// This function is executed when the hack is loaded into the game
/// it is used to initialize the hack, launch a new thread that listens for keyboard bindings etc
#[ctor]
fn load() {

    // Check if the current process has a linux_64_client module (the main AC binary)
    // otherwise don't load the cheat here
    let process = Process::current().expect("Could not use /proc to obtain process information");
    if let Err(_e) = process.module("native_client") {
        return;
    }

    // load libSDL dynamically by finding the module it is loaded at, get it's path and
    // use the libloading crate to dynamically load a pointer to the real SDL_GL_SwapBuffers()
    // function
    let mut found = false;
    let modules = process.modules().expect("Could not parse the loaded modules");
    for module_name in modules.keys() {
        if module_name.contains("libSDL2-2") {
            println!("{}", module_name);
            unsafe {
                SDL_DYLIB = Some(
                    libloading::Library::new(module_name)
                        .expect("Could not load libSDL")
                )
            };

            found = true;
        }
    }

    // this should not happen
    if !found {
        panic!("Could not find libSDL-1.2 in current process");
    }

    // let the user know we are loaded
    println!("Successfully loaded the hack into the game...");
    println!("Waiting 5 seconds for the game to initialize it self before touching anything.");


    // Wait 5 seconds in a new thread for the game to initialize
    // If we don't do this step, we might break something as some pointers might be uninitialized
    thread::spawn(|| {
        // Wait around 5 seconds to let the game actually load so that pointers are valid.
        thread::sleep(Duration::from_secs(3));

        // Load the cheat!
        unsafe {
            AC_HACK = Some(AcHack::init());
        }
    });
}

fn forward_to_orig_sdl_swap_buffers(window: *mut std::ffi::c_void) -> i64 {
    unsafe {
        // Überprüfen, ob SDL2 korrekt geladen wurde
        let libsdl2 = &SDL_DYLIB;
        if !libsdl2.is_some() {
            eprintln!("SDL2 is not loaded!");
            return 0; // Bild bleibt schwarz
        }

        // Finden der Original-SDL_GL_SwapWindow Funktion
        let orig_sdl_swap_window: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> i64> =
            SDL_DYLIB
                .as_ref()
                .unwrap()
                .get(b"SDL_GL_SwapWindow\0")
                .expect("Could not find SDL_GL_SwapWindow() in libSDL2");

        // Aufruf der Original-Funktion, um den Puffer zu tauschen
        orig_sdl_swap_window(window)
    }
}


#[no_mangle]
pub extern "C" fn SDL_GL_SwapWindow(window: *mut std::ffi::c_void) -> i64 {
    // rustc falsely detects this as an unused mutable
    #![allow(unused_mut)]
    let hack = unsafe { &mut AC_HACK };

    // Check if AC_HACK is initialized
    if !hack.is_some() {
        // If not initialized, just render the frame
        return forward_to_orig_sdl_swap_buffers(window);
    }
    println!("in the mehtod");

    let mut hack = hack.as_mut().unwrap();

    // Here goes the logic for the cheat

    // Handle ESP logic
    //hack.esp.draw();

    // Handle aimbot logic
    hack.aimbot.logic();

    // Call the real SDL_GL_SwapWindow to render the frame
    forward_to_orig_sdl_swap_buffers(window)
}

