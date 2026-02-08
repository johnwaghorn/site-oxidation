use std::net::IpAddr;

pub fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback() || (v6.segments()[0] & 0xfe00) == 0xfc00,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("127.0.0.1", true)]
    #[case("127.255.255.255", true)]
    #[case("10.0.0.1", true)]
    #[case("10.255.255.255", true)]
    #[case("172.16.0.1", true)]
    #[case("172.31.255.255", true)]
    #[case("192.168.0.1", true)]
    #[case("192.168.255.255", true)]
    #[case("169.254.1.1", true)]
    #[case("8.8.8.8", false)]
    #[case("1.1.1.1", false)]
    #[case("93.184.216.34", false)]
    #[case("172.32.0.1", false)]
    #[case("::1", true)]
    #[case("fd00::1", true)]
    #[case("fc00::1", true)]
    #[case("2001:4860:4860::8888", false)]
    #[case("2606:4700:4700::1111", false)]
    fn test_is_private_ip(#[case] ip: &str, #[case] expected: bool) {
        assert_eq!(is_private_ip(&ip.parse::<IpAddr>().unwrap()), expected);
    }
}
