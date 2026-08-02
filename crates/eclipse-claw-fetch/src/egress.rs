//! Fail-closed outbound network policy for user-supplied URLs.
//!
//! The policy rejects local, private, link-local, documentation and other
//! non-public address ranges. A custom wreq DNS resolver applies the same
//! decision to the addresses used by the actual connection, which avoids the
//! usual "validate, then resolve again" DNS-rebinding gap.

use std::error::Error;
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use tracing::{info, warn};
use url::{Host, Url};
use wreq::dns::{Addrs, Name, Resolve, Resolving};

use crate::error::FetchError;

/// Whether requests may reach non-public network ranges.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum NetworkPolicy {
    /// Internet destinations only. This is the safe default for REST and MCP.
    #[default]
    PublicOnly,
    /// Explicit local CLI escape hatch for trusted development environments.
    AllowPrivate,
}

impl NetworkPolicy {
    pub fn allows_private(self) -> bool {
        matches!(self, Self::AllowPrivate)
    }
}

/// Parse and validate a URL before a request is constructed.
pub fn validate_url(url: &str, policy: NetworkPolicy) -> Result<Url, FetchError> {
    let parsed = Url::parse(url).map_err(|_| FetchError::InvalidUrl(redact_url(url)))?;

    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(denied(&parsed, "only http and https schemes are allowed"));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(denied(&parsed, "URL credentials are not allowed"));
    }

    let host = parsed
        .host()
        .ok_or_else(|| denied(&parsed, "URL host is required"))?;

    match host {
        Host::Domain(domain) => {
            validate_domain(domain, policy).map_err(|reason| denied(&parsed, reason))?
        }
        Host::Ipv4(ip) => {
            validate_ip(IpAddr::V4(ip), policy).map_err(|reason| denied(&parsed, reason))?
        }
        Host::Ipv6(ip) => {
            validate_ip(IpAddr::V6(ip), policy).map_err(|reason| denied(&parsed, reason))?
        }
    }

    Ok(parsed)
}

/// Resolve a URL now and validate every returned address. This is used for
/// browser navigation where the HTTP client cannot install our DNS resolver.
pub async fn validate_resolved_url(url: &str, policy: NetworkPolicy) -> Result<Url, FetchError> {
    let parsed = validate_url(url, policy)?;
    if policy.allows_private() || matches!(parsed.host(), Some(Host::Ipv4(_) | Host::Ipv6(_))) {
        return Ok(parsed);
    }

    let host = parsed.host_str().expect("validated URL has host");
    let port = parsed.port_or_known_default().unwrap_or(80);
    let resolved: Vec<SocketAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|_| FetchError::PolicyDenied("DNS resolution failed".into()))?
        .collect();
    if resolved.is_empty() {
        return Err(FetchError::PolicyDenied("DNS returned no addresses".into()));
    }
    if resolved.iter().any(|address| !is_public_ip(address.ip())) {
        return Err(denied(&parsed, "DNS resolved to a non-public IP address"));
    }

    Ok(parsed)
}

/// Safe value for structured logs: scheme + host + explicit port, never path,
/// query, fragment, credentials, headers or content.
pub fn audit_target(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| {
            let host = u.host_str()?;
            let port = u.port().map(|p| format!(":{p}")).unwrap_or_default();
            Some(format!("{}://{host}{port}", u.scheme()))
        })
        .unwrap_or_else(|| "invalid-url".to_string())
}

/// DNS resolver used by the transport itself. If a hostname resolves to a mix
/// of public and private addresses, the entire request is rejected.
#[derive(Debug, Clone, Copy)]
pub struct PolicyResolver {
    policy: NetworkPolicy,
}

impl PolicyResolver {
    pub fn new(policy: NetworkPolicy) -> Self {
        Self { policy }
    }
}

impl Resolve for PolicyResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        let policy = self.policy;

        Box::pin(async move {
            validate_domain(&host, policy).map_err(boxed_io)?;

            let resolved: Vec<SocketAddr> = tokio::net::lookup_host((host.as_str(), 0))
                .await
                .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
                .collect();

            if resolved.is_empty() {
                return Err(boxed_io("DNS returned no addresses"));
            }

            for address in &resolved {
                if let Err(reason) = validate_ip(address.ip(), policy) {
                    warn!(
                        security_event = "egress_denied",
                        target_host = %host,
                        reason,
                        "blocked DNS result"
                    );
                    return Err(boxed_io(reason));
                }
            }

            info!(
                security_event = "egress_allowed",
                target_host = %host,
                resolved_addresses = resolved.len(),
                "outbound destination approved"
            );

            let addrs: Addrs = Box::new(resolved.into_iter());
            Ok(addrs)
        })
    }
}

fn validate_domain(domain: &str, policy: NetworkPolicy) -> Result<(), &'static str> {
    if policy.allows_private() {
        return Ok(());
    }

    let lower = domain.trim_end_matches('.').to_ascii_lowercase();
    if lower == "localhost"
        || lower.ends_with(".localhost")
        || lower.ends_with(".local")
        || lower.ends_with(".internal")
        || lower.ends_with(".home.arpa")
    {
        return Err("local hostnames are blocked");
    }

    Ok(())
}

fn validate_ip(ip: IpAddr, policy: NetworkPolicy) -> Result<(), &'static str> {
    if policy.allows_private() || is_public_ip(ip) {
        Ok(())
    } else {
        Err("non-public IP addresses are blocked")
    }
}

/// Conservative public-address check. Unknown/special ranges are denied.
pub fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => is_public_v4(ip),
        IpAddr::V6(ip) => is_public_v6(ip),
    }
}

fn is_public_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, d] = ip.octets();

    !(a == 0
        || a == 10
        || a == 127
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 192 && b == 88 && c == 99)
        || (a == 192 && b == 168)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 224
        || (a == 255 && b == 255 && c == 255 && d == 255))
}

fn is_public_v6(ip: Ipv6Addr) -> bool {
    if let Some(v4) = ip.to_ipv4_mapped() {
        return is_public_v4(v4);
    }

    let segments = ip.segments();
    !(ip.is_unspecified()
        || ip.is_loopback()
        || ip.is_multicast()
        || (segments[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
        || (segments[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
        || (segments[0] & 0xffc0) == 0xfec0 // deprecated site-local fec0::/10
        || (segments[0] == 0x2001 && segments[1] == 0x0db8) // documentation
        || (segments[0] == 0x0100 && segments[1..] == [0; 7])) // discard-only 100::/64 subset
}

fn denied(parsed: &Url, reason: &'static str) -> FetchError {
    warn!(
        security_event = "egress_denied",
        target = %audit_target(parsed.as_str()),
        reason,
        "outbound URL rejected"
    );
    FetchError::PolicyDenied(reason.to_string())
}

fn boxed_io(message: &'static str) -> Box<dyn Error + Send + Sync> {
    Box::new(io::Error::new(io::ErrorKind::PermissionDenied, message))
}

fn redact_url(url: &str) -> String {
    audit_target(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_https_is_allowed_without_dns() {
        let parsed = validate_url(
            "https://example.com/path?token=secret",
            NetworkPolicy::PublicOnly,
        )
        .expect("public URL should pass syntax policy");
        assert_eq!(parsed.host_str(), Some("example.com"));
        assert_eq!(audit_target(parsed.as_str()), "https://example.com");
    }

    #[test]
    fn blocks_credentials_local_names_and_non_http_schemes() {
        for url in [
            "http://user:pass@example.com/",
            "http://localhost/admin",
            "http://api.service.local/",
            "file:///etc/passwd",
            "gopher://example.com/",
        ] {
            assert!(
                validate_url(url, NetworkPolicy::PublicOnly).is_err(),
                "{url}"
            );
        }
    }

    #[test]
    fn blocks_private_metadata_and_special_ip_literals() {
        for url in [
            "http://127.0.0.1/",
            "http://10.0.0.1/",
            "http://172.16.0.1/",
            "http://192.168.1.1/",
            "http://169.254.169.254/latest/meta-data/",
            "http://100.64.0.1/",
            "http://192.0.2.1/",
            "http://[::1]/",
            "http://[fe80::1]/",
            "http://[fc00::1]/",
            "http://[::ffff:127.0.0.1]/",
            "http://2130706433/",
            "http://0x7f000001/",
            "http://0177.0.0.1/",
        ] {
            assert!(
                validate_url(url, NetworkPolicy::PublicOnly).is_err(),
                "{url}"
            );
        }
    }

    #[test]
    fn private_network_requires_explicit_policy() {
        assert!(validate_url("http://127.0.0.1:3000", NetworkPolicy::AllowPrivate).is_ok());
    }
}
