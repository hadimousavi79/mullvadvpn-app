use std::{
    ffi::{c_char, c_void, CStr},
    ptr::null_mut,
    sync::Arc,
};

use mullvad_api::proxy::{ApiConnectionMode, ConnectionModeProvider};

use mullvad_encrypted_dns_proxy::state::EncryptedDnsProxyState as State;

use crate::{encrypted_dns_proxy::EncryptedDnsProxyState, ios::mullvad_ios_runtime};
extern "C" {
    pub fn connection_mode_provider_initial(rawPointer: *const c_void);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init_connection_mode_provider(
    raw_provider: *const c_void,
    domain_name: *const c_char,
) -> SwiftConnectionModeProvider {
    let domain = {
        // SAFETY: domain_name points to a valid region of memory and contains a nul terminator.
        let c_str = unsafe { CStr::from_ptr(domain_name) };
        String::from_utf8_lossy(c_str.to_bytes())
    };

    let state = EncryptedDnsProxyState {
        state: State::default(),
        domain: domain.into_owned(),
    };
    let context = SwiftConnectionModeProviderContext {
        provider: raw_provider,
        encrypted_dns_state: state,
    };

    SwiftConnectionModeProvider::new(context)
}

#[repr(C)]
pub struct SwiftConnectionModeProvider(*mut SwiftConnectionModeProviderContext);
impl SwiftConnectionModeProvider {
    pub fn new(context: SwiftConnectionModeProviderContext) -> SwiftConnectionModeProvider {
        SwiftConnectionModeProvider(Box::into_raw(Box::new(context)))
    }

    pub unsafe fn into_rust_context(self) -> Box<SwiftConnectionModeProviderContext> {
        Box::from_raw(self.0)
    }
}

pub struct SwiftConnectionModeProviderContext {
    provider: *const c_void,
    encrypted_dns_state: EncryptedDnsProxyState,
}

unsafe impl Send for SwiftConnectionModeProviderContext {}

impl ConnectionModeProvider for SwiftConnectionModeProviderContext {
    fn initial(&self) -> ApiConnectionMode {
        unsafe {
            connection_mode_provider_initial(self.provider);
        }
        ApiConnectionMode::Direct
    }

    fn rotate(&self) -> impl std::future::Future<Output = ()> + Send {
        futures::future::ready(())
    }

    fn receive(&mut self) -> impl std::future::Future<Output = Option<ApiConnectionMode>> + Send {
        // let runtime = mullvad_ios_runtime().unwrap();
        // runtime.spawn_blocking(func)
        // tokio::runtime::Handle::current().spawn_blocking(
        async { Some(ApiConnectionMode::Direct) }
    }
}
