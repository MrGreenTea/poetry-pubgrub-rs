use core::cmp::{Ord, Ordering, PartialOrd};
use core::fmt::Display;
use core::option::Option::{None, Some};
use core::result::Result::{Err, Ok};
use lazy_static::lazy_static;
use pubgrub::version::Version;
use regex::Regex;
use std::fmt::Formatter;
use std::str::FromStr;
use thiserror::Error;

lazy_static! {
   // copied from packaging python package
   pub static ref VERSION_PATTERN: Regex = Regex::new(r"^v?(?:(?:(?P<epoch>[0-9]+)!)?(?P<release>[0-9]+(?:\.[0-9]+)*)(?P<pre>[-_\.]?(?P<pre_l>(a|b|c|rc|alpha|beta|pre|preview))[-_\.]?(?P<pre_n>[0-9]+)?)?(?P<post>(?:-(?P<post_n1>[0-9]+))|(?:[-_\.]?(?P<post_l>post|rev|r)[-_\.]?(?P<post_n2>[0-9]+)?))?(?P<dev>[-_\.]?(?P<dev_l>dev)[-_\.]?(?P<dev_n>[0-9]+)?)?)(?:\+(?P<local>[a-z0-9]+(?:[-_\.][a-z0-9]+)*))?$").unwrap();
}

/// Error creating [SemanticVersion] from [String].
#[derive(Error, Debug, PartialEq)]
pub enum VersionParseError {
    /// [SemanticVersion] must contain major, minor, patch versions.
    #[error("version {full_version} must contain 3 numbers separated by dot")]
    NotThreeParts {
        /// [SemanticVersion] that was being parsed.
        full_version: String,
    },
    /// Wrapper around [ParseIntError](core::num::ParseIntError).
    #[error("cannot parse '{version_part}' in '{full_version}' as u32: {parse_error}")]
    ParseIntError {
        /// [SemanticVersion] that was being parsed.
        full_version: String,
        /// A version part where parsing failed.
        version_part: String,
        /// A specific error resulted from parsing a part of the version as [u32].
        parse_error: String,
    },
    #[error("unknown prerelease '{pre_name} in '{full_version}")]
    PrereleaseParseError {
        full_version: String,
        pre_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PEP440Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub epoch: u32,
    pub pre: Option<(Prerelease, u32)>,
    pub post: Option<u32>,
    pub dev: Option<u32>,
}

impl PEP440Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> PEP440Version {
        return PEP440Version {
            major,
            minor,
            patch,
            epoch: 0,
            pre: None,
            post: None,
            dev: None,
        };
    }

    pub fn zero() -> Self {
        PEP440Version::new(0, 0, 0)
    }

    pub fn one() -> Self {
        PEP440Version::new(1, 0, 0)
    }

    pub fn bump_major(&self) -> Self {
        PEP440Version {
            major: self.major + 1,
            ..*self
        }
    }

    pub fn bump_minor(&self) -> Self {
        PEP440Version {
            minor: self.minor + 1,
            ..*self
        }
    }

    pub fn bump_patch(&self) -> Self {
        PEP440Version {
            patch: self.patch + 1,
            ..*self
        }
    }

    pub fn pre_release(&self, kind: Prerelease) -> Self {
        PEP440Version {
            pre: Some((kind, 0)),
            ..*self
        }
    }

    pub fn bump_post(&self) -> Self {
        let post = match self.post {
            None => 0,
            Some(p) => p + 1,
        };
        PEP440Version {
            post: Some(post),
            ..*self
        }
    }

    pub fn bump_dev(&self) -> Self {
        let dev = match self.dev {
            None => 0,
            Some(d) => d + 1,
        };
        PEP440Version {
            dev: Some(dev),
            ..*self
        }
    }
}

impl Ord for PEP440Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch, self.post, self.dev).cmp(&(
            other.major,
            other.minor,
            other.patch,
            other.post,
            other.dev,
        ))
    }
}

impl PartialOrd for PEP440Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for PEP440Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some((n, v)) = self.pre {
            write!(f, "{}{}", n, v)?
        }
        if let Some(post) = self.post {
            write!(f, "post{}", post)?
        }
        if let Some(dev) = self.dev {
            write!(f, "dev{}", dev)?
        }
        Ok(())
    }
}

impl Version for PEP440Version {
    fn lowest() -> Self {
        PEP440Version::zero()
    }

    fn bump(&self) -> Self {
        match (self.pre, self.post, self.dev) {
            (None, None, None) => PEP440Version::new(self.major, self.minor, self.patch + 1),
            (Some((k, v)), None, None) => PEP440Version {
                pre: Some((k, v + 1)),
                ..*self
            },
            (_, Some(_), None) => self.bump_post(),
            (_, _, Some(_)) => self.bump_dev(),
        }
    }
}

impl FromStr for PEP440Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_u32 = |part: &str| {
            part.parse::<u32>().map_err(|e| Self::Err::ParseIntError {
                full_version: s.to_string(),
                version_part: part.to_string(),
                parse_error: e.to_string(),
            })
        };

        let captures = VERSION_PATTERN.captures(s);
        if let Some(c) = captures {
            if let Some(release) = c.name("release") {
                let dev = match c.name("dev_n") {
                    Some(m) => Some(parse_u32(m.as_str())?),
                    None => None,
                };
                let post = match c.name("post_n2") {
                    Some(m) => Some(parse_u32(m.as_str())?),
                    None => None,
                };
                let pre = match (c.name("pre_l"), c.name("pre_n")) {
                    (Some(name), Some(version)) => {
                        Some((name.as_str().parse()?, parse_u32(version.as_str())?))
                    }
                    (_, _) => None,
                };
                let mut parts = release.as_str().split(".");
                let (major, minor, patch) = match (parts.next(), parts.next(), parts.next()) {
                    (Some(major), Some(minor), Some(patch)) => {
                        (parse_u32(major)?, parse_u32(minor)?, parse_u32(patch)?)
                    }
                    (Some(major), Some(minor), None) => (parse_u32(major)?, parse_u32(minor)?, 0),
                    (Some(major), None, None) => (parse_u32(major)?, 0, 0),
                    _ => {
                        return Err(VersionParseError::NotThreeParts {
                            full_version: s.into(),
                        })
                    }
                };
                return Ok(PEP440Version {
                    major,
                    minor,
                    patch,
                    epoch: 0,
                    pre,
                    dev,
                    post,
                });
            }
        }
        Err(VersionParseError::NotThreeParts {
            full_version: s.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Prerelease {
    Alpha,
    Beta,
    ReleaseCandidate,
}

impl Display for Prerelease {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Prerelease::Alpha => write!(f, "a"),
            Prerelease::Beta => write!(f, "b"),
            Prerelease::ReleaseCandidate => write!(f, "rc"),
        }
    }
}

impl FromStr for Prerelease {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a" | "alpha" => Ok(Prerelease::Alpha),
            "b" | "beta" => Ok(Prerelease::Beta),
            "rc" | "c" | "pre" | "preview" => Ok(Prerelease::ReleaseCandidate),
            _ => Err(VersionParseError::PrereleaseParseError {
                full_version: s.into(),
                pre_name: s.into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::version::VERSION_PATTERN;
    use crate::version::{PEP440Version, Prerelease};
    use pubgrub::version::Version;

    #[test]
    fn version_pattern() {
        let captures = VERSION_PATTERN.captures("1.0.0").unwrap();
        assert_eq!(captures.name("release").unwrap().as_str(), "1.0.0");
    }

    #[test]
    fn version_pattern_dev() {
        let captures = VERSION_PATTERN.captures("1.0.0dev0").unwrap();
        assert_eq!(captures.name("release").unwrap().as_str(), "1.0.0");
        assert_eq!(captures.name("dev").unwrap().as_str(), "dev0");
    }

    #[test]
    fn version_pattern_alpha() {
        let captures = VERSION_PATTERN.captures("1.0.0a0").unwrap();
        assert_eq!(captures.name("pre_l").unwrap().as_str(), "a");
        assert_eq!(captures.name("pre_n").unwrap().as_str(), "0");
    }

    #[test]
    fn parse_version() {
        assert_eq!("0.0.0".parse(), Ok(PEP440Version::zero()));
        assert_eq!("1.0.0".parse(), Ok(PEP440Version::one()));
        assert_eq!("1.0.0dev0".parse(), Ok(PEP440Version::one().bump_dev()));
        assert_eq!("1.0.0post0".parse(), Ok(PEP440Version::one().bump_post()));
        assert_eq!(
            "1.0.0a0".parse(),
            Ok(PEP440Version {
                pre: Some((Prerelease::Alpha, 0)),
                ..PEP440Version::one()
            })
        );
        assert_eq!(
            "0.0.0a0".parse(),
            Ok(PEP440Version {
                pre: Some((Prerelease::Alpha, 0)),
                ..PEP440Version::zero()
            })
        );
    }

    #[test]
    fn format_version() {
        assert_eq!(format!("{}", PEP440Version::zero()), "0.0.0");
        assert_eq!(
            format!("{}", PEP440Version::one().pre_release(Prerelease::Alpha)),
            "1.0.0a0"
        );
        assert_eq!(
            format!("{}", PEP440Version::one().pre_release(Prerelease::Beta)),
            "1.0.0b0"
        );
        assert_eq!(
            format!(
                "{}",
                PEP440Version::one().pre_release(Prerelease::ReleaseCandidate)
            ),
            "1.0.0rc0"
        );
        assert_eq!(
            format!("{}", PEP440Version::zero().bump_post()),
            "0.0.0post0"
        );
        assert_eq!(format!("{}", PEP440Version::zero().bump_dev()), "0.0.0dev0");
    }

    #[test]
    fn bump_patch() {
        let version = PEP440Version::zero();
        assert_eq!(version.bump(), version.bump_patch());
    }

    #[test]
    fn bump_minor() {
        let version = PEP440Version::zero();
        assert_eq!(version.bump_minor(), PEP440Version::new(0, 1, 0));
    }

    #[test]
    fn bump_major() {
        let version = PEP440Version::zero();
        assert_eq!(version.bump_major(), PEP440Version::one());
    }

    #[test]
    fn bump_post() {
        let version = PEP440Version::zero().bump_post();
        assert_eq!(version.bump(), version.bump_post())
    }

    #[test]
    fn bump_dev() {
        let version = PEP440Version::zero().bump_dev();
        assert_eq!(
            version.bump(),
            PEP440Version {
                major: 0,
                minor: 0,
                patch: 0,
                epoch: 0,
                pre: None,
                post: None,
                dev: Some(1)
            }
        )
    }

    #[test]
    fn bump_post_dev() {
        let version = PEP440Version::zero().bump_post().bump_dev();
        assert_eq!(
            version.bump(),
            PEP440Version {
                major: 0,
                minor: 0,
                patch: 0,
                epoch: 0,
                pre: None,
                post: Some(0),
                dev: Some(1)
            }
        )
    }
}
