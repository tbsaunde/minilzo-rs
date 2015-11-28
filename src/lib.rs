extern crate minilzo_sys;
extern crate libc;

use std::mem::size_of;
use std::ptr;

use libc::{c_int, c_short, c_long};
use minilzo_sys::{
    // Types
    lzo_uint,
    lzo_callback_t,

    // Helpers
    LZO1X_1_MEM_COMPRESS,
    lzo_version,
    __lzo_init_v2,

    // (De)compress
    lzo1x_1_compress,
    lzo1x_decompress_safe,
};

#[derive(Debug, PartialEq)]
pub enum Error {
    Error,
    OutOfMemory,
    NotCompressible,
    InputOverrun,
    OutputOverrun,
    LookbehindOverrun,
    EOFNotFound,
    InputNotConsumed,
    NotYetImplemented,
    InvalidArgument,
    InvalidAlignment,
    OutputNotConsumed,
    InternalError,
}

impl Error {
    pub fn from_code(code: i32) -> Error {
        match code {
             -1 => Error::Error,
             -2 => Error::OutOfMemory,
             -3 => Error::NotCompressible,
             -4 => Error::InputOverrun,
             -5 => Error::OutputOverrun,
             -6 => Error::LookbehindOverrun,
             -7 => Error::EOFNotFound,
             -8 => Error::InputNotConsumed,
             -9 => Error::NotYetImplemented,
            -10 => Error::InvalidArgument,
            -11 => Error::InvalidAlignment,
            -12 => Error::OutputNotConsumed,
            -99 => Error::InternalError,
            _ => Error::Error,
        }
    }
}

fn _lzo_init() -> i32 {
    unsafe {
        __lzo_init_v2(lzo_version(),
                      size_of::<c_short>() as c_int,
                      size_of::<c_int>() as c_int,
                      size_of::<c_long>() as c_int,
                      size_of::<u32>() as c_int, // lzo_uint32_t
                      size_of::<lzo_uint>() as c_int,
                      size_of::<usize>() as c_int, // lzo_sizeof_dict_t
                      size_of::<usize>() as c_int, // char*
                      size_of::<usize>() as c_int, // lzo_voidp
                      size_of::<lzo_callback_t>() as c_int
                     )
    }
}

pub fn compress(indata: &[u8]) -> Result<Vec<u8>, Error> {
    let mut wrkmem : [u8; LZO1X_1_MEM_COMPRESS] = unsafe {
        std::mem::uninitialized()
    };

    let inlen = indata.len();
    let outlen = inlen + inlen / 16 + 64 + 3;
    let mut outdata = Vec::with_capacity(outlen);

    unsafe {
        let r = lzo1x_1_compress(
            indata.as_ptr(),
            inlen as u64,
            outdata.as_mut_ptr(),
            &outlen as *const _ as *mut _,
            wrkmem.as_mut_ptr() as *mut _);

        if r == 0 {
            if outlen > inlen {
                return Err(Error::NotCompressible)
            }

            outdata.set_len(outlen);
            return Ok(outdata)
        }

        return Err(Error::from_code(r))
    }
}

pub fn decompress(indata: &[u8], newlen: usize) -> Result<Vec<u8>, Error> {
    let inlen = indata.len();
    let mut outdata = Vec::with_capacity(newlen);

    unsafe {
        let r = lzo1x_decompress_safe(
            indata.as_ptr(),
            inlen as u64,
            outdata.as_mut_ptr(),
            &newlen as *const _ as *mut _,
            ptr::null_mut());

        if r == 0 {
            outdata.set_len(newlen);
            return Ok(outdata)
        }

        return Err(Error::from_code(r))
    }
}

#[test]
fn init() {
    // We test this, but we don't export it to the user right now
    assert_eq!(0, _lzo_init());
}

#[test]
fn test_compress_skips_short() {
    assert_eq!(Err(Error::NotCompressible), compress("foo".as_bytes()));
}

#[test]
fn test_compress_fails_with_short_output() {
    let data = [0; 128*1024];
    let compressed = compress(&data[..]).unwrap();

    assert_eq!(Err(Error::OutputOverrun),
               decompress(&compressed, 128));
}

#[test]
fn simple_compress_decompress() {
    let data = [0; 128*1024];
    let compressed = compress(&data[..]).unwrap();

    assert_eq!(593, compressed.len());

    let decompressed = decompress(&compressed, 128*1024).unwrap();
    assert_eq!(128*1024, decompressed.len());
}

#[test]
fn test_compress_decompress_lorem_round() {
    let lorem = "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod \
                 tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At \
                 vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, \
                 no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit \
                 amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut \
                 labore et dolore magna aliquyam erat, sed diam voluptua.";

    let compressed = compress(lorem.as_bytes()).unwrap();
    let decompressed = decompress(&compressed, lorem.len()).unwrap();

    assert_eq!(lorem.len(), decompressed.len());
    assert_eq!(lorem.as_bytes(), &decompressed[..]);
}

#[test]
fn test_alice_wonderland_both() {
    let alice = "\r\n\r\n\r\n\r\n                ALICE'S ADVENTURES IN WONDERLAND\r\n";

    let compressed = compress(alice.as_bytes()).unwrap();
    let decompressed = decompress(&compressed, alice.len()).unwrap();

    assert_eq!(alice.len(), decompressed.len());
    assert_eq!(alice.as_bytes(), &decompressed[..]);
}
