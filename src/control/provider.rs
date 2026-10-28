//! Read-only Codex provider detection and effective output-limit policy.

use crate::control::settings::{Tier, ToolBudgetLevel, ToolBudgetPreferences, ToolBudgets};
use std::path::Path;
use toml_edit::Item;

/// Host-side output ceiling used when a provider contract requires protection.
pub const GUARDED_HOST_LIMIT: i64 = 10_000;
/// FastCtx response budget kept below the Guarded host ceiling.
pub const GUARDED_FASTCTX_BUDGET: usize = 9_000;

/// Where the configured provider route originates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderProvenance {
    OfficialOpenAi,
    Azure,
    AmazonBedrock,
    LocalRuntime,
    ThirdPartyRelay,
    Unknown,
}

/// Compaction implementation Codex 0.147.0 selects for the configured provider.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodexCompaction {
    RemoteV2,
    RemoteV1,
    Local,
    Unknown,
}

/// User-facing reason the Guarded policy is active.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardReason {
    LocalCompaction,
    UnverifiedRelay,
}

/// Read-only classification of the provider visible in one Codex configuration file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderDetection {
    /// Selected provider identifier when it could be read.
    pub provider_id: Option<String>,
    /// Friendly provider name when it could be resolved.
    pub provider_name: Option<String>,
    /// Provenance of the selected route.
    pub provenance: ProviderProvenance,
    /// Compaction implementation Codex will select.
    pub codex_compaction: CodexCompaction,
    /// Whether the selected custom definition explicitly carries a base URL.
    pub base_url_configured: bool,
    /// Stable English explanation suitable for Status diagnostics.
    pub detail: String,
}

impl ProviderDetection {
    fn resolved(
        provider_id: impl Into<String>,
        provider_name: impl Into<String>,
        provenance: ProviderProvenance,
        codex_compaction: CodexCompaction,
        base_url_configured: bool,
    ) -> Self {
        let provider_id = provider_id.into();
        let provider_name = provider_name.into();
        let detail = match (provenance, codex_compaction) {
            (ProviderProvenance::OfficialOpenAi, CodexCompaction::RemoteV2) => format!(
                "Codex provider {provider_id} ({provider_name}) is the official OpenAI route and uses remote V2 compaction."
            ),
            (ProviderProvenance::Azure, CodexCompaction::RemoteV2) => format!(
                "Codex provider {provider_id} ({provider_name}) is an Azure route and uses remote V2 compaction."
            ),
            (ProviderProvenance::AmazonBedrock, CodexCompaction::RemoteV1) => {
                format!("Codex provider {provider_id} ({provider_name}) uses remote V1 compaction.")
            }
            (ProviderProvenance::LocalRuntime, CodexCompaction::Local) => format!(
                "Codex provider {provider_id} ({provider_name}) uses local inline compaction and refeeds the conversation history."
            ),
            (ProviderProvenance::ThirdPartyRelay, CodexCompaction::RemoteV2) => format!(
                "Codex provider {provider_id} ({provider_name}) is a third-party relay declared as OpenAI, so Codex sends remote V2 compaction without a local fallback; FastCtx cannot verify that relay contract."
            ),
            (ProviderProvenance::ThirdPartyRelay, CodexCompaction::Local) => format!(
                "Codex provider {provider_id} ({provider_name}) is a third-party route for which Codex uses local inline compaction; FastCtx cannot verify its model catalog or relay contract."
            ),
            _ => unreachable!("resolved provider dimensions must form a supported classification"),
        };
        Self {
            provider_id: Some(provider_id),
            provider_name: Some(provider_name),
            provenance,
            codex_compaction,
            base_url_configured,
            detail,
        }
    }

    fn unknown(provider_id: Option<String>, detail: impl Into<String>) -> Self {
        Self {
            provider_id,
            provider_name: None,
            provenance: ProviderProvenance::Unknown,
            codex_compaction: CodexCompaction::Unknown,
            base_url_configured: false,
            detail: detail.into(),
        }
    }

    /// Whether the Guarded output policy should be active when protection is enabled.
    pub const fn requires_guard(&self) -> bool {
        matches!(
            self.provenance,
            ProviderProvenance::LocalRuntime | ProviderProvenance::ThirdPartyRelay
        )
    }

    /// Risk statement used by localized control surfaces when Guarded is active.
    pub const fn guard_reason(&self) -> Option<GuardReason> {
        match self.provenance {
            ProviderProvenance::LocalRuntime => Some(GuardReason::LocalCompaction),
            ProviderProvenance::ThirdPartyRelay => Some(GuardReason::UnverifiedRelay),
            _ => None,
        }
    }
}

/// Effective output mode for one provider/configuration snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveOutputMode {
    /// The user's selected tier is effective.
    SelectedTier,
    /// The provider route is unverified or compacts locally, so Guarded ceilings are effective.
    Guarded,
}

/// Concrete output limits consumed by Apply and by one runtime session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectiveOutput {
    /// User preference retained independently from the environment-derived mode.
    pub selected_tier: Tier,
    /// Environment-derived effective mode.
    pub mode: EffectiveOutputMode,
    /// Host token limit written by Apply.
    pub host_limit: i64,
    /// Global FastCtx response budget.
    pub fastctx_budget: usize,
    /// Concrete per-tool shares, including defaults for the effective mode.
    pub tool_budgets: ToolBudgets,
}

/// Reads and classifies the visible Codex config without touching credentials.
pub fn detect_path(path: &Path) -> ProviderDetection {
    match std::fs::read(path) {
        Ok(bytes) => detect_bytes(Some(&bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => detect_bytes(None),
        Err(error) => ProviderDetection::unknown(
            None,
            format!(
                "Cannot read Codex config {} while detecting the model provider: {error}",
                crate::paths::display_path(path)
            ),
        ),
    }
}

/// Classifies a Codex config snapshot; absence means Codex's built-in OpenAI default.
pub fn detect_bytes(config: Option<&[u8]>) -> ProviderDetection {
    let Some(config) = config else {
        return ProviderDetection::resolved(
            "openai",
            "OpenAI",
            ProviderProvenance::OfficialOpenAi,
            CodexCompaction::RemoteV2,
            false,
        );
    };
    let source = match std::str::from_utf8(config) {
        Ok(source) => source,
        Err(error) => {
            return ProviderDetection::unknown(
                None,
                format!(
                    "Codex config is not valid UTF-8 ({error}); provider detection was skipped."
                ),
            );
        }
    };
    let document = match source.parse::<toml_edit::DocumentMut>() {
        Ok(document) => document,
        Err(error) => {
            return ProviderDetection::unknown(
                None,
                format!("Codex config could not be parsed for provider detection: {error}"),
            );
        }
    };
    let provider_id = match document.get("model_provider") {
        None => "openai".to_string(),
        Some(item) => match item.as_str().filter(|value| !value.is_empty()) {
            Some(value) => value.to_string(),
            None => {
                return ProviderDetection::unknown(
                    None,
                    "Codex config key model_provider is not a non-empty string; provider detection was skipped.",
                );
            }
        },
    };

    // Mirrors Codex rust-v0.147.0 provider selection: configured providers choose V2 when
    // `is_openai()` or Azure matches and otherwise declare compaction unsupported
    // (`codex-rs/model-provider/src/provider.rs:300-306`). Bedrock independently implements
    // remote V1 (`codex-rs/model-provider/src/amazon_bedrock/mod.rs:130`). The official-vs-relay
    // distinction follows `codex-rs/tui/src/status/card.rs:922` (`is_openai() && base_url.is_none()`).
    match provider_id.as_str() {
        "amazon-bedrock" => {
            return ProviderDetection::resolved(
                provider_id,
                "Amazon Bedrock",
                ProviderProvenance::AmazonBedrock,
                CodexCompaction::RemoteV1,
                false,
            );
        }
        "ollama" => {
            return ProviderDetection::resolved(
                provider_id,
                "Ollama",
                ProviderProvenance::LocalRuntime,
                CodexCompaction::Local,
                false,
            );
        }
        "lmstudio" => {
            return ProviderDetection::resolved(
                provider_id,
                "LM Studio",
                ProviderProvenance::LocalRuntime,
                CodexCompaction::Local,
                false,
            );
        }
        _ => {}
    }

    let provider = document
        .get("model_providers")
        .and_then(Item::as_table_like)
        .and_then(|providers| providers.get(&provider_id))
        .and_then(Item::as_table_like);
    if provider_id == "openai" && provider.is_none() {
        return ProviderDetection::resolved(
            provider_id,
            "OpenAI",
            ProviderProvenance::OfficialOpenAi,
            CodexCompaction::RemoteV2,
            false,
        );
    }
    let Some(provider) = provider else {
        return ProviderDetection::unknown(
            Some(provider_id.clone()),
            format!(
                "Codex model_provider {provider_id} has no matching model_providers.{provider_id} definition; output protection was not tightened."
            ),
        );
    };
    let Some(name) = provider
        .get("name")
        .and_then(Item::as_str)
        .filter(|name| !name.is_empty())
    else {
        return ProviderDetection::unknown(
            Some(provider_id.clone()),
            format!(
                "Codex provider model_providers.{provider_id}.name is missing or invalid; output protection was not tightened."
            ),
        );
    };
    let base_url = match provider.get("base_url") {
        None => None,
        Some(item) => match item.as_str() {
            Some(value) => Some(value),
            None => {
                return ProviderDetection::unknown(
                    Some(provider_id.clone()),
                    format!(
                        "Codex provider model_providers.{provider_id}.base_url is not a string; output protection was not tightened."
                    ),
                );
            }
        },
    };
    let base_url_configured = base_url.is_some();
    let (provenance, codex_compaction) = if is_azure_responses_provider(name, base_url) {
        (ProviderProvenance::Azure, CodexCompaction::RemoteV2)
    } else if name == "OpenAI" && !base_url_configured {
        (
            ProviderProvenance::OfficialOpenAi,
            CodexCompaction::RemoteV2,
        )
    } else if name == "OpenAI" {
        (
            ProviderProvenance::ThirdPartyRelay,
            CodexCompaction::RemoteV2,
        )
    } else {
        (ProviderProvenance::ThirdPartyRelay, CodexCompaction::Local)
    };
    ProviderDetection::resolved(
        provider_id,
        name,
        provenance,
        codex_compaction,
        base_url_configured,
    )
}

/// Resolves the user's preference against the provider-derived protection state.
pub fn effective_output(
    selected_tier: Tier,
    preferences: ToolBudgetPreferences,
    guard_enabled: bool,
    detection: &ProviderDetection,
) -> EffectiveOutput {
    if guard_enabled && detection.requires_guard() {
        let defaults = ToolBudgets {
            read: ToolBudgetLevel::Inherit,
            grep: ToolBudgetLevel::Inherit,
            glob: ToolBudgetLevel::Inherit,
            run: ToolBudgetLevel::Inherit,
            job_output: ToolBudgetLevel::Inherit,
        };
        EffectiveOutput {
            selected_tier,
            mode: EffectiveOutputMode::Guarded,
            host_limit: GUARDED_HOST_LIMIT,
            fastctx_budget: GUARDED_FASTCTX_BUDGET,
            tool_budgets: ToolBudgets {
                read: preferences.read.unwrap_or(defaults.read),
                grep: preferences.grep.unwrap_or(defaults.grep),
                glob: preferences.glob.unwrap_or(defaults.glob),
                run: preferences.run.unwrap_or(defaults.run),
                job_output: preferences.job_output.unwrap_or(defaults.job_output),
            },
        }
    } else {
        EffectiveOutput {
            selected_tier,
            mode: EffectiveOutputMode::SelectedTier,
            host_limit: selected_tier.host_limit(),
            fastctx_budget: selected_tier.fastctx_budget(),
            tool_budgets: preferences.resolve(selected_tier),
        }
    }
}

// Keep this byte-for-byte aligned with Codex rust-v0.147.0
// `codex-rs/model-provider/src/provider.rs:327-350` `is_azure_responses_provider`. A mismatch
// changes which remote compaction implementation Codex selects.
fn is_azure_responses_provider(name: &str, base_url: Option<&str>) -> bool {
    if name.eq_ignore_ascii_case("azure") {
        return true;
    }
    let Some(base_url) = base_url else {
        return false;
    };
    let base_url = base_url.to_ascii_lowercase();
    [
        "openai.azure.",
        "cognitiveservices.azure.",
        "aoai.azure.",
        "azure-api.",
        "azurefd.",
        "windows.net/openai",
    ]
    .iter()
    .any(|marker| base_url.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::{
        CodexCompaction, EffectiveOutputMode, GUARDED_FASTCTX_BUDGET, GUARDED_HOST_LIMIT,
        ProviderProvenance, detect_bytes, effective_output,
    };
    use crate::control::settings::{Tier, ToolBudgetLevel, ToolBudgetPreferences};

    #[test]
    fn provider_detection_matches_codex_0_147_0_compaction_matrix() {
        let cases = [
            (
                None,
                ProviderProvenance::OfficialOpenAi,
                CodexCompaction::RemoteV2,
                false,
            ),
            (
                Some(b"theme = 'dark'\n".as_slice()),
                ProviderProvenance::OfficialOpenAi,
                CodexCompaction::RemoteV2,
                false,
            ),
            (
                Some(b"model_provider = 'openai'\n".as_slice()),
                ProviderProvenance::OfficialOpenAi,
                CodexCompaction::RemoteV2,
                false,
            ),
            (
                Some(
                    b"model_provider='openai'\n[model_providers.openai]\nname='OpenAI'\nbase_url='https://relay.example/v1'\n"
                        .as_slice(),
                ),
                ProviderProvenance::ThirdPartyRelay,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(b"model_provider = 'amazon-bedrock'\n".as_slice()),
                ProviderProvenance::AmazonBedrock,
                CodexCompaction::RemoteV1,
                false,
            ),
            (
                Some(b"model_provider = 'ollama'\n".as_slice()),
                ProviderProvenance::LocalRuntime,
                CodexCompaction::Local,
                false,
            ),
            (
                Some(b"model_provider = 'lmstudio'\n".as_slice()),
                ProviderProvenance::LocalRuntime,
                CodexCompaction::Local,
                false,
            ),
            (
                Some(
                    b"model_provider='custom'\n[model_providers.custom]\nname='OpenAI'\n"
                        .as_slice(),
                ),
                ProviderProvenance::OfficialOpenAi,
                CodexCompaction::RemoteV2,
                false,
            ),
            (
                Some(
                    b"model_provider='custom'\n[model_providers.custom]\nname='OpenAI'\nbase_url='https://relay.example/v1'\n"
                        .as_slice(),
                ),
                ProviderProvenance::ThirdPartyRelay,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(
                    b"model_provider='custom'\n[model_providers.custom]\nname='openai'\n"
                        .as_slice(),
                ),
                ProviderProvenance::ThirdPartyRelay,
                CodexCompaction::Local,
                false,
            ),
            (
                Some(
                    b"model_provider='az'\n[model_providers.az]\nname='aZuRe'\nbase_url='https://example.test'\n"
                        .as_slice(),
                ),
                ProviderProvenance::Azure,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(
                    b"model_provider='az'\n[model_providers.az]\nname='proxy'\nbase_url='https://EXAMPLE.OPENAI.AZURE.COM/openai'\n"
                        .as_slice(),
                ),
                ProviderProvenance::Azure,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(
                    b"model_provider='az'\n[model_providers.az]\nname='proxy'\nbase_url='https://host.cognitiveservices.azure.cn/openai'\n"
                        .as_slice(),
                ),
                ProviderProvenance::Azure,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(
                    b"model_provider='az'\n[model_providers.az]\nname='proxy'\nbase_url='https://host.aoai.azure.com/openai'\n"
                        .as_slice(),
                ),
                ProviderProvenance::Azure,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(
                    b"model_provider='az'\n[model_providers.az]\nname='proxy'\nbase_url='https://gateway.azure-api.net/openai'\n"
                        .as_slice(),
                ),
                ProviderProvenance::Azure,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(
                    b"model_provider='az'\n[model_providers.az]\nname='proxy'\nbase_url='https://edge.azurefd.net/'\n"
                        .as_slice(),
                ),
                ProviderProvenance::Azure,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(
                    b"model_provider='az'\n[model_providers.az]\nname='proxy'\nbase_url='https://host.windows.net/openai/deployments/x'\n"
                        .as_slice(),
                ),
                ProviderProvenance::Azure,
                CodexCompaction::RemoteV2,
                true,
            ),
            (
                Some(
                    b"model_provider='az'\n[model_providers.az]\nname='proxy'\nbase_url='https://host.azurewebsites.net/openai'\n"
                        .as_slice(),
                ),
                ProviderProvenance::ThirdPartyRelay,
                CodexCompaction::Local,
                true,
            ),
        ];
        for (source, provenance, codex_compaction, base_url_configured) in cases {
            let detected = detect_bytes(source);
            assert_eq!(detected.provenance, provenance, "{source:?}: {detected:?}");
            assert_eq!(
                detected.codex_compaction, codex_compaction,
                "{source:?}: {detected:?}"
            );
            assert_eq!(
                detected.base_url_configured, base_url_configured,
                "{source:?}: {detected:?}"
            );
            assert_eq!(
                detected.requires_guard(),
                matches!(
                    provenance,
                    ProviderProvenance::LocalRuntime | ProviderProvenance::ThirdPartyRelay
                )
            );
        }
    }

    #[test]
    fn malformed_or_unresolved_providers_never_tighten_silently() {
        for source in [
            b"[broken".as_slice(),
            b"model_provider = 1\n".as_slice(),
            b"model_provider = 'missing'\n".as_slice(),
            b"model_provider='custom'\n[model_providers.custom]\nbase_url='https://example.test'\n"
                .as_slice(),
            b"model_provider='custom'\n[model_providers.custom]\nname='Third Party'\nbase_url=1\n"
                .as_slice(),
        ] {
            let detected = detect_bytes(Some(source));
            assert_eq!(
                detected.provenance,
                ProviderProvenance::Unknown,
                "{detected:?}"
            );
            assert_eq!(detected.codex_compaction, CodexCompaction::Unknown);
            assert!(!detected.requires_guard());
            assert!(!detected.detail.is_empty());
        }
    }

    #[test]
    fn guarded_output_preserves_the_tier_preference_and_explicit_shares() {
        let detection = detect_bytes(Some(
            b"model_provider='custom'\n[model_providers.custom]\nname='Third Party'\n",
        ));
        let preferences = ToolBudgetPreferences {
            grep: Some(ToolBudgetLevel::Percent(37)),
            ..ToolBudgetPreferences::default()
        };
        let guarded = effective_output(Tier::High, preferences, true, &detection);
        assert_eq!(guarded.selected_tier, Tier::High);
        assert_eq!(guarded.mode, EffectiveOutputMode::Guarded);
        assert_eq!(guarded.host_limit, GUARDED_HOST_LIMIT);
        assert_eq!(guarded.fastctx_budget, GUARDED_FASTCTX_BUDGET);
        assert_eq!(guarded.tool_budgets.read, ToolBudgetLevel::Inherit);
        assert_eq!(guarded.tool_budgets.grep, ToolBudgetLevel::Percent(37));
        assert_eq!(guarded.tool_budgets.glob, ToolBudgetLevel::Inherit);

        let disabled = effective_output(Tier::High, preferences, false, &detection);
        assert_eq!(disabled.mode, EffectiveOutputMode::SelectedTier);
        assert_eq!(disabled.host_limit, Tier::High.host_limit());
        assert_eq!(
            disabled.tool_budgets.glob,
            Tier::High.default_budgets().glob
        );
    }
}
