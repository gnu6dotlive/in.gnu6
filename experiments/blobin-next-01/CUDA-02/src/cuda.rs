//! Explicit optional CUDA shared-library provider. Linux dlopen on operator-supplied path.
//! Library is loaded only for a requested CUDA run. No implicit process-wide symbol resolution.
use crate::{Request, Sample};
use std::{
    ffi::{c_char, c_void, CString},
    path::Path,
};

#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> i32;
}
const RTLD_NOW: i32 = 2;
struct Library(*mut c_void);
impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            dlclose(self.0);
        }
    }
}
type Compute =
    unsafe extern "C" fn(*const crate::Primitive, u32, u32, u32, *mut Sample, u64) -> i32;
pub fn evaluate(req: &Request, path: &Path) -> Result<Vec<Sample>, String> {
    req.validate()?;
    let raw = path.to_str().ok_or("CUDA02 LIBRARY PATH NON UTF8")?;
    if !path.is_absolute() {
        return Err("CUDA02 CUDA LIBRARY MUST BE ABSOLUTE PATH".into());
    }
    let name = CString::new(raw).map_err(|e| e.to_string())?;
    let handle = unsafe { dlopen(name.as_ptr(), RTLD_NOW) };
    if handle.is_null() {
        return Err("CUDA02 CUDA LIBRARY NOT AVAILABLE".into());
    }
    let _library = Library(handle);
    let symbol = CString::new("blobin_cuda02_evaluate").expect("static C name");
    let procedure = unsafe { dlsym(handle, symbol.as_ptr()) };
    if procedure.is_null() {
        return Err("CUDA02 CUDA PROVIDER SYMBOL MISSING".into());
    }
    let compute: Compute = unsafe { std::mem::transmute(procedure) };
    let pixels = req.width as usize * req.height as usize;
    let mut samples = vec![Sample::default(); pixels];
    let code = unsafe {
        compute(
            req.primitives.as_ptr(),
            req.primitives.len() as u32,
            req.width,
            req.height,
            samples.as_mut_ptr(),
            pixels as u64,
        )
    };
    if code != 0 {
        return Err(format!("CUDA02 CUDA PROVIDER STATUS {code}"));
    }
    Ok(samples)
}
