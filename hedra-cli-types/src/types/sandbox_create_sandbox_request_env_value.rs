pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum CreateSandboxRequestEnvValue {
        String(String),

        SandboxEnvRef(SandboxEnvRef),
}

impl CreateSandboxRequestEnvValue {
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_sandbox_env_ref(&self) -> bool {
        matches!(self, Self::SandboxEnvRef(_))
    }


    pub fn as_string(&self) -> Option<&str> {
        match self {
                    Self::String(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_string(self) -> Option<String> {
        match self {
                    Self::String(value) => Some(value),
                    _ => None,
                }
    }

    pub fn as_sandbox_env_ref(&self) -> Option<&SandboxEnvRef> {
        match self {
                    Self::SandboxEnvRef(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_sandbox_env_ref(self) -> Option<SandboxEnvRef> {
        match self {
                    Self::SandboxEnvRef(value) => Some(value),
                    _ => None,
                }
    }
}

impl fmt::Display for CreateSandboxRequestEnvValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => write!(f, "{}", value),
            Self::SandboxEnvRef(value) => write!(f, "{}", serde_json::to_string(value).unwrap_or_else(|_| format!("{:?}", value))),
        }
    }
}
