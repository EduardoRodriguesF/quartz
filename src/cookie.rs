use crate::QuartzResult;
use chrono::prelude::*;
use hyper::http::uri::Scheme;
use std::{
    collections::HashSet,
    convert::Infallible,
    hash::Hash,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    str::FromStr,
};

pub enum Field {
    Domain,
    Subdomains,
    Path,
    Secure,
    ExpiresAt,
    Name,
    Value,
}

#[derive(Debug, Clone)]
pub struct CookieError;

#[derive(Debug, PartialEq, Eq)]
pub struct Domain(String);

impl Deref for Domain {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Domain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&str> for Domain {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl Domain {
    pub fn new<T>(s: T) -> Self
    where
        T: Into<String>,
    {
        let s = Self::canonicalize(&s.into());

        Self(s)
    }

    /// Standardize representation of a domain name string `value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::cookie::Domain;
    ///
    /// assert_eq!(Domain::canonicalize("www.example.com"), "www.example.com");
    /// assert_eq!(Domain::canonicalize("www.EXAMPLE.com"), "www.example.com");
    /// assert_eq!(Domain::canonicalize("www.example.com\n"), "www.example.com");
    /// assert_eq!(Domain::canonicalize(" www.example.com"), "www.example.com");
    /// assert_eq!(Domain::canonicalize("www....example..com"), "www.example.com");
    /// ```
    // TODO: Allow non-ASCII characters (punycode)
    pub fn canonicalize(value: &str) -> String {
        let value = value.to_ascii_lowercase();
        let value = value.trim();

        let mut res = String::new();

        let mut chars = value.chars();
        let mut last = '*';
        while let Some(ch) = chars.next() {
            if ch == '.' && last == '.' {
                continue;
            }

            last = ch;
            res.push(ch);
        }

        res
    }

    /// Whether this [`Domain`] can set cookies for
    /// `other` according to [RFC 6265](https://datatracker.ietf.org/doc/html/rfc6265).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::cookie::Domain;
    ///
    /// assert!(Domain::new(".example.com").matches("www.example.com"));
    /// assert!(Domain::new(".example.com").matches("example.com"));
    /// assert!(Domain::new("example.com").matches("www.example.com"));
    /// assert!(Domain::new("www.example.com").matches("example.com"));
    /// assert!(Domain::new("sub.sub.sub.example.com").matches("example.com"));
    ///
    /// assert_eq!(Domain::new("example.com").matches("anotherexample.com"), false);
    /// assert_eq!(Domain::new("www.example.com").matches("www2.example.com"), false);
    /// assert_eq!(Domain::new("www.example.com").matches("www.example.com.au"), false);
    /// ```
    #[must_use]
    pub fn matches<T>(&self, other: T) -> bool
    where
        T: Into<Domain>,
    {
        let other: Domain = other.into();

        if **self == *other {
            return true;
        }

        let this_segments: Vec<&str> = self.as_segments().collect();
        let other_segments = other.as_segments();
        for (idx, other_seg) in other_segments.enumerate() {
            if this_segments.len() <= idx {
                break;
            }

            if other_seg != this_segments[idx] {
                return false;
            }
        }

        true
    }

    /// Transforms a string into domain segments from top-level.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::cookie::Domain;
    ///
    /// let domain = Domain::new("www.example.com");
    ///
    /// let expected = vec!["com", "example", "www"];
    /// let result = domain.as_segments().collect::<Vec<&str>>();
    /// assert_eq!(expected, result);
    #[must_use]
    pub fn as_segments(&self) -> impl Iterator<Item = &str> {
        self.split('.').filter(|s| !s.is_empty()).rev()
    }
}

#[derive(Default)]
pub struct CookieBuilder {
    domain: Option<String>,
    subdomains: bool,
    path: Option<String>,
    secure: bool,
    expires_at: i64,
    name: Option<String>,
    value: Option<String>,
}

impl CookieBuilder {
    pub fn domain<T>(&mut self, s: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.domain = Some(s.into());
        self
    }

    pub fn subdomains(&mut self, v: bool) -> &mut Self {
        self.subdomains = v;
        self
    }

    pub fn path<T>(&mut self, s: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.path = Some(s.into());
        self
    }

    pub fn secure(&mut self, v: bool) -> &mut Self {
        self.secure = v;
        self
    }

    pub fn expires_at(&mut self, v: i64) -> &mut Self {
        self.expires_at = v;
        self
    }

    pub fn name<T>(&mut self, s: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.name = Some(s.into());
        self
    }

    pub fn value<T>(&mut self, s: T) -> &mut Self
    where
        T: Into<String>,
    {
        self.value = Some(s.into());
        self
    }

    /// Generate [`Cookie`] from this [`CookieBuilder`] components.
    ///
    /// # Errors
    ///
    /// This function will return an error if builder has any invalid cookie component, such as
    /// missing `domain`, `name`, or `value`.
    pub fn build(self) -> QuartzResult<Cookie, CookieError> {
        let domain = Domain::new(self.domain.ok_or(CookieError)?);
        let name = self.name.ok_or(CookieError)?;
        let value = self.value.ok_or(CookieError)?;

        Ok(Cookie {
            domain,
            subdomains: self.subdomains,
            path: PathAttr::from(self.path.unwrap_or_default().as_str()),
            secure: self.secure,
            expires_at: self.expires_at,
            name,
            value,
        })
    }
}

#[derive(Debug)]
pub struct Cookie {
    domain: Domain,
    subdomains: bool,
    path: PathAttr,
    secure: bool,
    expires_at: i64,
    name: String,
    value: String,
}

impl Eq for Cookie {}

impl PartialEq for Cookie {
    fn eq(&self, other: &Self) -> bool {
        self.domain == other.domain && self.name == other.name
    }
}

impl Hash for Cookie {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.domain.hash(state);
        self.name.hash(state);
    }
}

impl ToString for Cookie {
    /// Converts a given [`Cookie`] into a Netspace HTTP Cookie file line.
    ///
    ///# Examples
    ///
    /// ```
    /// use quartz_cli::cookie::Cookie;
    ///
    /// let mut cookie = Cookie::builder();
    /// cookie
    ///     .domain("httpbin.org")
    ///     .subdomains(true)
    ///     .name("mysecret")
    ///     .value("supersecretkey")
    ///     .path("/somepath");
    ///
    /// let cookie = cookie.build().unwrap();
    ///
    /// assert_eq!(cookie.to_string(),
    /// "httpbin.org\tTRUE\t/somepath\tFALSE\t0\tmysecret\tsupersecretkey");
    /// ```
    fn to_string(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            *self.domain,
            self.subdomains.to_string().to_uppercase(),
            self.path.to_string(),
            self.secure.to_string().to_uppercase(),
            self.expires_at,
            self.name,
            self.value,
        )
    }
}

impl FromStr for Cookie {
    type Err = CookieError;

    /// Parses a string `s` to return a [`Cookie`].
    ///
    /// If parsing succeeds, return the value inside [`Ok`], otherwise
    /// when the string is ill-formatted return an error specific to the
    /// inside [`Err`]. The error type is specific to the implementation of the trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use quartz_cli::cookie::Cookie;
    ///
    /// let s = "httpbin.org\tFALSE\t/somepath\tTRUE\t0\tmycookie\tsecret";
    /// let cookie = Cookie::from_str(s).unwrap();
    ///
    /// assert_eq!(**cookie.domain(), "httpbin.org");
    /// assert_eq!(cookie.subdomains(), false);
    /// assert_eq!(cookie.path().to_string(), "/somepath");
    /// assert_eq!(cookie.secure(), true);
    /// assert_eq!(cookie.name(), "mycookie");
    /// assert_eq!(cookie.value(), "secret");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cookie = Cookie::builder();
        let line: Vec<&str> = s.splitn(7, '\t').collect();

        if line.len() != 7 {
            return Err(CookieError);
        }

        cookie
            .domain(line[Field::Domain as usize])
            .subdomains(line[Field::Subdomains as usize] == "TRUE")
            .path(line[Field::Path as usize])
            .secure(line[Field::Secure as usize] == "TRUE")
            .name(line[Field::Name as usize])
            .value(line[Field::Value as usize]);

        if let Ok(v) = line[4].parse() {
            cookie.expires_at(v);
        }

        cookie.build()
    }
}

impl Cookie {
    pub fn builder() -> CookieBuilder {
        CookieBuilder::default()
    }

    pub fn matches<T>(&self, req: hyper::Request<T>) -> bool {
        if !self.domain().matches(req.uri().host().unwrap_or_default()) {
            return false;
        }

        if self.secure() {
            let scheme = req.uri().scheme().unwrap_or(&Scheme::HTTP);
            if scheme == &Scheme::HTTP {
                return false;
            }
        }

        if !self.path().matches(req.uri().path()) {
            return false;
        }

        true
    }

    /// Whether this cookie is expired.
    pub fn expired(&self) -> bool {
        Utc::now().timestamp_micros() > self.expires_at
    }

    pub fn domain(&self) -> &Domain {
        &self.domain
    }

    pub fn subdomains(&self) -> bool {
        self.subdomains
    }

    pub fn path(&self) -> &PathAttr {
        &self.path
    }

    pub fn secure(&self) -> bool {
        self.secure
    }

    pub fn expires_at(&self) -> i64 {
        self.expires_at
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn value(&self) -> &str {
        self.value.as_ref()
    }
}

#[derive(Default)]
pub struct CookieJar {
    data: HashSet<Cookie>,
    pub path: PathBuf,
}

impl Deref for CookieJar {
    type Target = HashSet<Cookie>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for CookieJar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl ToString for CookieJar {
    fn to_string(&self) -> String {
        let mut jar = String::new();

        for cookie in self.iter() {
            jar.push_str(&cookie.to_string());
            jar.push('\n');
        }

        jar
    }
}

impl CookieJar {
    fn pair(v: &str) -> Option<(&str, &str)> {
        v.trim().split_once('=')
    }

    /// Insert new [`Cookie`] from Set-Cookie `input` from `domain`.
    pub fn set(&mut self, domain: &str, input: &'_ str) {
        for input in input.split(',') {
            let mut cookie = Cookie::builder();
            cookie.domain(domain);

            let (pair, settings) = input.split_once(';').unwrap_or((input, ""));

            let (key, value) =
                Self::pair(pair).unwrap_or_else(|| panic!("malformed cookie: {}", pair));

            cookie.name(key);
            cookie.value(value);

            for v in settings.split(';') {
                let (key, value) = Self::pair(v).unwrap_or((v, ""));

                match key.to_lowercase().as_str() {
                    "domain" => cookie.domain(value),
                    "path" => cookie.path(value),
                    "secure" => cookie.secure(true),
                    "max-age" => cookie
                        .expires_at(value.parse::<i64>().unwrap() + Utc::now().timestamp_micros()),
                    "expires" => cookie.expires_at(
                        DateTime::parse_from_rfc2822(value)
                            .unwrap()
                            .timestamp_micros(),
                    ),
                    _ => &mut cookie,
                };
            }

            let cookie = cookie.build().unwrap();

            if self.contains(&cookie) {
                self.remove(&cookie);
            }

            self.insert(cookie);
        }
    }

    pub fn find_by_name(&self, s: &str) -> Vec<&Cookie> {
        self.iter().filter(|c| c.name() == s).collect()
    }
}

impl CookieJar {
    pub const FILENAME: &'static str = "cookies";

    /// Read [`CookieJar`] struct from Netscape HTTP Cookie file.
    /// Empty, malformed, or commented (starting with "#") lines will be skipped.
    ///
    /// # Errors
    ///
    /// This function will return an error if the file does not exist.
    pub fn read(path: &Path) -> QuartzResult<Self> {
        let mut cookies = Self::default();
        let file = std::fs::read_to_string(path)?;
        let lines = file.lines();

        for line in lines {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Ok(cookie) = Cookie::from_str(line) {
                cookies.insert(cookie);
            }
        }

        cookies.path = path.to_path_buf();
        Ok(cookies)
    }

    /// Write cookie jar contents to environment cookie jar in Netspace HTTP Cookie file format.
    pub fn write(&self) -> std::io::Result<()> {
        self.write_at(&self.path)
    }

    /// Write cookie jar contents to `path` in Netspace HTTP Cookie file format.
    pub fn write_at(&self, path: &Path) -> std::io::Result<()> {
        std::fs::write(path, self.to_string())
    }
}

#[derive(Debug, Default)]
pub struct PathAttr(Vec<String>);

impl Deref for PathAttr {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PathAttr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromStr for PathAttr {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl From<&str> for PathAttr {
    /// Parses a string `s` to return a cookie Path attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::cookie::PathAttr;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(PathAttr::from_str("/").unwrap().len(), 0);
    /// assert_eq!(PathAttr::from_str("invalid").unwrap().len(), 0);
    /// assert_eq!(PathAttr::from_str("http://www.example.com").unwrap().len(), 0);
    /// assert_eq!(*PathAttr::from_str("/somepath").unwrap(), vec!["somepath".to_string()]);
    /// assert_eq!(
    ///     *PathAttr::from_str("http://www.example.com/somepath").unwrap(),
    ///     vec!["somepath".to_string()]
    /// );
    /// ```
    fn from(value: &str) -> Self {
        if let Ok(uri) = hyper::Uri::from_str(value) {
            let path = uri
                .path()
                .split('/')
                .filter(|v| !v.is_empty())
                .map(String::from);

            return Self(path.collect::<Vec<String>>());
        }

        // If the uri-path is empty or if the first character of the uri-
        // path is not a %x2F ("/") character, output %x2F ("/") and skip
        // the remaining steps.
        if !value.starts_with('/') {
            return Self::default();
        }

        let path: Vec<String> = value
            .split('/')
            .filter(|v| !v.is_empty())
            .map(String::from)
            .collect();

        Self(path)
    }
}

impl ToString for PathAttr {
    /// Converts this into a Path attribute-value string.
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::cookie::PathAttr;
    ///
    /// ```
    fn to_string(&self) -> String {
        let mut s = String::from("/");

        s.push_str(&self.join("/"));

        s
    }
}

impl PathAttr {
    /// Whether this cookie-path matches `other` request-path.
    ///
    /// See [RFC 6265](https://datatracker.ietf.org/doc/html/rfc6265#section-1.3).
    ///
    /// # Examples
    ///
    /// ```
    /// use quartz_cli::cookie::PathAttr;
    ///
    /// assert!(PathAttr::from("/").matches("/"));
    /// assert!(PathAttr::from("/").matches("/some/nested/path"));
    /// assert!(PathAttr::from("/some").matches("/some/"));
    /// assert!(PathAttr::from("/some").matches("/some/nested/path"));
    ///
    /// assert_eq!(PathAttr::from("/somepath").matches("/some"), false);
    /// assert_eq!(PathAttr::from("/somepath").matches("/"), false);
    /// assert_eq!(PathAttr::from("/some/nested/path").matches("/"), false);
    /// ```
    #[must_use]
    pub fn matches<T>(&self, other: T) -> bool
    where
        T: Into<PathAttr>,
    {
        if self.is_empty() {
            return true;
        }

        let other: PathAttr = other.into();

        if self.len() > other.len() {
            return false;
        }

        for (idx, p) in self.iter().enumerate() {
            if p != &other[idx] {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn jar_set_overwrite() {
        let mut jar = CookieJar::default();

        jar.set("example.com", "foo=bar");
        jar.set("example.com", "foo=baz");

        let found = jar.find_by_name("foo");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].value(), "baz");
    }

    #[test]
    fn jar_set_same_name_different_domain() {
        let mut jar = CookieJar::default();

        jar.set("example.com", "mycookie=true");
        jar.set("httpbin.org", "mycookie=false");

        let cookies = jar.find_by_name("mycookie");
        assert_eq!(cookies.len(), 2);

        let cookie = cookies
            .iter()
            .find(|c| c.domain().matches("example.com"))
            .expect("did not find cookie");

        assert_eq!(cookie.value(), "true");

        let cookie = cookies
            .iter()
            .find(|c| c.domain().matches("httpbin.org"))
            .expect("did not find cookie");

        assert_eq!(cookie.value(), "false");
    }
}
