use crate::ranges::parse_dependency;
use crate::version::PEP440Version;
use pubgrub::range::Range;
use pubgrub::solver::Dependencies::Known;
use pubgrub::solver::{
    choose_package_with_fewest_versions, Dependencies, DependencyConstraints, DependencyProvider,
};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;

pub struct PypiProvider {
    client: reqwest::blocking::Client,
    releases_cache: RefCell<HashMap<String, Vec<PEP440Version>>>,
}

impl Default for PypiProvider {
    fn default() -> Self {
        PypiProvider {
            client: reqwest::blocking::Client::new(),
            releases_cache: RefCell::new(Default::default()),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct PypiPackage {
    info: PackageInfo,
    releases: Map<String, Value>,
}

#[derive(Deserialize, Debug, Clone)]
struct PackageInfo {
    requires_dist: Option<Vec<String>>,
}

fn get_deps(
    client: &reqwest::blocking::Client,
    package: &String,
    version: &PEP440Version,
) -> Result<DependencyConstraints<String, PEP440Version>, Box<dyn Error>> {
    let url = format!("https://pypi.org/pypi/{}/{}/json", package, version);
    let response = client.get(&url).send()?;
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

impl DependencyProvider<String, PEP440Version> for PypiProvider {
    fn choose_package_version<T: Borrow<String>, U: Borrow<Range<PEP440Version>>>(
        &self,
        potential_packages: impl Iterator<Item = (T, U)>,
    ) -> Result<(T, Option<PEP440Version>), Box<dyn Error>> {
        let list_available_versions = |package: &String| {
            let versions: Vec<PEP440Version> = self
                .releases_cache
                .borrow_mut()
                .entry(package.clone())
                .or_insert_with(|| {
                    let url = format!("https://pypi.org/pypi/{}/json", package);
                    let mut versions: Vec<PEP440Version> = self
                        .client
                        .get(&url)
                        .send()
                        .and_then(|response| {
                            let data: PypiPackage = response.json()?;
                            Ok(data
                                .releases
                                .keys()
                                .filter_map(|v| v.parse::<PEP440Version>().ok())
                                .collect())
                        })
                        .unwrap_or(Default::default());
                    versions.sort();
                    versions
                })
                .clone();
            versions.into_iter().rev()
        };

        Ok(choose_package_with_fewest_versions(
            list_available_versions,
            potential_packages,
        ))
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &PEP440Version,
    ) -> Result<Dependencies<String, PEP440Version>, Box<dyn Error>> {
        let deps = Known(get_deps(&self.client, package, version)?);
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
        let provider = PypiProvider::default();
        let solution = resolve(&provider, "requests".to_string(), PEP440Version::one()).unwrap();
        println!("{:?}", solution);
    }

    #[test]
    fn test_requests_2_25_0() {
        let provider = PypiProvider::default();
        let solution = resolve(&provider, "requests".into(), PEP440Version::new(2, 25, 0)).unwrap();
        for (p, v) in vec![
            ("requests".into(), PEP440Version::new(2, 25, 0)),
            ("certifi".into(), PEP440Version::new(2020, 12, 5)),
            ("chardet".into(), PEP440Version::new(3, 0, 4)),
            ("idna".into(), PEP440Version::new(2, 10, 0)),
            ("urllib3".into(), PEP440Version::new(1, 26, 2)),
        ] {
            assert_eq!((p, solution.get(p).unwrap()), (p, &v))
        }
    }

    #[test]
    fn test_frozenlist_1_1_1() {
        let provider = PypiProvider::default();
        let solution =
            resolve(&provider, "frozenlist".into(), PEP440Version::new(1, 1, 1)).unwrap();
        println!("{:?}", solution);
    }

    #[test]
    fn test_django_3_1_3() {
        let provider = PypiProvider::default();
        let solution = resolve(&provider, "django".into(), PEP440Version::new(3, 1, 3)).unwrap();
        for (p, v) in vec![
            ("asgiref", PEP440Version::new(3, 3, 1)),
            ("pytz", PEP440Version::new(2020, 4, 0)),
            ("sqlparse", PEP440Version::new(0, 4, 1)),
            ("django", PEP440Version::new(3, 1, 3)),
        ] {
            assert_eq!((p, solution.get(p).unwrap()), (p, &v));
        }
    }

    #[test]
    fn test_tensorflow_2_3_1() {
        let provider = PypiProvider::default();
        let solution =
            resolve(&provider, "tensorflow".into(), PEP440Version::new(2, 3, 1)).unwrap();
    }
}
