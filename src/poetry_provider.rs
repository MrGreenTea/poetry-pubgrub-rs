use crate::provider::PypiProvider;
use crate::version::PEP440Version;
use pubgrub::package::Package;
use pubgrub::range::Range;
use pubgrub::solver::{Dependencies, DependencyConstraints, DependencyProvider};
use pubgrub::version::Version;
use std::borrow::Borrow;
use std::error::Error;

#[derive(Clone)]
pub struct RootPackage<P: Package, V: Version> {
    pub package: P,
    pub version: V,
    pub dependencies: DependencyConstraints<P, V>,
}

pub struct PoetryProvider {
    remote: PypiProvider,
    root: RootPackage<String, PEP440Version>,
}

impl PoetryProvider {
    pub fn new(root: RootPackage<String, PEP440Version>) -> Self {
        PoetryProvider {
            remote: PypiProvider::default(),
            root,
        }
    }
}

impl DependencyProvider<String, PEP440Version> for PoetryProvider {
    fn choose_package_version<T: Borrow<String>, U: Borrow<Range<PEP440Version>>>(
        &self,
        potential_packages: impl Iterator<Item = (T, U)>,
    ) -> Result<(T, Option<PEP440Version>), Box<dyn Error>> {
        let (root, other): (Vec<_>, Vec<_>) = potential_packages.partition(|(p, _)| p.borrow() == &self.root.package);
        match root.into_iter().next() {
            Some((p, _)) => Ok((p, Some(self.root.version.clone()))),
            None => self.remote.choose_package_version(other.into_iter())
        }
    }

    fn get_dependencies(
        &self,
        package: &String,
        version: &PEP440Version,
    ) -> Result<Dependencies<String, PEP440Version>, Box<dyn Error>> {
        if package == &self.root.package {
            return Ok(Dependencies::Known(self.root.dependencies.clone()))
        }
        self.remote.get_dependencies(package, version)
    }
}
