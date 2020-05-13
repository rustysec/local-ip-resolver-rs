#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[cfg_attr(target_pointer_width = "32", path = "api32.rs")]
#[cfg_attr(target_pointer_width = "64", path = "api64.rs")]
mod api;

const AF_INET: u32 = 2;

use crate::Result;
pub use api::*;
use std::ptr::null_mut;
pub use winapi::shared::inaddr::IN_ADDR;

pub(crate) unsafe fn get_ip_for_interface(index: DWORD) -> Result<String> {
    let mut needed = 0;

    GetAdaptersAddresses(AF_INET, 0, null_mut(), null_mut(), &mut needed);

    let mut buffer: Vec<u8> = vec![0; needed as _];

    GetAdaptersAddresses(
        AF_INET,
        0,
        null_mut(),
        buffer.as_mut_ptr() as _,
        &mut needed,
    );

    let mut first: _IP_ADAPTER_ADDRESSES_XP = std::ptr::read(buffer.as_mut_ptr() as _);

    while first.__bindgen_anon_1.__bindgen_anon_1.IfIndex != index {
        if first.Next.is_null() {
            break;
        }
        first = std::ptr::read(first.Next);
    }

    if first.FirstUnicastAddress.is_null() {
        return Err(String::from("Unable to get interface addresses").into());
    }

    let address: &PIP_ADAPTER_UNICAST_ADDRESS_XP = &(first.FirstUnicastAddress as _);
    let addr_in: sockaddr_in = std::ptr::read((*(*address)).Address.lpSockaddr as _);

    let octets = addr_in.sin_addr.S_un.S_addr.to_be_bytes();

    Ok(std::net::Ipv4Addr::from([octets[3], octets[2], octets[1], octets[0]]).to_string())
}
