use std::thread;
use std::time::Duration;
extern crate libloading;
extern crate ctor;

use ctor::ctor;
use libloading::{Library, Symbol};
use crate::{InternalMemory, ESP};
use crate::util::{game_base, Vec3, ViewMatrix};

pub mod process;
pub mod player;
pub mod aimbot;
pub mod esp;
pub mod util;

pub use esp::*;
pub use aimbot::*;
pub use player::*;
pub use process::*;
pub use util::*;

/// This is a static reference to the initialized hack. It is initialized at load time of the library
/// and used for every frame of the game (SDL_GL_SwapBuffers())
static mut AC_HACK: Option<AcHack> = None;

/// a reference to the dynamically loaded libSDL. We use this dynamically loaded library
/// to keep a reference to the real SDL_GL_SwapBuffers() so that the hack can call it after
/// the hook has finished.
static mut SDL_DYLIB: Option<libloading::Library> = None;

/// The main struct containing the current configuration of the cheat
struct AcHack {
    pub player: Player,
    pub god_mode: GodMode,
    pub infinite_ammo: InfiniteAmmo,
    pub aimbot: AimBot,
    pub esp: ESP,
    pub player_pointer: *const i64,
}

impl AcHack {
    fn new() -> Self {
        let player = Player::player1();
        AcHack {
            aimbot: AimBot::new(),
            esp: ESP::new(),
            god_mode: GodMode::new(),
            infinite_ammo: InfiniteAmmo::new(),
            player,
            player_pointer: std::ptr::null_mut(),
        }
    }

    fn init() -> Self {
        println!("here");
        let mut hack = Self::new();
        hack.aimbot.enable();
        hack.aimbot.norecoil_spread.toggle();
        hack.aimbot.enable_autoshoot();
        hack.infinite_ammo.toggle();
        hack.god_mode.toggle();

        let offset: usize = 0x19D518;
        let gameBase: usize = game_base();
        let player1: *const i64 = (gameBase + offset) as *const i64;
        
        unsafe {
            let player: *const i64 = *player1 as *const i64;
            hack.player_pointer = player;
        }

        hack
    }
}

#[ctor]
fn load() {
    let process = Process::current().expect("Could not use /proc to obtain process information");
    if let Err(_e) = process.module("linux_64_client") {
        return;
    }

    let mut found = false;
    let modules = process.modules().expect("Could not parse the loaded modules");
    for module_name in modules.keys() {
        if module_name.contains("libSDL2") {
            unsafe {
                SDL_DYLIB = Some(libloading::Library::new(module_name).expect("Could not load libSDL"));
            };
            found = true;
        }
    }

    if !found {
        panic!("Could not find libSDL-1.2 in current process");
    }

    println!("Successfully loaded the hack into the game...");
    println!("Waiting 5 seconds for the game to initialize before touching anything.");

    thread::spawn(|| {
        thread::sleep(Duration::from_secs(5));
        unsafe {
            AC_HACK = Some(AcHack::init());
        }
    });
}

fn forward_to_orig_sdl_swap_buffers(window: *mut std::ffi::c_void) -> i64 {
    unsafe {
        let libsdl2 = &SDL_DYLIB;
        if !libsdl2.is_some() {
            eprintln!("SDL2 is not loaded!");
            return 0;
        }

        let orig_sdl_swap_window: Symbol<unsafe extern "C" fn(*mut std::ffi::c_void) -> i64> =
            SDL_DYLIB
                .as_ref()
                .unwrap()
                .get(b"SDL_GL_SwapWindow\0")
                .expect("Could not find SDL_GL_SwapWindow() in libSDL2");

        orig_sdl_swap_window(window)
    }
}

#[no_mangle]
pub extern "C" fn SDL_GL_SwapWindow(window: *mut std::ffi::c_void) -> i64 {
    #![allow(unused_mut)]
    let hack = unsafe { &mut AC_HACK };

    if hack.is_none() {
        return forward_to_orig_sdl_swap_buffers(window);
    }

    let mut hack = unsafe { hack.as_mut().unwrap() };
    let offset_health: i32 = 0x100;
    let offset_ammo: i32 = 0x154;

    unsafe {
        if !hack.player_pointer.is_null() {
             let health: *const i64 = (hack.player_pointer as usize + offset_health as usize) as *const i64;
             InternalMemory::write::<i32>(health as usize, 1000); // Setting health to 1000 (God Mode)
             let ammo: *const i64 = (hack.player_pointer as usize + offset_ammo as usize) as *const i64;
             InternalMemory::write::<i32>(ammo as usize, 1000); 
        } else {
            println!("Player pointer is null!");
        }

    }

    println!("game_base: {:#x}", game_base());

    // Here you can add other cheat logic (e.g., ESP, aimbot)
    // hack.esp.draw();
    // hack.aimbot.logic();

    forward_to_orig_sdl_swap_buffers(window)
}

