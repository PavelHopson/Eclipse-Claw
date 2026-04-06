//! eclipse-claw-fetch: HTTP client layer with browser TLS fingerprint impersonation.
//! Uses wreq (BoringSSL) for browser-grade TLS + HTTP/2 fingerprinting.
//! Automatically detects PDF responses and delegates to eclipse-claw-pdf.
pub mod browser;
pub mod client;
pub mod crawler;
pub mod document;
pub mod error;
pub mod linkedin;
pub mod proxy;
pub mod reddit;
pub mod sitemap;
pub mod tls;

pub use browser::BrowserProfile;
pub use client::{BatchExtractResult, BatchResult, FetchClient, FetchConfig, FetchResult};
pub use crawler::{CrawlConfig, CrawlResult, CrawlState, Crawler, PageResult};
pub use error::FetchError;
pub use http::HeaderMap;
pub use proxy::{parse_proxy_file, parse_proxy_line};
pub use sitemap::SitemapEntry;
pub use eclipse_claw_pdf::PdfMode;
