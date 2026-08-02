//! Static, allowlisted connector metadata and read-only diagnostics.
//!
//! The registry never discovers or installs executables, opens a browser profile,
//! validates credentials over the network, or returns secret values. Runtime code
//! provides boolean readiness signals after initializing its built-in components.

use serde::Serialize;

/// Environment variable that explicitly permits automatic cloud fallback.
pub const CLOUD_FALLBACK_ENV: &str = "ECLIPSE_CLAW_CLOUD_FALLBACK";

/// Signals collected by a runtime without exposing credential values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeSignals {
    pub local_fetch_ready: bool,
    pub local_llm_ready: bool,
    pub cloud_key_present: bool,
    pub cloud_fallback_enabled: bool,
}

/// Stable diagnostic document returned by REST and MCP.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub schema_version: &'static str,
    pub status: OverallStatus,
    pub connectors: Vec<ConnectorHealth>,
    pub fallback: FallbackPolicy,
    pub safety: SafetyBoundary,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverallStatus {
    Ready,
    Degraded,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConnectorHealth {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ConnectorKind,
    pub status: ConnectorStatus,
    pub capabilities: &'static [&'static str],
    pub data_boundary: &'static str,
    pub provenance: &'static str,
    pub account_requirement: AccountRequirement,
    pub requires_browser_session: bool,
    pub automatic_fallback_eligible: bool,
    pub reason: &'static str,
    pub next_step: &'static str,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorKind {
    BuiltIn,
    LocalProvider,
    EclipseCloud,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountRequirement {
    None,
    DependsOnProvider,
    Required,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorStatus {
    Ready,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FallbackPolicy {
    pub mode: &'static str,
    pub automatic_cloud_fallback: bool,
    pub order: Vec<&'static str>,
    pub disclosure: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SafetyBoundary {
    pub network_probe_performed: bool,
    pub credentials_exposed: bool,
    pub browser_session_accessed: bool,
    pub dynamic_installation_performed: bool,
}

impl DoctorReport {
    /// Build a deterministic report from already-known runtime state.
    pub fn from_signals(signals: RuntimeSignals) -> Self {
        let cloud_auto = automatic_cloud_fallback_allowed(
            signals.cloud_key_present,
            signals.cloud_fallback_enabled,
        );

        let local_fetch = ConnectorHealth {
            id: "local_http",
            name: "Built-in local HTTP extractor",
            kind: ConnectorKind::BuiltIn,
            status: if signals.local_fetch_ready {
                ConnectorStatus::Ready
            } else {
                ConnectorStatus::Unavailable
            },
            capabilities: &["scrape", "crawl", "map", "batch", "diff", "brand"],
            data_boundary: "local process; requested public URL still receives the HTTP request",
            provenance: "compiled eclipse-claw workspace component",
            account_requirement: AccountRequirement::None,
            requires_browser_session: false,
            automatic_fallback_eligible: false,
            reason: if signals.local_fetch_ready {
                "built-in extractor initialized"
            } else {
                "fetch client failed to initialize"
            },
            next_step: if signals.local_fetch_ready {
                "none"
            } else {
                "inspect startup logs and TLS/runtime dependencies"
            },
        };

        let local_llm = ConnectorHealth {
            id: "local_or_direct_llm",
            name: "Configured LLM provider chain",
            kind: ConnectorKind::LocalProvider,
            status: if signals.local_llm_ready {
                ConnectorStatus::Ready
            } else {
                ConnectorStatus::Unavailable
            },
            capabilities: &["extract", "summarize"],
            data_boundary: "depends on the selected provider; Ollama stays local, cloud providers receive extracted content",
            provenance: "compiled provider adapters with environment-only credentials",
            account_requirement: AccountRequirement::DependsOnProvider,
            requires_browser_session: false,
            automatic_fallback_eligible: false,
            reason: if signals.local_llm_ready {
                "at least one provider initialized"
            } else {
                "no LLM provider initialized"
            },
            next_step: if signals.local_llm_ready {
                "review the selected provider data boundary before sending sensitive content"
            } else {
                "run Ollama or configure one supported provider credential"
            },
        };

        let cloud = ConnectorHealth {
            id: "eclipse_cloud",
            name: "Eclipse Claw Cloud API",
            kind: ConnectorKind::EclipseCloud,
            status: match (signals.cloud_key_present, signals.cloud_fallback_enabled) {
                (false, _) => ConnectorStatus::Unavailable,
                (true, _) => ConnectorStatus::Ready,
            },
            capabilities: &["protected_scrape", "js_render", "search", "research"],
            data_boundary: "URL, extraction options and fetched content may be processed by api.webclaw.io",
            provenance: "first-party Eclipse Claw service; no community connector is loaded",
            account_requirement: AccountRequirement::Required,
            requires_browser_session: false,
            automatic_fallback_eligible: cloud_auto,
            reason: match (signals.cloud_key_present, signals.cloud_fallback_enabled) {
                (false, _) => "cloud credential is not configured",
                (true, false) => {
                    "explicit cloud tools are ready; automatic fallback has no explicit opt-in"
                }
                (true, true) => "credential and explicit automatic-fallback opt-in are present",
            },
            next_step: match (signals.cloud_key_present, signals.cloud_fallback_enabled) {
                (false, _) => {
                    "keep local-only mode or configure a cloud credential for explicit cloud tools"
                }
                (true, false) => {
                    "set ECLIPSE_CLAW_CLOUD_FALLBACK=1 only after approving cloud data transfer"
                }
                (true, true) => {
                    "monitor cloud usage and avoid private or client data without approval"
                }
            },
        };

        let mut order = vec!["local_http"];
        if cloud_auto {
            order.push("eclipse_cloud");
        }

        Self {
            schema_version: "1",
            status: if signals.local_fetch_ready {
                OverallStatus::Ready
            } else {
                OverallStatus::Degraded
            },
            connectors: vec![local_fetch, local_llm, cloud],
            fallback: FallbackPolicy {
                mode: if cloud_auto {
                    "local_then_explicitly_enabled_cloud"
                } else {
                    "local_only"
                },
                automatic_cloud_fallback: cloud_auto,
                order,
                disclosure: "Automatic cloud transfer is disabled unless ECLIPSE_CLAW_CLOUD_FALLBACK is explicitly enabled.",
            },
            safety: SafetyBoundary {
                network_probe_performed: false,
                credentials_exposed: false,
                browser_session_accessed: false,
                dynamic_installation_performed: false,
            },
        }
    }
}

/// A credential authenticates a cloud request but never grants transfer consent.
pub const fn automatic_cloud_fallback_allowed(
    cloud_key_present: bool,
    explicit_consent: bool,
) -> bool {
    cloud_key_present && explicit_consent
}

/// Parse a human-friendly opt-in value without reading or returning any secret.
pub fn explicit_opt_in(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

/// Read only whether the cloud credential is non-empty.
pub fn cloud_key_present() -> bool {
    std::env::var("ECLIPSE_CLAW_API_KEY")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

/// Read the separate automatic-fallback consent flag.
pub fn cloud_fallback_enabled() -> bool {
    explicit_opt_in(std::env::var(CLOUD_FALLBACK_ENV).ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_in_parser_is_fail_closed() {
        for value in [None, Some(""), Some("0"), Some("false"), Some("later")] {
            assert!(!explicit_opt_in(value));
        }
        for value in [Some("1"), Some("TRUE"), Some(" yes "), Some("on")] {
            assert!(explicit_opt_in(value));
        }
    }

    #[test]
    fn cloud_fallback_requires_credential_and_consent() {
        assert!(!automatic_cloud_fallback_allowed(false, false));
        assert!(!automatic_cloud_fallback_allowed(true, false));
        assert!(!automatic_cloud_fallback_allowed(false, true));
        assert!(automatic_cloud_fallback_allowed(true, true));

        let without_consent = DoctorReport::from_signals(RuntimeSignals {
            local_fetch_ready: true,
            cloud_key_present: true,
            ..RuntimeSignals::default()
        });
        assert!(!without_consent.fallback.automatic_cloud_fallback);
        assert_eq!(without_consent.fallback.order, vec!["local_http"]);
        assert_eq!(without_consent.connectors[2].status, ConnectorStatus::Ready);

        let enabled = DoctorReport::from_signals(RuntimeSignals {
            local_fetch_ready: true,
            cloud_key_present: true,
            cloud_fallback_enabled: true,
            ..RuntimeSignals::default()
        });
        assert!(enabled.fallback.automatic_cloud_fallback);
        assert_eq!(enabled.fallback.order, vec!["local_http", "eclipse_cloud"]);
        assert_eq!(enabled.connectors[2].status, ConnectorStatus::Ready);
    }

    #[test]
    fn serialized_doctor_report_contains_no_secret_values() {
        let report = DoctorReport::from_signals(RuntimeSignals {
            local_fetch_ready: true,
            local_llm_ready: true,
            cloud_key_present: true,
            cloud_fallback_enabled: false,
        });
        let json = serde_json::to_string(&report).unwrap();

        assert!(!json.contains("api_key"));
        assert!(!json.contains("token"));
        assert!(!report.safety.network_probe_performed);
        assert!(!report.safety.credentials_exposed);
        assert!(!report.safety.browser_session_accessed);
        assert!(!report.safety.dynamic_installation_performed);
    }
}
