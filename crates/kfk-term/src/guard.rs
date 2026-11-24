//! who may open a terminal: this machine only, and only from kfk's own pages

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Deny {
    /// `web_shell` is off
    Off,
    /// the web view listens on a network address
    Network,
    /// the request names a host that is not this machine (DNS rebinding)
    Host,
    /// the request comes from another site's page
    Origin,
}

impl Deny {
    pub fn why(self) -> &'static str {
        match self {
            Deny::Off => "Terminals are turned off (kfk config web_shell on).",
            Deny::Network => "Terminals work only when the web view listens on this machine (web_addr 127.0.0.1); it listens on a network address, so this view is read-only.",
            Deny::Host => "Terminals are served only to this machine's own address (127.0.0.1 or localhost).",
            Deny::Origin => "Terminals are served only to kfk's own pages.",
        }
    }
}

pub struct Ask<'a> {
    /// config `web_shell` is not off
    pub enabled: bool,
    /// the server listens on a loopback address
    pub local: bool,
    pub host: Option<&'a str>,
    pub origin: Option<&'a str>,
}

/// can this server offer terminals at all? decides whether a page shows one
pub fn usable(enabled: bool, local: bool) -> Result<(), Deny> {
    match (enabled, local) {
        (false, _) => Err(Deny::Off),
        (_, false) => Err(Deny::Network),
        _ => Ok(()),
    }
}

/// may this request open a terminal?
pub fn admit(a: &Ask) -> Result<(), Deny> {
    usable(a.enabled, a.local)?;
    let host = a.host.filter(|h| loopback_host(h)).ok_or(Deny::Host)?;
    match a.origin {
        // a browser always says where a page came from; a local program without one could run tmux itself
        None => Ok(()),
        Some(o) if o.strip_prefix("http://").or_else(|| o.strip_prefix("https://")) == Some(host) => Ok(()),
        Some(_) => Err(Deny::Origin),
    }
}

/// `127.0.0.1:7878`, `localhost`, `[::1]:7878`: a name that can only mean this machine
pub fn loopback_host(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(v6) => v6.split(']').next().unwrap_or(""),
        None => host.rsplit_once(':').map(|x| x.0).unwrap_or(host),
    };
    name.eq_ignore_ascii_case("localhost") || name.parse::<std::net::IpAddr>().map(|ip| ip.is_loopback()).unwrap_or(false)
}
