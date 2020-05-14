//! local-ip-resolver
//! =================
//!
//! simple interface to get the current ip address used
//! to communicate to a remote host.
//!

#[cfg(windows)]
mod win;
#[cfg(windows)]
pub use win::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(target_os = "macos")]
pub fn for_host<S: AsRef<str>>(host: S) -> Result<String> {
    let host = get_host_ip(host)?;
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
        .ok_or_else(|| String::from("Could not get IP address of interface").into())
}

#[cfg(target_os = "linux")]
pub fn for_host<S: AsRef<str>>(host: S) -> Result<String> {
    let host = get_host_ip(host)?;
    use std::process::Command;

    let command = Command::new("ip")
        .args(&["route", "get", host.as_ref()])
        .output()?;

    let output = String::from_utf8(command.stdout)?
        .split_whitespace()
        .skip_while(|part| part != &"src")
        .skip(1)
        .map(|ip| ip.to_string())
        .next();

    output.ok_or_else(|| String::from("Could not get route information").into())
}

#[cfg(windows)]
pub fn for_host<S: AsRef<str>>(host: S) -> Result<String> {
    let host = get_host_ip(host)?;
    let ip: std::net::Ipv4Addr = host.parse()?;
    let in_addr: u32 = ip.into();
    let mut index: DWORD = 0;

    unsafe {
        GetBestInterface(in_addr, &mut index);
    }

    let ip = unsafe { get_ip_for_interface(index)? };
    Ok(ip)
}

fn get_host_ip<S: AsRef<str>>(host: S) -> Result<String> {
    use std::net::ToSocketAddrs;
    let mut addrs = format!("{}:0", host.as_ref()).to_socket_addrs()?;
    addrs
        .next()
        .map(|ip| ip.ip().to_string())
        .ok_or_else(|| format!("Cannot resolve IP for {}", host.as_ref()).into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(super::for_host("204.17.220.5").is_ok());
    }
}
