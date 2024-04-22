use inflector::Inflector;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InflectedString(String);

impl InflectedString {
    pub fn orig(&self) -> &str {
        &self.0
    }

    pub fn to_class_case(&self) -> String {
        self.orig().to_class_case()
    }
}

impl From<String> for InflectedString {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for InflectedString {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}
