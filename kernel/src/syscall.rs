use tracing::trace;

#[derive(Debug)]
#[repr(usize)]
enum Error {
    InvalidPointer = 1,
}

fn user_pointer<T>(p: *const T) -> Result<(), Error> {
    // TODO: is the pointer a valid VA for reads
    if p as usize > 0x0000_ffff_ffff_ffff || !p.is_aligned() {
        return Err(Error::InvalidPointer);
    }

    Ok(())
}

fn user_slice<'a, T>(base: *const T, len: usize) -> Result<&'a [T], Error> {
    // TODO: is the pointer a valid VA for len reads
    user_pointer(base)?;
    Ok(unsafe { core::slice::from_raw_parts(base, len) })
}

pub fn dispatch(num: usize, params: &mut [usize; 8]) {
    // lol
    let [ref mut a, ref mut b, ref mut _c, ref mut _d, ref mut _e, ref mut _f, ref mut _g, ref mut h] =
        params;
    let res = match num {
        0 => syscall_exit(),
        1 => syscall_print(*a as _, *b),
        _ => {
            tracing::warn!("invalid syscall number {a}");
            syscall_exit();
        }
    };
    *h = match res {
        Ok(_) => 0,
        Err(e) => e as usize,
    };
}

#[tracing::instrument(level = "debug")]
fn syscall_exit() -> ! {
    trace!("");
    crate::context::exit();
}

#[tracing::instrument(level = "debug", err(Debug))]
fn syscall_print(base: *const u8, len: usize) -> Result<(), Error> {
    trace!("");
    let bytes = user_slice(base, len)?;
    let bytes = unsafe { core::slice::from_raw_parts(base, len) };
    let putchar = crate::console::get_writer().0;
    for byte in bytes {
        putchar(*byte);
    }
    Ok(())
}
