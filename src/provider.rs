use crate::ranges::parse_dependency;
use crate::version::PEP440Version;
use pubgrub::range::Range;
use pubgrub::solver::Dependencies::Known;
use pubgrub::solver::{Dependencies, DependencyConstraints, DependencyProvider};
use serde::export::Formatter;
use serde::Deserialize;
use std::borrow::Borrow;
use std::error::Error;

struct PypiProvider {}

#[derive(Deserialize, Debug, Clone)]
struct PypiPackage {
    info: PackageInfo,
}

#[derive(Deserialize, Debug, Clone)]
struct PackageInfo {
    requires_dist: Option<Vec<String>>,
}

#[derive(Debug)]
enum TestError {
    Test,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "test_error")
    }
}

fn get_deps(
    package: &String,
    version: &PEP440Version,
) -> Result<DependencyConstraints<String, PEP440Version>, Box<dyn Error>> {
    let url = format!("https://pypi.org/pypi/{}/{}/json", package, version);
    let response = reqwest::blocking::get(&url)?;
    let package: PypiPackage = response.json()?;
    let deps = package
        .info
        .requires_dist
        .unwrap_or(Default::default())
        .iter()
        .filter_map(|v| parse_dependency(v.as_str()))
        .collect();
    Ok(deps)
}

impl Error for TestError {}

impl DependencyProvider<String, PEP440Version> for PypiProvider {
    fn choose_package_version<T: Borrow<String>, U: Borrow<Range<PEP440Version>>>(
        &self,
        potential_packages: impl Iterator<Item = (T, U)>,
    ) -> Result<(T, Option<PEP440Version>), Box<dyn Error>> {
        let mut i = potential_packages.map(|v| v);
        match i.next() {
            Some((p, range)) => Ok((p, range.borrow().lowest_version())),
            _ => Err(Box::new(TestError::Test)),
        }
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &PEP440Version,
    ) -> Result<Dependencies<String, PEP440Version>, Box<dyn Error>> {
        let deps = Known(get_deps(package, version)?);
        Ok(deps)
    }
}

#[cfg(test)]
mod test {
    use crate::provider::PypiProvider;
    use crate::version::PEP440Version;
    use pubgrub::solver::resolve;

    #[test]
    fn test_requests_1_0_0() {
        let provider = PypiProvider {};
        let solution = resolve(&provider, "requests".to_string(), PEP440Version::one()).unwrap();
        println!("{:?}", solution);
    }

    #[test]
    fn test_requests_2_25_0() {
        let provider = PypiProvider {};
        let solution = resolve(&provider, "requests".into(), PEP440Version::new(2, 25, 0)).unwrap();
        println!("{:?}", solution);
    }

    #[test]
    fn test_frozenlist_1_1_1() {
        let provider = PypiProvider {};
        let solution =
            resolve(&provider, "frozenlist".into(), PEP440Version::new(1, 1, 1)).unwrap();
        println!("{:?}", solution);
    }
}
