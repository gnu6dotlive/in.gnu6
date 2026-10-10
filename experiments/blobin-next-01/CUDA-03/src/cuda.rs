//! Explicitly loaded, owned CUDA session. All allocations and streams are native and bounded.
//! ABI library path must be a trusted, operator-built absolute path. dlopen is NOT sandboxed.
use blobin_cuda02_provider::{Primitive, Request, Sample};
use std::{
    ffi::{c_char, c_void, CString},
    path::Path,
};
#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(name: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> i32;
}
struct Library(*mut c_void);
impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            dlclose(self.0);
        }
    }
}
type Create = unsafe extern "C" fn(*const Primitive, u32, u32, u32, u64) -> *mut c_void;
type Update = unsafe extern "C" fn(*mut c_void, *const Primitive, u32) -> i32;
type Run = unsafe extern "C" fn(*mut c_void, *mut f32) -> i32;
type Copy = unsafe extern "C" fn(*mut c_void, *mut Sample, u64) -> i32;
type Bytes = unsafe extern "C" fn(*mut c_void) -> u64;
type Destroy = unsafe extern "C" fn(*mut c_void);
pub struct Resident {
    context: *mut c_void,
    destroy: Destroy,
    update: Update,
    run: Run,
    copy: Copy,
    bytes: Bytes,
    _library: Library,
    count: usize,
    pixels: usize,
}
impl Resident {
    pub fn new(req: &Request, path: &Path) -> Result<Self, String> {
        req.validate()?;
        if !path.is_absolute() {
            return Err("CUDA03 LIBRARY PATH MUST BE ABSOLUTE".into());
        }
        let name = CString::new(path.to_str().ok_or("CUDA03 LIBRARY PATH NOT UTF8")?)
            .map_err(|e| e.to_string())?;
        let handle = unsafe { dlopen(name.as_ptr(), 2) };
        if handle.is_null() {
            return Err("CUDA03 LIBRARY UNAVAILABLE".into());
        }
        let lib = Library(handle);
        fn symbol(lib: &Library, name: &'static str) -> Result<*mut c_void, String> {
            let key = CString::new(name).expect("static identifier");
            let address = unsafe { dlsym(lib.0, key.as_ptr()) };
            if address.is_null() {
                return Err(format!("CUDA03 SYMBOL ABSENT: {name}"));
            }
            Ok(address)
        }
        let create: Create = unsafe { std::mem::transmute(symbol(&lib, "blobin_cuda03_create")?) };
        let destroy: Destroy =
            unsafe { std::mem::transmute(symbol(&lib, "blobin_cuda03_destroy")?) };
        let update: Update = unsafe { std::mem::transmute(symbol(&lib, "blobin_cuda03_update")?) };
        let run: Run = unsafe { std::mem::transmute(symbol(&lib, "blobin_cuda03_run")?) };
        let copy: Copy = unsafe { std::mem::transmute(symbol(&lib, "blobin_cuda03_copy")?) };
        let bytes: Bytes =
            unsafe { std::mem::transmute(symbol(&lib, "blobin_cuda03_resident_bytes")?) };
        let (width, height) = req.dimensions();
        let pixels = width as usize * height as usize;
        let context = unsafe {
            create(
                req.primitives().as_ptr(),
                req.primitives().len() as u32,
                width,
                height,
                pixels as u64,
            )
        };
        if context.is_null() {
            return Err("CUDA03 CREATE FAILED".into());
        }
        Ok(Self {
            context,
            destroy,
            update,
            run,
            copy,
            bytes,
            _library: lib,
            count: req.primitives().len(),
            pixels,
        })
    }
    pub fn resident_bytes(&self) -> u64 {
        unsafe { (self.bytes)(self.context) }
    }
    pub fn update(&mut self, req: &Request) -> Result<(), String> {
        req.validate()?;
        if req.primitives().len() != self.count
            || req.dimensions().0 as usize * req.dimensions().1 as usize != self.pixels
        {
            return Err("CUDA03 UPDATE CHANGED SHAPE".into());
        }
        let error =
            unsafe { (self.update)(self.context, req.primitives().as_ptr(), self.count as u32) };
        if error != 0 {
            return Err(format!("CUDA03 UPDATE FAILED status={error}"));
        }
        Ok(())
    }
    pub fn launch(&mut self) -> Result<f32, String> {
        let mut ms = 0.0;
        let error = unsafe { (self.run)(self.context, &mut ms) };
        if error != 0 {
            return Err(format!("CUDA03 KERNEL FAILED status={error}"));
        }
        if !ms.is_finite() || ms < 0.0 {
            return Err("CUDA03 INVALID GPU EVENT".into());
        }
        Ok(ms)
    }
    pub fn readback(&self) -> Result<Vec<Sample>, String> {
        let mut output = vec![Sample::default(); self.pixels];
        let code = unsafe { (self.copy)(self.context, output.as_mut_ptr(), self.pixels as u64) };
        if code != 0 {
            return Err(format!("CUDA03 COPY FAILED status={code}"));
        }
        Ok(output)
    }
}
impl Drop for Resident {
    fn drop(&mut self) {
        unsafe {
            (self.destroy)(self.context);
        }
    }
}
