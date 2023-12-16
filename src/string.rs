use std::{
    alloc::{alloc, Layout, LayoutError},
    slice,
    str::{self, Utf8Error},
};

pub fn make_from_str(s: &str) -> Result<Box<[u8]>, LayoutError> {
    let allocation = unsafe { alloc(Layout::array::<u8>(s.len() + 1)?) };
    let slice = unsafe { slice::from_raw_parts_mut(allocation, s.len() + 1) };
    slice[..s.len()].copy_from_slice(s.as_bytes());
    let mut zs = unsafe { Box::from_raw(slice as *mut [u8]) };
    zs[zs.len() - 1] = 0;
    Ok(zs)
}

pub unsafe fn ptr_to_str<'a>(ptr: *const u8) -> Result<&'a str, Utf8Error> {
    let len = len_from_ptr(ptr);
    let slice = unsafe { slice::from_raw_parts(ptr, len) };
    str::from_utf8(slice)
}

fn len_from_ptr(mut s: *const u8) -> usize {
    let mut len = 0;
    while unsafe { *s } != 0 {
        len += 1;
        s = unsafe { s.add(1) };
    }
    len
}
