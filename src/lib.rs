mod provider;
mod ranges;
mod version;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::version::PEP440Version;
use crate::provider::PypiProvider;

use pubgrub::range::Range;
use pubgrub::solver::resolve;
use std::collections::HashMap;

#[pyfunction]
fn resolve_pywrapper(root: &str, version: &str, requires: Vec<(&str, &str)>, dev_requires: Vec<(&str, &str)>) -> PyResult<Vec<(String, String)>> {
    // not an impl yet, just playing with stuff
    println!("rust side");
    println!("root: {}, version: {}", root, version);
    println!("requires: {:?}", requires);
    println!("dev_requires: {:?}", dev_requires);

    let provider = PypiProvider::default();
    let _r: Vec<HashMap<String, PEP440Version, _>> = requires.iter().map(|req| {
        let (name, version_str) = req;
        // range doesn't implement string parsing
        //let range: Range = version_str.parse();
        // this will fail since version_str encodes a range
        let version: PEP440Version = version_str.parse().expect("error");
        let solution: HashMap<String, PEP440Version, _> = resolve(&provider, name.to_string().into(), version).unwrap();
        println!("{:?}", solution);
        solution
    }).collect();

    // test result
    Ok(vec![("foo".into(), "1.2.3".into())])
}

//fn from_constraint(constraint: &str) -> Range

/// A Python module implemented in Rust.
#[pymodule]
fn _poetry_ext(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(resolve_pywrapper, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::version::PEP440Version;
    use pubgrub::range::Range;
    use pubgrub::solver::{resolve, OfflineDependencyProvider};

    #[test]
    fn simple_dep() {
        let mut provider: OfflineDependencyProvider<&str, PEP440Version> =
            OfflineDependencyProvider::new();
        provider.add_dependencies(
            "my_package",
            PEP440Version::one(),
            vec![("dep_1", Range::higher_than(PEP440Version::one()))],
        );
        provider.add_dependencies("dep_1", PEP440Version::zero().bump_minor(), vec![]);
        provider.add_dependencies("dep_1", PEP440Version::one(), vec![]);
        let solution = resolve(&provider, "my_package", PEP440Version::one()).unwrap();
        assert_eq!(solution.get("my_package"), Some(&PEP440Version::one()));
    }

    #[test]
    fn oslo_utils() {
        let mut provider: OfflineDependencyProvider<&str, PEP440Version> =
            OfflineDependencyProvider::new();
        provider.add_dependencies(
            "oslo.utils",
            PEP440Version::new(1, 4, 0),
            vec![
                ("Babel", Range::higher_than(PEP440Version::new(1, 3, 0))),
                ("iso8601", Range::higher_than(PEP440Version::new(0, 1, 9))),
                ("netaddr", Range::higher_than(PEP440Version::new(0, 7, 12))),
                (
                    "netifaces",
                    Range::higher_than(PEP440Version::new(0, 10, 4)),
                ),
                (
                    "pbr",
                    Range::between(PEP440Version::new(0, 6, 0), PEP440Version::new(0, 7, 0)).union(
                        &Range::between(PEP440Version::new(0, 7, 0), PEP440Version::one()),
                    ),
                ),
                ("six", Range::higher_than(PEP440Version::new(1, 9, 0))),
            ],
        );
        provider.add_dependencies(
            "Babel",
            PEP440Version::new(2, 9, 0),
            vec![("pytz", Range::higher_than(PEP440Version::new(2015, 7, 0)))],
        );
        provider.add_dependencies("pytz", PEP440Version::new(2020, 4, 0), vec![]);
        provider.add_dependencies("iso8601", PEP440Version::new(0, 1, 13), vec![]);
        provider.add_dependencies("netaddr", PEP440Version::new(0, 8, 0), vec![]);
        provider.add_dependencies("netifaces", PEP440Version::new(0, 10, 9), vec![]);
        provider.add_dependencies(
            "oslo.i18n",
            PEP440Version::new(2, 1, 0),
            vec![
                ("Babel", Range::higher_than(PEP440Version::new(1, 3, 0))),
                (
                    "pbr",
                    Range::between(PEP440Version::new(0, 11, 0), PEP440Version::new(2, 0, 0)),
                ),
                ("six", Range::higher_than(PEP440Version::new(1, 9, 0))),
            ],
        );
        provider.add_dependencies("pbr", PEP440Version::new(0, 11, 1), vec![]);
        provider.add_dependencies("six", PEP440Version::new(1, 15, 0), vec![]);

        let solution = resolve(&provider, "oslo.utils", PEP440Version::new(1, 4, 0)).unwrap();
        assert_eq!(
            solution.get("oslo.utils"),
            Some(&PEP440Version::new(1, 4, 0))
        )
    }
}
