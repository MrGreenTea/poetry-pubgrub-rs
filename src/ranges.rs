use crate::version::PEP440Version;
use lazy_static::lazy_static;
use pubgrub::range::Range;
use regex::Regex;
use std::str::FromStr;

lazy_static! {
    // copied from packaging python package
    pub static ref SPECIFIER_PATTERN: Regex = Regex::new(r"^(?P<compare>~=|==|!=|<=|>=|<|>|===)\s*(?P<version>\S+)\s*$").unwrap();
    pub static ref DEPENDENCY_PATTERN: Regex = Regex::new(r"^(?P<name>\S+)\s*(:?\((?P<specs>.+?)\))?\s*(?:;\s*(?P<extra>.*))?$").unwrap();
}

enum Compare {
    Compatible,
    Matching,
    Exclusion,
    LessOrEqual,
    GreaterOrEqual,
    StrictLess,
    StrictGreater,
    ArbitraryEqual,
}

impl FromStr for Compare {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s {
            "~=" => Self::Compatible,
            "==" => Self::Matching,
            "!=" => Self::Exclusion,
            "<=" => Self::LessOrEqual,
            ">=" => Self::GreaterOrEqual,
            "<" => Self::StrictLess,
            ">" => Self::StrictGreater,
            "===" => Self::ArbitraryEqual,
            _ => return Err(()),
        };
        Ok(v)
    }
}

fn compare_to_range(cmp: Compare, version: PEP440Version) -> Range<PEP440Version> {
    match cmp {
        Compare::GreaterOrEqual => Range::higher_than(version),
        Compare::LessOrEqual => {
            Range::strictly_lower_than(version.clone()).union(&Range::exact(version))
        }
        Compare::StrictLess => Range::strictly_lower_than(version),
        // TODO
        Compare::Matching => Range::exact(version),
        Compare::ArbitraryEqual => Range::exact(version),
        // TODO
        Compare::Compatible => Range::exact(version),
        Compare::Exclusion => Range::exact(version).negate(),
        Compare::StrictGreater => Range::strictly_lower_than(version.clone())
            .union(&Range::exact(version))
            .negate(),
    }
}

fn parse_specifier(spec: &str) -> Option<Range<PEP440Version>> {
    SPECIFIER_PATTERN.captures(spec).and_then(|captures| {
        let cmp = captures
            .name("compare")
            .and_then(|cmp| cmp.as_str().parse::<Compare>().ok());
        let version = captures
            .name("version")
            .and_then(|v| v.as_str().parse::<PEP440Version>().ok()).expect(&format!("{} could not be parsed", spec));
        match (cmp, version) {
            (Some(cmp), version) => Some(compare_to_range(cmp, version)),
            _ => None,
        }
    })
}

pub fn parse_dependency(versions: &str) -> Option<(String, Range<PEP440Version>)> {
    if let Some(captures) = DEPENDENCY_PATTERN.captures(versions) {
        // TODO handle extra
        if captures.name("extra").is_some() {
            return None;
        }
        match (captures.name("name"), captures.name("specs")) {
            (Some(name), Some(specs)) => {
                let range = specs
                    .as_str()
                    .split(",")
                    .filter_map(|p| {
                        let s = parse_specifier(p);
                        debug_assert!(s.is_some());
                        s
                    })
                    .fold(Range::any(), |acc, r| acc.intersection(&r));
                return Some((name.as_str().into(), range));
            }
            (Some(name), None) => return Some((name.as_str().into(), Range::any())),
            _ => (),
        }
    }
    None
}

#[cfg(test)]
mod test {
    use crate::ranges::{compare_to_range, parse_dependency, parse_specifier, Compare};
    use crate::version::PEP440Version;
    use pubgrub::range::Range;

    #[test]
    fn test_compare_to_range() {
        let range = compare_to_range(Compare::GreaterOrEqual, PEP440Version::zero());
        assert_eq!(range, Range::any());
    }

    #[test]
    fn test_parse_specifier_lt() {
        let range = parse_specifier("<4.0.0").unwrap();
        assert_eq!(
            range,
            Range::strictly_lower_than(PEP440Version::new(4, 0, 0))
        )
    }

    #[test]
    fn test_parse_specificer_gte() {
        let range = parse_specifier(">=3.0.2").unwrap();
        assert_eq!(range, Range::higher_than(PEP440Version::new(3, 0, 2)));
    }

    #[test]
    fn test_range_set_operation() {
        let range: Range<PEP440Version> = Range::higher_than(PEP440Version::new(3, 0, 2))
            .intersection(&Range::strictly_lower_than(PEP440Version::new(4, 0, 0)));
        assert_eq!(
            range,
            Range::between(PEP440Version::new(3, 0, 2), PEP440Version::new(4, 0, 0))
        );
    }

    #[test]
    fn test_parsing_chardet() {
        let require = "chardet (<4.0.0,>=3.0.2)";
        let range = parse_dependency(require).unwrap();
        assert_eq!(
            range,
            (
                "chardet".into(),
                Range::between(PEP440Version::new(3, 0, 2), PEP440Version::new(4, 0, 0))
            )
        );
    }

    #[test]
    fn test_parsing_idna() {
        let require = "idna (<3.0.0,>=2.5.0)";
        let range = parse_dependency(require).unwrap();
        assert_eq!(
            range,
            (
                "idna".into(),
                Range::between(PEP440Version::new(2, 5, 0), PEP440Version::new(3, 0, 0))
            )
        )
    }

    #[test]
    fn test_parsing_pyopenssl() {
        let require = "pyOpenSSL (>=0.14.0) ; extra == 'security'";
        let range = parse_dependency(require);
        assert_eq!(range, None)
    }

    #[test]
    fn test_parsing_without_constrains() {
        let require = "pytz";
        let range = parse_dependency(require).unwrap();
        assert_eq!(range, ("pytz".into(), Range::any()));
    }
}
