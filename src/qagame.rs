//! Code for the `qagame` Quake 3 module
//!
//! The `qagame` module contains the server-side logic of Quake 3, e.g.:
//!
//! * player physics
//! * game types
//! * weapons
//! * bot A.I.
//!
//! Unlike the other modules, it does not have to be present on the game client,
//! i.e. players do not have to download it.

use std::ffi::CString;
use {ffi, Syscall};
use num_traits::ToPrimitive;

/// System traps provided by the engine
///
/// See `gameImport_t` in [ioquake3's `game/g_public.h`](https://github.com/ioquake/ioq3/blob/master/code/game/g_public.h).
#[repr(C)]
// TODO: Should these be shortened and renamed, e.g. `Print` and `Error` instead of `G_PRINT` and `G_ERROR`?
#[allow(non_camel_case_types)]
#[derive(Primitive)]
pub enum Imports {
    G_PRINT = 0,
    G_ERROR = 1,
}

/// Functions exported by the module
///
/// See `gameExport_t` in [ioquake3's `game/g_public.h`](https://github.com/ioquake/ioq3/blob/master/code/game/g_public.h).
#[repr(C)]
// TODO: Should these be shortened and renamed, e.g. `Init` and `Shutdown` instead of `GAME_INIT` and `GAME_SHUTDOWN`?
#[allow(non_camel_case_types)]
#[derive(Primitive)]
pub enum Exports {
    GAME_INIT = 0,
    GAME_SHUTDOWN = 1,
}

/// `qagame` specific wrapper around generic [`Syscall`](Syscall)
///
/// See [ioquake3's `game/g_syscalls.c`](https://github.com/ioquake/ioq3/blob/master/code/game/g_syscalls.c).
pub struct Syscalls {
    syscall: Syscall,
}

impl Syscalls {
    /// See `dllEntry` in [ioquake3's `game/g_syscalls.c`](https://github.com/ioquake/ioq3/blob/master/code/game/g_syscalls.c).
    pub fn new(syscall: Syscall) -> Syscalls {
        Syscalls { syscall }
    }

    /// See `trap_Error` in [ioquake3's `game/g_syscalls.c`](https://github.com/ioquake/ioq3/blob/master/code/game/g_syscalls.c).
    pub fn error<T: Into<Vec<u8>>>(&self, text: T) {
        let msg = CString::new(text).unwrap();
        (self.syscall)(Imports::G_ERROR.to_isize().unwrap(), msg.as_ptr());
    }
}

/// See `vmMain` in [ioquake3's `game/g_main.c`](https://github.com/ioquake/ioq3/blob/master/code/game/g_main.c).
pub trait Module: 'static + Sync + Send {
    fn dll_entry(syscalls: Syscalls) -> Box<Self>
    where
        Self: Sized;

    /// See `G_InitGame` in [ioquake3's `game/g_main.c`](https://github.com/ioquake/ioq3/blob/master/code/game/g_main.c).
    fn init(&self, level_time: ffi::c_int, random_seed: ffi::c_int, restart: bool);

    /// See `G_ShutdownGame` in [ioquake3's `game/g_main.c`](https://github.com/ioquake/ioq3/blob/master/code/game/g_main.c).
    fn shutdown(&self, restart: bool);
}

#[macro_export]
macro_rules! game_module {
    ($ty:ident) => {
        struct ModuleWrapper {
            module: Box<$crate::qagame::Module>,
        }

        use $crate::NativeVM;
        impl $crate::NativeVM for ModuleWrapper {
            fn dll_entry(syscall: $crate::Syscall) -> Box<Self> {
                Box::new(ModuleWrapper {
                    module: $ty::dll_entry($crate::qagame::Syscalls::new(syscall)),
                })
            }

            fn vm_main(
                &self,
                command: $crate::ffi::c_int,
                arg0: $crate::ffi::c_int,
                arg1: $crate::ffi::c_int,
                arg2: $crate::ffi::c_int,
                _arg3: $crate::ffi::c_int,
                _arg4: $crate::ffi::c_int,
                _arg5: $crate::ffi::c_int,
                _arg6: $crate::ffi::c_int,
                _arg7: $crate::ffi::c_int,
                _arg8: $crate::ffi::c_int,
                _arg9: $crate::ffi::c_int,
                _arg10: $crate::ffi::c_int,
                _arg11: $crate::ffi::c_int,
            ) -> $crate::ffi::intptr_t {
                use $crate::num_traits::FromPrimitive;

                match $crate::qagame::Exports::from_i32(command) {
                    Some($crate::qagame::Exports::GAME_INIT) => {
                        self.module.init(arg0, arg1, arg2 != 0);
                        0
                    }
                    Some($crate::qagame::Exports::GAME_SHUTDOWN) => {
                        self.module.shutdown(arg0 != 0);
                        0
                    }
                    _ => panic!("Game command not implemented"),
                }
            }
        }

        native_vm!(ModuleWrapper);
    }
}
