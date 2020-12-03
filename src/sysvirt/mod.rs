use libvirt_sys::*;
use std::ptr;

#[allow(dead_code)]
pub struct Virt {
    conn: virConnectPtr,
}

#[allow(dead_code)]
impl Virt {
    pub fn new(uri: &str) -> Self {
        Virt {
            conn: unsafe { virConnectOpenReadOnly(uri.as_ptr() as *const i8) },
        }
    }

    pub fn start(&self) {
        unimplemented!();
    }
}

impl Drop for Virt {
    fn drop(&mut self) {
        if self.conn != ptr::null_mut() {
            unsafe { virConnectClose(self.conn) };
        }
    }
}
