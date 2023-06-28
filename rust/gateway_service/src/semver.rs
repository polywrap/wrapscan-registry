use lazy_static::lazy_static;
use std::cmp::Ordering;
use regex::Regex;

type SemVer = (u32, u32, u32, Option<String>, Option<String>);

pub trait IVersion {
    fn name(&self) -> String;
}

lazy_static! {
    static ref SEMVER_REGEX: Regex = 
        Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$").unwrap();
}

pub fn compare_semver(a: &SemVer, b: &SemVer) -> Ordering {
    a.0.cmp(&b.0)
        .then_with(|| a.1.cmp(&b.1))
        .then_with(|| a.2.cmp(&b.2))
        .then_with(|| match (a.3.is_some(), b.3.is_some()) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (true, true) => a.3.as_ref().unwrap().cmp(b.3.as_ref().unwrap()),
            (false, false) => Ordering::Equal,
        })
}

pub fn get_latest<'a, T: IVersion>(partial: &str, versions: &'a [T]) -> Option<&'a T> {
    let partial_parts: Vec<u32> = partial.split('.').map(|s| s.parse().unwrap()).collect();
    versions.iter()
        .filter(|v| {
            if let Some(sem_ver) = parse_semver(&v.name()) {
                partial_parts.iter().enumerate().all(|(i, &part)| match i {
                    0 => sem_ver.0 == part,
                    1 => sem_ver.1 == part,
                    2 => sem_ver.2 == part,
                    _ => false,
                })
            } else {
                false
            }
        })
        .max_by_key(|v| parse_semver(&v.name()))
}

pub fn sort_versions<T: IVersion>(versions: &mut Vec<T>) {
    versions.sort_by(|a, b| {
        let a_semver = parse_semver(&a.name());
        let b_semver = parse_semver(&b.name());
        match (a_semver, b_semver) {
            (Some(a), Some(b)) => compare_semver(&a, &b),
            _ => Ordering::Equal,
        }
    });
}

pub fn parse_semver(version: &str) -> Option<SemVer> {
    let re = Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$").unwrap();
    let caps = re.captures(version)?;
    Some((
        caps[1].parse().unwrap(),
        caps[2].parse().unwrap(),
        caps[3].parse().unwrap(),
        caps.get(4).map(|m| m.as_str().to_string()),
        caps.get(5).map(|m| m.as_str().to_string()),
    ))
}

pub fn verify<T: IVersion>(version: &T) -> bool {
    parse_semver(&version.name()).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Version {
        name: String,
    }

    impl IVersion for Version {
        fn name(&self) -> String {
            self.name.clone()
        }
    }

    #[test]
    fn verify_should_return_true_for_valid_versions_and_false_for_invalid_versions() {
        assert!(verify(&Version { name: "1.2.3".into() }));
        assert!(verify(&Version { name: "1.2.3-alpha".into() }));
        assert!(!verify(&Version { name: "1.2".into() }));
        assert!(verify(&Version { name: "1.2.3-alpha.1".into() }));
        assert!(!verify(&Version { name: "a.b.c".into() }));
        assert!(!verify(&Version { name: "1.2.3-".into() }));
        assert!(!verify(&Version { name: "1.2.3+".into() }));
    }

    #[test]
    fn get_latest_should_return_the_latest_version_object_for_a_given_version_prefix() {
        let version_objects = vec![
            Version { name: "1.0.0".into() },
            Version { name: "1.1.0".into() },
            Version { name: "1.2.0".into() },
            Version { name: "2.0.0".into() },
            Version { name: "2.1.0".into() },
            Version { name: "2.1.1".into() },
        ];

        assert_eq!(get_latest("1", &version_objects).unwrap().name(), "1.2.0");
        assert_eq!(get_latest("2.1", &version_objects).unwrap().name(), "2.1.1");
        assert!(get_latest("3", &version_objects).is_none());
    }

    #[test]
    fn sort_versions_should_return_an_array_of_version_objects_sorted_in_ascending_order() {
        let mut unordered = vec![
            Version { name: "1.0.0".into() },
            Version { name: "2.0.0".into() },
            Version { name: "1.2.0".into() },
            Version { name: "2.1.0".into() },
            Version { name: "1.1.0".into() },
            Version { name: "2.1.1".into() },
        ];
        let expected_sorted = vec![
            Version { name: "1.0.0".into() },
            Version { name: "1.1.0".into() },
            Version { name: "1.2.0".into() },
            Version { name: "2.0.0".into() },
            Version { name: "2.1.0".into() },
            Version { name: "2.1.1".into() },
        ];

        sort_versions(&mut unordered);

        assert_eq!(
            unordered.iter().map(|v| v.name()).collect::<Vec<_>>(), 
            expected_sorted.iter().map(|v| v.name()).collect::<Vec<_>>()
        );
    }
}
