
/// Internal version struct for crate components
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u16,
}

impl Version {
    pub fn as_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl From<&str> for Version {
    fn from(val: &str) -> Self {
        let parts: Vec<&str> = val.split("\\-").collect();
        let vparts: Vec<&str> = parts[0].split("\\.").collect();
        Version {
            major: if vparts.len() > 0 {vparts[0].parse::<u8>().unwrap_or(0)} else {0},
            minor: if vparts.len() > 1 {vparts[1].parse::<u8>().unwrap_or(0)} else {0},
            patch: if vparts.len() > 2 {vparts[2].parse::<u16>().unwrap_or(0)} else {0},
        }
    }
}

impl Into<String> for Version {
    fn into(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// impl From<&String> for Version {
//     fn from(val: &String) -> Self {
//         let slice: &str = val;
//         Version::from(slice)
//     }
// }
