//! Every request goes through here: identity, robots.txt, a pause per host, retries
//! and limits, as `job_radar/http.py`. The transport is a trait so tests inject
//! responses and never touch the network.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::time::{Duration, Instant};

use crate::crawler::robots::{self, PRODUCT_TOKEN};

pub const MAX_BODY: usize = 10 * 1024 * 1024;
pub const USER_AGENT: &str = concat!(
    "job-radar-desktop/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/nandomoreirame/job-radar-desktop)"
);
const MIN_PAUSE: f64 = 1.0;
const RETRIES: u32 = 4;
const ROBOTS_RETRIES: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawResponse {
    pub status: u16,
    pub body: Vec<u8>,
    pub retry_after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    BadUrl,
    /// robots.txt disallows the page, or it cannot be read.
    Blocked,
    Status(u16),
    TooLarge,
    Network(String),
}

pub trait Transport {
    fn get(&self, url: &str, accept: &str) -> Result<RawResponse, String>;
    fn sleep(&self, seconds: f64);
    /// Monotonic seconds, for the pause per host.
    fn now(&self) -> f64;
}

#[derive(Debug, Clone, Default)]
pub struct GetOptions<'a> {
    pub accept: Option<&'a str>,
    /// Extra statuses to retry, as Solides needs for its 403 bursts.
    pub retry_statuses: &'a [u16],
    /// Fixed backoff per attempt instead of the exponential default.
    pub fixed_backoff: Option<f64>,
    pub retries: Option<u32>,
}

#[derive(Debug, Clone)]
enum HostPolicy {
    AllowAll,
    Unreachable,
    Rules { body: String, delay: Option<f64> },
}

pub struct Client<T: Transport> {
    transport: T,
    hosts: RefCell<HashMap<String, HostPolicy>>,
    last_request: RefCell<HashMap<String, f64>>,
}

fn host_key(url: &url::Url) -> String {
    let port = url.port().map(|p| format!(":{p}")).unwrap_or_default();
    format!("{}://{}{port}", url.scheme(), url.host_str().unwrap_or("")).to_lowercase()
}

/// Exponential 0.5 to 8 seconds, or Retry-After capped at 60.
fn backoff_seconds(attempt: u32, retry_after: Option<&str>) -> f64 {
    if let Some(value) = retry_after.and_then(|v| v.trim().parse::<f64>().ok()) {
        return value.clamp(0.0, 60.0);
    }
    // A fixed quarter second stands for Python's random jitter, keeping runs testable.
    (0.5 * 2f64.powi(i32::try_from(attempt).unwrap_or(4))).min(8.0) + 0.25
}

impl<T: Transport> Client<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            hosts: RefCell::new(HashMap::new()),
            last_request: RefCell::new(HashMap::new()),
        }
    }

    fn policy(&self, url: &url::Url) -> HostPolicy {
        let key = host_key(url);
        if let Some(policy) = self.hosts.borrow().get(&key) {
            return policy.clone();
        }
        let robots_url = format!("{key}/robots.txt");
        let mut fetched = None;
        for attempt in 0..=ROBOTS_RETRIES {
            match self.transport.get(&robots_url, "text/plain,*/*") {
                Ok(raw) if raw.status >= 500 || raw.status < 200 => fetched = Some(raw),
                Ok(raw) => {
                    fetched = Some(raw);
                    break;
                }
                Err(_) => fetched = None,
            }
            if attempt < ROBOTS_RETRIES {
                self.transport
                    .sleep((0.5 * 2f64.powi(i32::try_from(attempt).unwrap_or(2))).min(8.0));
            }
        }
        let policy = match fetched {
            None => HostPolicy::Unreachable,
            // RFC 9309 §2.3.1.3: a 4xx publishes no restriction.
            Some(raw) if (400..500).contains(&raw.status) => HostPolicy::AllowAll,
            Some(raw) if raw.status >= 500 || raw.status < 200 => HostPolicy::Unreachable,
            Some(raw) => {
                let body = String::from_utf8_lossy(&raw.body).into_owned();
                if robots::is_robots_body(&body) {
                    let delay = robots::crawl_delay(&body, PRODUCT_TOKEN);
                    HostPolicy::Rules { body, delay }
                } else {
                    HostPolicy::Unreachable
                }
            }
        };
        self.hosts.borrow_mut().insert(key, policy.clone());
        policy
    }

    fn pace(&self, key: &str, delay: Option<f64>) {
        let pause = MIN_PAUSE.max(delay.unwrap_or(0.0));
        if let Some(last) = self.last_request.borrow().get(key) {
            let wait = pause - (self.transport.now() - last);
            if wait > 0.0 {
                self.transport.sleep(wait);
            }
        }
    }

    pub fn get(&self, raw_url: &str, options: GetOptions<'_>) -> Result<Response, HttpError> {
        let url = url::Url::parse(raw_url).map_err(|_| HttpError::BadUrl)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(HttpError::BadUrl);
        }
        let delay = match self.policy(&url) {
            HostPolicy::Unreachable => return Err(HttpError::Blocked),
            HostPolicy::AllowAll => None,
            HostPolicy::Rules { body, delay } => {
                if !robots::allowed(&body, PRODUCT_TOKEN, &robots::path_of(raw_url)) {
                    return Err(HttpError::Blocked);
                }
                delay
            }
        };
        let key = host_key(&url);
        self.pace(&key, delay);
        let retries = options.retries.unwrap_or(RETRIES);
        let accept = options.accept.unwrap_or("*/*");
        let mut last_error = HttpError::Network("no attempt".into());
        for attempt in 0..=retries {
            let (retry_after, error) = match self.transport.get(raw_url, accept) {
                Ok(raw) if (200..300).contains(&raw.status) => {
                    if raw.body.len() > MAX_BODY {
                        return Err(HttpError::TooLarge);
                    }
                    self.last_request
                        .borrow_mut()
                        .insert(key, self.transport.now());
                    return Ok(Response { body: raw.body });
                }
                Ok(raw) => {
                    let retryable = raw.status == 429
                        || (500..600).contains(&raw.status)
                        || options.retry_statuses.contains(&raw.status);
                    if !retryable {
                        return Err(HttpError::Status(raw.status));
                    }
                    (raw.retry_after, HttpError::Status(raw.status))
                }
                Err(message) => (None, HttpError::Network(message)),
            };
            last_error = error;
            if attempt == retries {
                break;
            }
            let wait = options.fixed_backoff.map_or_else(
                || backoff_seconds(attempt, retry_after.as_deref()),
                |base| base * f64::from(attempt + 1),
            );
            self.transport.sleep(wait);
        }
        Err(last_error)
    }
}

/// The real transport: rustls, 25 s timeout, at most 5 redirects, bounded body.
pub struct UreqTransport {
    agent: ureq::Agent,
    started: Instant,
}

impl Default for UreqTransport {
    fn default() -> Self {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(25)))
            .max_redirects(5)
            .http_status_as_error(false)
            .user_agent(USER_AGENT)
            .build()
            .into();
        Self {
            agent,
            started: Instant::now(),
        }
    }
}

impl Transport for UreqTransport {
    fn get(&self, url: &str, accept: &str) -> Result<RawResponse, String> {
        let mut response = self
            .agent
            .get(url)
            .header("Accept", accept)
            .call()
            .map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let mut body = Vec::new();
        // One byte over the limit is enough to know it is too large.
        response
            .body_mut()
            .as_reader()
            .take(MAX_BODY as u64 + 1)
            .read_to_end(&mut body)
            .map_err(|e| e.to_string())?;
        Ok(RawResponse {
            status,
            body,
            retry_after,
        })
    }

    fn sleep(&self, seconds: f64) {
        std::thread::sleep(Duration::from_secs_f64(seconds.max(0.0)));
    }

    fn now(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    #[derive(Default)]
    struct Fake {
        responses: RefCell<VecDeque<(String, Result<RawResponse, String>)>>,
        requests: RefCell<Vec<String>>,
        slept: RefCell<Vec<f64>>,
    }
    impl Fake {
        fn on(self, url: &str, status: u16, body: &str) -> Self {
            self.responses.borrow_mut().push_back((
                url.to_owned(),
                Ok(RawResponse {
                    status,
                    body: body.as_bytes().to_vec(),
                    retry_after: None,
                }),
            ));
            self
        }
    }
    impl Transport for &Fake {
        fn get(&self, url: &str, _accept: &str) -> Result<RawResponse, String> {
            self.requests.borrow_mut().push(url.to_owned());
            let mut queue = self.responses.borrow_mut();
            let at = queue
                .iter()
                .position(|(u, _)| u == url)
                .ok_or_else(|| format!("unexpected {url}"))?;
            queue.remove(at).map(|(_, r)| r).unwrap()
        }
        fn sleep(&self, seconds: f64) {
            self.slept.borrow_mut().push(seconds);
        }
        fn now(&self) -> f64 {
            self.slept.borrow().iter().sum()
        }
    }

    #[test]
    fn honors_robots_pacing_and_retries() {
        let fake = Fake::default()
            .on(
                "https://a.example/robots.txt",
                200,
                "User-agent: *\nDisallow: /no\nCrawl-delay: 3\n",
            )
            .on("https://a.example/one", 503, "")
            .on("https://a.example/one", 200, "ok")
            .on("https://a.example/two", 200, "ok");
        let client = Client::new(&fake);
        assert_eq!(
            client.get("https://a.example/no", GetOptions::default()),
            Err(HttpError::Blocked)
        );
        assert_eq!(
            client
                .get("https://a.example/one", GetOptions::default())
                .unwrap()
                .body,
            b"ok"
        );
        assert_eq!(
            client
                .get("https://a.example/two", GetOptions::default())
                .unwrap()
                .body,
            b"ok"
        );
        assert!(
            fake.slept.borrow().iter().any(|s| *s >= 3.0),
            "crawl-delay not honored: {:?}",
            fake.slept.borrow()
        );
        assert_eq!(
            fake.requests
                .borrow()
                .iter()
                .filter(|u| u.ends_with("robots.txt"))
                .count(),
            1
        );
        // 404 is not retried and is an answer, not an error of the client.
        let gone = Fake::default()
            .on("https://b.example/robots.txt", 404, "")
            .on("https://b.example/x", 404, "");
        assert_eq!(
            Client::new(&gone).get("https://b.example/x", GetOptions::default()),
            Err(HttpError::Status(404))
        );
    }

    #[test]
    fn bounds_the_body_and_refuses_other_schemes() {
        let fake = Fake::default();
        let client = Client::new(&fake);
        assert_eq!(
            client.get("file:///etc/passwd", GetOptions::default()),
            Err(HttpError::BadUrl)
        );
        assert_eq!(
            client.get("ftp://a.example/x", GetOptions::default()),
            Err(HttpError::BadUrl)
        );
        assert!(fake.requests.borrow().is_empty());
        let big = "x".repeat(MAX_BODY + 1);
        let fake = Fake::default()
            .on("https://c.example/robots.txt", 404, "")
            .on("https://c.example/x", 200, &big);
        assert_eq!(
            Client::new(&fake).get("https://c.example/x", GetOptions::default()),
            Err(HttpError::TooLarge)
        );
    }
}
