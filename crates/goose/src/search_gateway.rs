use crate::policy_engine::DataClassification;
use crate::security::dlp::{DlpScanner, FindingKind};
use anyhow::{bail, Result};
use std::collections::BTreeSet;
use std::net::IpAddr;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchRequest {
    pub query: String,
    pub max_results: usize,
}

impl SearchRequest {
    pub fn new(query: impl Into<String>, max_results: usize) -> Result<Self> {
        let query = query.into().trim().to_string();
        if query.is_empty() {
            bail!("search query must not be empty")
        }
        if query.chars().count() > 2000 {
            bail!("search query exceeds the configured length limit")
        }
        if max_results == 0 {
            bail!("search result count must be positive")
        }
        Ok(Self {
            query,
            max_results: max_results.min(10),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPolicy {
    allowed_hosts: BTreeSet<String>,
    max_results: usize,
}

impl SearchPolicy {
    pub fn new<I, S>(allowed_hosts: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut normalized = BTreeSet::new();
        for host in allowed_hosts {
            let host = host
                .into()
                .trim()
                .trim_end_matches('.')
                .to_ascii_lowercase();
            if host.is_empty() || host.parse::<IpAddr>().is_ok() {
                bail!("search allowlist entries must be domain names")
            }
            normalized.insert(host);
        }
        if normalized.is_empty() {
            bail!("search allowlist must not be empty")
        }
        Ok(Self {
            allowed_hosts: normalized,
            max_results: 10,
        })
    }

    pub fn with_max_results(mut self, max_results: usize) -> Result<Self> {
        if max_results == 0 {
            bail!("search result count must be positive")
        }
        self.max_results = max_results.min(10);
        Ok(self)
    }

    fn allows_host(&self, host: &str) -> bool {
        self.allowed_hosts
            .iter()
            .any(|allowed| host == allowed || host.ends_with(&format!(".{allowed}")))
    }
}

#[derive(Debug, Clone)]
pub struct SearchGateway {
    policy: SearchPolicy,
    dlp: DlpScanner,
}

impl SearchGateway {
    pub fn new(policy: SearchPolicy) -> Self {
        Self {
            policy,
            dlp: DlpScanner::new(),
        }
    }

    pub fn authorize_url(&self, url: &str) -> Result<()> {
        let parsed = Url::parse(url).map_err(|_| anyhow::anyhow!("invalid search URL"))?;
        if parsed.scheme() != "https" {
            bail!("search requests require HTTPS")
        }
        if parsed.port().is_some_and(|port| port != 443) {
            bail!("search requests allow only port 443")
        }
        let Some(host) = parsed.host_str() else {
            bail!("search URL has no host")
        };
        let normalized_host = host.trim_end_matches('.').to_ascii_lowercase();
        if is_private_or_local_host(&normalized_host) {
            bail!("search requests cannot target local or private addresses")
        }
        if !self.policy.allows_host(&normalized_host) {
            bail!("search host is not on the allowlist")
        }
        Ok(())
    }

    pub fn plan(&self, request: SearchRequest, endpoint: &str) -> Result<SearchPlan> {
        self.authorize_url(endpoint)?;
        let report = self.dlp.scan(&request.query);
        if report.classification >= DataClassification::L3HighSensitive {
            bail!("sensitive search queries require local review")
        }
        Ok(SearchPlan {
            query: request.query,
            endpoint: endpoint.to_string(),
            max_results: request.max_results.min(self.policy.max_results),
            classification: report.classification,
        })
    }

    pub fn compress_results(
        &self,
        results: impl IntoIterator<Item = SearchResult>,
        max_results: usize,
    ) -> Vec<CompressedSearchResult> {
        let mut seen = BTreeSet::new();
        let mut compressed = Vec::new();
        for result in results {
            if compressed.len() >= max_results.min(self.policy.max_results) {
                break;
            }
            if self.authorize_url(&result.url).is_err() {
                continue;
            }
            let report = self.dlp.scan(&result.content);
            if report
                .findings
                .iter()
                .any(|finding| finding.kind == FindingKind::PromptInjection)
            {
                continue;
            }
            let content_hash = blake3::hash(result.content.as_bytes()).to_hex().to_string();
            if !seen.insert(content_hash.clone()) {
                continue;
            }
            let excerpt: String = result.content.chars().take(2000).collect();
            compressed.push(CompressedSearchResult {
                title: result.title,
                url: result.url,
                excerpt,
                content_hash,
                classification: report.classification,
            });
        }
        compressed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPlan {
    pub query: String,
    pub endpoint: String,
    pub max_results: usize,
    pub classification: DataClassification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompressedSearchResult {
    pub title: String,
    pub url: String,
    pub excerpt: String,
    pub content_hash: String,
    pub classification: DataClassification,
}

fn is_private_or_local_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host.eq_ignore_ascii_case("local")
    {
        return true;
    }
    host.parse::<IpAddr>().is_ok_and(|ip| match ip {
        IpAddr::V4(ip) => {
            ip.is_loopback()
                || ip.is_private()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
        }
        IpAddr::V6(ip) => {
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || (ip.segments()[0] & 0xfe00) == 0xfc00
                || (ip.segments()[0] & 0xffc0) == 0xfe80
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gateway() -> SearchGateway {
        SearchGateway::new(SearchPolicy::new(["search.example"]).unwrap())
    }

    #[test]
    fn only_allowlisted_https_hosts_are_authorized() {
        assert!(gateway()
            .authorize_url("https://search.example/query?q=policy")
            .is_ok());
        assert!(gateway()
            .authorize_url("https://news.search.example/article")
            .is_ok());
        assert!(gateway()
            .authorize_url("http://search.example/query")
            .is_err());
        assert!(gateway()
            .authorize_url("https://other.example/query")
            .is_err());
        assert!(gateway().authorize_url("https://127.0.0.1/query").is_err());
    }

    #[test]
    fn search_plan_bounds_results_and_classifies_query() {
        let plan = gateway()
            .plan(
                SearchRequest::new("official policy", 50).unwrap(),
                "https://search.example/query",
            )
            .unwrap();

        assert_eq!(plan.max_results, 10);
        assert_eq!(plan.classification, DataClassification::L1Internal);
    }

    #[test]
    fn result_compression_deduplicates_truncates_and_filters_destinations() {
        let long_content = "正文".repeat(2000);
        let results = vec![
            SearchResult {
                title: "one".to_string(),
                url: "https://search.example/a".to_string(),
                content: long_content.clone(),
            },
            SearchResult {
                title: "duplicate".to_string(),
                url: "https://search.example/b".to_string(),
                content: long_content,
            },
            SearchResult {
                title: "private".to_string(),
                url: "https://127.0.0.1/secret".to_string(),
                content: "private".to_string(),
            },
        ];

        let compressed = gateway().compress_results(results, 10);
        assert_eq!(compressed.len(), 1);
        assert_eq!(compressed[0].excerpt.chars().count(), 2000);
    }

    #[test]
    fn query_limits_are_enforced() {
        assert!(SearchRequest::new("", 5).is_err());
        assert!(SearchRequest::new("x".repeat(2001), 5).is_err());
        assert!(SearchRequest::new("query", 0).is_err());
    }
}
