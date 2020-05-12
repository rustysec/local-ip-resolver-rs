//! local-ip-resolver
//! =================
//!
//! simple interface to get the current ip address used
//! to communicate to a remote host.
//!

#[cfg(windows)]
use widestring::WideCString;
#[cfg(windows)]
use winapi::shared::{
    inaddr::IN_ADDR,
    minwindef::{DWORD, ULONG},
    ntdef::LPWSTR,
};

#[cfg(windows)]
#[link(name = "iphlpapi")]
extern "system" {
    fn GetBestInterface(addr: IN_ADDR, index: *mut DWORD) -> u32;
    fn GetAdapterIndex(adapter_name: LPWSTR, index: *mut ULONG) -> u32;
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(target_os = "macos")]
pub fn for_host<S: AsRef<str>>(host: S) -> Result<String> {
    use std::process::Command;

    let command = Command::new("route")
        .args(&["get", host.as_ref()])
        .output()?;

    let output = String::from_utf8(command.stdout)?
        .lines()
        .map(|line| line.trim())
        .filter(|line| line.starts_with("interface:"))
        .next()
        .map(|line| line.to_owned())
        .ok_or(String::from("Could not find route for host"))?;

    let iface_name = output
        .split_whitespace()
        .nth(1)
        .ok_or(String::from("Could not get interface name"))?;

    let command = Command::new("ifconfig").args(&[iface_name]).output()?;

    let output = String::from_utf8(command.stdout)?
        .lines()
        .map(|line| line.trim())
        .filter(|line| line.starts_with("inet "))
        .next()
        .map(|line| line.to_owned())
        .ok_or(String::from("Could not find IPv4 address for interface"))?;

    output
        .split_whitespace()
        .nth(1)
        .map(|ip| ip.to_string())
        .ok_or(String::from("Could not get IP address of interface").into())
}

#[cfg(target_os = "linux")]
pub fn for_host<S: AsRef<str>>(host: S) -> Result<String> {
    use std::process::Command;

    let command = Command::new("ip")
        .args(&["route", "get", host.as_ref()])
        .output()?;

    let output = String::from_utf8(command.stdout)?
        .split_whitespace()
        .skip_while(|part| part != &"src")
        .into_iter()
        .skip(1)
        .map(|ip| ip.to_string())
        .next();

    output.ok_or(String::from("Could not get route information").into())
}

#[cfg(windows)]
pub fn for_host<S: AsRef<str>>(host: S) -> Result<String> {
    let ip: std::net::Ipv4Addr = host.as_ref().parse()?;

    let octets = ip.octets();

    let mut in_addr: IN_ADDR = unsafe { std::mem::zeroed() };
    unsafe {
        in_addr.S_un.S_un_b_mut().s_b1 = octets[3];
        in_addr.S_un.S_un_b_mut().s_b2 = octets[2];
        in_addr.S_un.S_un_b_mut().s_b3 = octets[1];
        in_addr.S_un.S_un_b_mut().s_b4 = octets[0];
    }

    let mut index: DWORD = 0;

    unsafe {
        GetBestInterface(in_addr, &mut index);
    }

    let adapter = ipconfig::get_adapters()?
        .into_iter()
        .filter(|adapter| {
            let name = WideCString::from_str(&format!(r"\DEVICE\TCPIP_{}", adapter.adapter_name()))
                .unwrap();
            let mut idx = 0;
            unsafe {
                GetAdapterIndex(name.as_ptr() as _, &mut idx);
            }
            idx == index
        })
        .next()
        .ok_or(format!("Could not locate adapter #{}", index))?;

    adapter
        .ip_addresses()
        .iter()
        .filter(|addr| addr.is_ipv4())
        .collect::<Vec<_>>()
        .first()
        .ok_or(format!("Could not get IP of adapter #{}", index).into())
        .map(|ip| ip.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        unsafe {
            assert!(super::for_host("204.17.220.5").is_ok());
        }
    }
}
