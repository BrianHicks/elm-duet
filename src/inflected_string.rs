use color_eyre::Result;
use eyre::bail;
use inflector::Inflector;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct InflectedString(String);

impl InflectedString {
    pub fn orig(&self) -> &str {
        &self.0
    }

    fn sanitized(&self) -> Result<&str> {
        if self.0.starts_with(char::is_numeric) {
            bail!("identifier `{}` cannot start with a digit", self.0)
        }

        Ok(&self.0)
    }

    pub fn to_pascal_case(&self) -> Result<String> {
        Ok(self.sanitized()?.to_pascal_case())
    }

    pub fn to_camel_case(&self) -> Result<String> {
        Ok(self.sanitized()?.to_camel_case())
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
