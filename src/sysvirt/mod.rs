use libc::{c_int, c_void};
use libvirt_sys::*;
use log::{error, info};
use std::process;
use std::ptr;
use std::thread;

#[allow(dead_code)]
pub struct Virt {
    conn: virConnectPtr,
    reboot: i32,
}

#[allow(dead_code)]
impl Virt {
    pub fn new(uri: &str) -> Self {
        let conn: virConnectPtr;
        unsafe {
            if virInitialize() < 0 {
                error!("Failed to initialize libvirt");
                process::exit(1);
            }

            if virEventRegisterDefaultImpl() < 0 {
                error!(
                    "Failed to register event implementation: {:?}",
                    virGetLastErrorMessage()
                );
                process::exit(1);
            }

            conn = virConnectOpenReadOnly(uri.as_ptr() as *const i8);
            if conn == ptr::null_mut() {
                error!("Failed to opening libvirt");
                process::exit(1);
            }
        }

        Self { conn, reboot: 0 }
    }

    pub fn start(&mut self) {
        unsafe {
            let ret = virConnectRegisterCloseCallback(
                self.conn,
                Some(Self::connectClose),
                self as *mut _ as *mut c_void,
                None,
            );
            if ret < 0 {
                error!("Failed to virConnectRegisterCloseCallback");
                process::exit(1);
            }

            let ret = virConnectDomainEventRegister(
                self.conn,
                Some(Self::domainEventCallback),
                ptr::null_mut(),
                None,
            );
            if ret < 0 {
                error!("Failed to virConnectDomainEventRegister");
                process::exit(1);
            }

            let ret = virConnectSetKeepAlive(self.conn, 5, 3);
            if ret < 0 {
                error!("Failed to virConnectSetKeepAlive");
                process::exit(1);
            }

            self.reboot = virConnectDomainEventRegisterAny(
                self.conn,
                ptr::null_mut(),
                VIR_DOMAIN_EVENT_ID_REBOOT as i32,
                Some(Self::domainEventRebootCallback),
                ptr::null_mut(),
                None,
            );

            thread::spawn(|| loop {
                let ret = virEventRunDefaultImpl();
                if ret < 0 {
                    error!("Failed to run event loop: {:?}", virGetLastErrorMessage());
                    process::exit(1);
                }
            });
        }
    }

    extern "C" fn connectClose(conn: virConnectPtr, reason: c_int, opaque: *mut c_void) -> () {
        error!("connectClose");
    }

    extern "C" fn domainEventCallback(
        conn: virConnectPtr,
        dom: virDomainPtr,
        event: c_int,
        detail: c_int,
        opaque: *mut c_void,
    ) -> c_int {
        info!("domainEventCallback");
        0
    }

    extern "C" fn domainEventRebootCallback(
        conn: virConnectPtr,
        dom: virDomainPtr,
        opaque: *mut c_void,
    ) -> () {
        info!("domainEventRebootCallback");
    }
}

impl Drop for Virt {
    fn drop(&mut self) {
        unsafe { virConnectClose(self.conn) };
    }
}
