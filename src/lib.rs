mod poetry_provider;
mod provider;
mod ranges;
mod version;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;


use crate::poetry_provider::{PoetryProvider, RootPackage};
use crate::ranges::parse_dependency;

fn resolve(root: &str, version: &str, requires: Vec<(&str, &str)>, dev_requires: Vec<(&str, &str)>) -> Vec<(String, String)> {
    let version = version.parse().unwrap();
    let dependencies = requires
        .iter()
        .chain(dev_requires.iter())
        .map(|(name, range)| {
            let range = parse_dependency(&format!("{} ({})", name, range)).unwrap();
            range
        })
        .collect();
    let root = RootPackage {
        package: String::from(root),
        version,
        dependencies,
    };
    let provider = PoetryProvider::new(root.clone());
    let solution = pubgrub::solver::resolve(&provider, root.package, root.version).unwrap();
    solution.iter().map(|(p, v)| (p.clone(), format!("{}", v))).collect()
}

#[pyfunction]
fn resolve_pywrapper(
    root: &str,
    version: &str,
    requires: Vec<(&str, &str)>,
    dev_requires: Vec<(&str, &str)>,
) -> PyResult<Vec<(String, String)>> {
    // not an impl yet, just playing with stuff
    println!("rust side");
    println!("root: {}, version: {}", root, version);
    println!("requires: {:?}", requires);
    println!("dev_requires: {:?}", dev_requires);

    let solution = resolve(root, version, requires, dev_requires);
    println!("solution: {:?}", solution);
    Ok(solution)
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
    use crate::resolve;


    #[test]
    fn test_resolve_poetry() {
        let solution = resolve("poetry", "1.2.0a0", vec![
            ("poetry-core", ">=1.0.0,<2"),
            ("cleo", ">=0.8.1,<0.9"),
            ("clikit", ">=0.6.2,<0.7"),
            ("crashtest", ">=0.3.0,<0.4"),
            ("requests", ">=2.18,<3"),
            ("cachy", ">=0.3.0,<0.4"),
            ("requests-toolbelt", ">=0.9.1,<0.10"),
            ("pkginfo", ">=1.4,<2"),
            ("html5lib", ">=1.0,<2"),
            ("shellingham", ">=1.1,<2"),
            ("tomlkit", ">=0.7.0,<1.0.0"),
            ("pexpect", ">=4.7.0,<5"),
            ("packaging", ">=20.4,<21"),
            ("virtualenv", ">20.0.26,<21"),
            ("keyring", ">=21.2.0,<22"),
        ], vec![]);
        assert!(solution.contains(&("poetry".into(), "1.2.0a0".into())))
    }
}
