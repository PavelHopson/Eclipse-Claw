//! Minimal robots.txt policy for polite, bounded crawling.

use std::time::Duration;

use url::Url;

#[derive(Debug, Clone, Default)]
pub struct RobotsPolicy {
    rules: Vec<Rule>,
    crawl_delay: Option<Duration>,
}

#[derive(Debug, Clone)]
struct Rule {
    allow: bool,
    path: String,
}

#[derive(Debug, Default)]
struct Group {
    agents: Vec<String>,
    rules: Vec<Rule>,
    crawl_delay: Option<Duration>,
}

impl RobotsPolicy {
    /// Parse rules for the `eclipse-claw` user agent, falling back to `*`.
    pub fn parse(text: &str) -> Self {
        let mut groups = Vec::new();
        let mut group = Group::default();
        let mut has_directives = false;

        for raw_line in text.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                if !group.agents.is_empty() {
                    groups.push(std::mem::take(&mut group));
                }
                has_directives = false;
                continue;
            }

            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim();

            match key.as_str() {
                "user-agent" => {
                    if has_directives && !group.agents.is_empty() {
                        groups.push(std::mem::take(&mut group));
                        has_directives = false;
                    }
                    group.agents.push(value.to_ascii_lowercase());
                }
                "allow" | "disallow" if !group.agents.is_empty() => {
                    has_directives = true;
                    // Empty Disallow means no restriction.
                    if !value.is_empty() {
                        group.rules.push(Rule {
                            allow: key == "allow",
                            path: value.to_string(),
                        });
                    }
                }
                "crawl-delay" if !group.agents.is_empty() => {
                    has_directives = true;
                    if let Ok(seconds) = value.parse::<f64>()
                        && seconds.is_finite()
                        && seconds >= 0.0
                    {
                        group.crawl_delay = Some(Duration::from_secs_f64(seconds.min(60.0)));
                    }
                }
                _ => {}
            }
        }
        if !group.agents.is_empty() {
            groups.push(group);
        }

        let specific: Vec<&Group> = groups
            .iter()
            .filter(|group| group.agents.iter().any(|agent| agent == "eclipse-claw"))
            .collect();
        let selected: Vec<&Group> = if specific.is_empty() {
            groups
                .iter()
                .filter(|group| group.agents.iter().any(|agent| agent == "*"))
                .collect()
        } else {
            specific
        };

        let rules = selected
            .iter()
            .flat_map(|group| group.rules.iter().cloned())
            .collect();
        let crawl_delay = selected.iter().filter_map(|group| group.crawl_delay).max();

        Self { rules, crawl_delay }
    }

    pub fn allows(&self, url: &Url) -> bool {
        let mut target = url.path().to_string();
        if let Some(query) = url.query() {
            target.push('?');
            target.push_str(query);
        }

        self.rules
            .iter()
            .filter(|rule| robots_pattern_matches(&rule.path, &target))
            .max_by_key(|rule| (rule_specificity(&rule.path), rule.allow))
            .is_none_or(|rule| rule.allow)
    }

    pub fn crawl_delay(&self) -> Option<Duration> {
        self.crawl_delay
    }
}

fn rule_specificity(pattern: &str) -> usize {
    pattern
        .bytes()
        .filter(|byte| !matches!(byte, b'*' | b'$'))
        .count()
}

fn robots_pattern_matches(pattern: &str, target: &str) -> bool {
    let (pattern, anchored) = pattern
        .strip_suffix('$')
        .map_or((pattern.as_bytes(), false), |value| {
            (value.as_bytes(), true)
        });
    wildcard_prefix_match(pattern, target.as_bytes(), anchored)
}

fn wildcard_prefix_match(pattern: &[u8], target: &[u8], anchored: bool) -> bool {
    if pattern.is_empty() {
        return !anchored || target.is_empty();
    }
    if pattern[0] == b'*' {
        return (0..=target.len())
            .any(|offset| wildcard_prefix_match(&pattern[1..], &target[offset..], anchored));
    }
    !target.is_empty()
        && pattern[0] == target[0]
        && wildcard_prefix_match(&pattern[1..], &target[1..], anchored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specific_agent_overrides_wildcard_and_allow_wins_longest_match() {
        let policy = RobotsPolicy::parse(
            "User-agent: *\nDisallow: /\n\nUser-agent: eclipse-claw\nDisallow: /private\nAllow: /private/public\nCrawl-delay: 2\n",
        );
        assert!(policy.allows(&Url::parse("https://example.com/").unwrap()));
        assert!(!policy.allows(&Url::parse("https://example.com/private/a").unwrap()));
        assert!(policy.allows(&Url::parse("https://example.com/private/public/a").unwrap()));
        assert_eq!(policy.crawl_delay(), Some(Duration::from_secs(2)));
    }

    #[test]
    fn empty_disallow_allows_everything() {
        let policy = RobotsPolicy::parse("User-agent: *\nDisallow:\n");
        assert!(policy.allows(&Url::parse("https://example.com/private").unwrap()));
    }

    #[test]
    fn supports_wildcards_and_end_anchors() {
        let policy = RobotsPolicy::parse(
            "User-agent: *\nDisallow: /*?secret=*\nDisallow: /download/*.zip$\nAllow: /download/public.zip$\n",
        );
        assert!(!policy.allows(&Url::parse("https://example.com/page?secret=yes").unwrap()));
        assert!(!policy.allows(&Url::parse("https://example.com/download/private.zip").unwrap()));
        assert!(policy.allows(&Url::parse("https://example.com/download/public.zip").unwrap()));
        assert!(
            policy.allows(&Url::parse("https://example.com/download/file.zip?view=1").unwrap())
        );
    }
}
