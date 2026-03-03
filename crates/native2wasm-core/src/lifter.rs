use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_uchar, c_void};
use std::path::Path;

use crate::binary::ParsedBinary;
use crate::error::CoreError;

// Opaque handles returned by C++ bridge.
type RemillContext = c_void;
type RemillModule = c_void;

unsafe extern "C" {
    fn remill_context_create(
        semantics_dir: *const c_char,
        os_name: *const c_char,
        syscall_symbol: *const c_char,
    ) -> *mut RemillContext;
    fn remill_context_destroy(ctx: *mut RemillContext);

    fn remill_add_code_region(
        ctx: *mut RemillContext,
        name: *const c_char,
        vaddr: u64,
        bytes: *const c_uchar,
        size: u64,
    );

    fn remill_add_data_section(
        ctx: *mut RemillContext,
        name: *const c_char,
        vaddr: u64,
        bytes: *const c_uchar,
        size: u64,
        writable: i32,
    );

    fn remill_set_entry_point(ctx: *mut RemillContext, vaddr: u64);

    fn remill_lift(ctx: *mut RemillContext) -> *mut RemillModule;

    fn remill_module_destroy(module: *mut RemillModule);

    fn remill_write_bc(module: *mut RemillModule, path: *const c_char) -> i32;
    fn remill_write_ll(module: *mut RemillModule, path: *const c_char) -> i32;

    fn remill_get_error(ctx: *mut RemillContext) -> *const c_char;
}

/// Result of lifting x86_64 code to LLVM IR via Remill.
pub struct LiftedModule {
    module: *mut RemillModule,
}

impl Drop for LiftedModule {
    fn drop(&mut self) {
        if !self.module.is_null() {
            unsafe {
                remill_module_destroy(self.module);
            }
        }
    }
}

impl LiftedModule {
    /// Write the lifted module as LLVM bitcode (.bc).
    pub fn write_bc(&self, path: &Path) -> Result<(), CoreError> {
        let path_c = path_to_cstring(path);
        let rc = unsafe { remill_write_bc(self.module, path_c.as_ptr()) };
        if rc != 0 {
            return Err(CoreError::LiftingFailed(format!(
                "failed to write bitcode to {}",
                path.display()
            )));
        }
        Ok(())
    }

    /// Write the lifted module as human-readable LLVM IR (.ll).
    pub fn write_ll(&self, path: &Path) -> Result<(), CoreError> {
        let path_c = path_to_cstring(path);
        let rc = unsafe { remill_write_ll(self.module, path_c.as_ptr()) };
        if rc != 0 {
            return Err(CoreError::LiftingFailed(format!(
                "failed to write LLVM IR to {}",
                path.display()
            )));
        }
        Ok(())
    }
}

/// Lift a parsed binary to LLVM IR using Remill.
pub fn lift(binary: &dyn ParsedBinary, semantics_dir: &Path) -> Result<LiftedModule, CoreError> {
    let sem_dir = path_to_cstring(semantics_dir);
    let os_name = CString::new(binary.os_target().remill_os()).unwrap_or_default();
    let syscall_sym = CString::new(binary.os_target().syscall_symbol()).unwrap_or_default();

    unsafe {
        let ctx = remill_context_create(sem_dir.as_ptr(), os_name.as_ptr(), syscall_sym.as_ptr());
        if ctx.is_null() {
            return Err(CoreError::LiftingFailed(
                "failed to create remill context".into(),
            ));
        }

        // Register code regions.
        for section in binary.code_sections() {
            let name = CString::new(section.name.as_str()).unwrap_or_default();
            remill_add_code_region(
                ctx,
                name.as_ptr(),
                section.addr,
                section.data.as_ptr(),
                section.data.len() as u64,
            );
        }

        // Register data sections.
        for section in binary.data_sections() {
            let name = CString::new(section.name.as_str()).unwrap_or_default();
            remill_add_data_section(
                ctx,
                name.as_ptr(),
                section.addr,
                section.data.as_ptr(),
                section.data.len() as u64,
                section.writable as i32,
            );
        }

        // Set entry point.
        remill_set_entry_point(ctx, binary.entry_point());

        // Lift.
        let module = remill_lift(ctx);
        if module.is_null() {
            let err_ptr = remill_get_error(ctx);
            let msg = if err_ptr.is_null() {
                "unknown lifting error".to_string()
            } else {
                CStr::from_ptr(err_ptr).to_string_lossy().into_owned()
            };
            remill_context_destroy(ctx);
            return Err(CoreError::LiftingFailed(msg));
        }

        remill_context_destroy(ctx);
        Ok(LiftedModule { module })
    }
}

fn path_to_cstring(path: &Path) -> CString {
    CString::new(path.to_string_lossy().as_bytes()).unwrap_or_default()
}
