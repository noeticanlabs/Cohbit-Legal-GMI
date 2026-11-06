use crate::receipt::stable_hash;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataClassification {
    Public,
    Private,
    Privileged,
    Restricted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseScope {
    pub tenant_id: String,
    pub case_id: String,
    pub operator_id: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub max_document_bytes: usize,
    pub allowed_roles: Vec<String>,
    pub allow_external_model_sharing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UntrustedDocument {
    pub content_hash: String,
    pub byte_len: usize,
    pub classification: DataClassification,
    pub text: String,
}

impl SecurityPolicy {
    pub fn authorize_case(&self, actor: &CaseScope, tenant_id: &str, case_id: &str) -> Result<()> {
        if actor.tenant_id != tenant_id || actor.case_id != case_id {
            return Err(anyhow!("Cross-tenant or cross-case access denied"));
        }
        if !actor
            .roles
            .iter()
            .any(|role| self.allowed_roles.contains(role))
        {
            return Err(anyhow!("Operator has no authorized role"));
        }
        Ok(())
    }

    pub fn ingest_untrusted_text(
        &self,
        bytes: &[u8],
        classification: DataClassification,
    ) -> Result<UntrustedDocument> {
        if bytes.len() > self.max_document_bytes {
            return Err(anyhow!("Document exceeds configured size limit"));
        }
        let text =
            std::str::from_utf8(bytes).map_err(|_| anyhow!("Document is not valid UTF-8"))?;
        if text.contains('\0') {
            return Err(anyhow!("Document contains a NUL byte"));
        }
        Ok(UntrustedDocument {
            content_hash: stable_hash(&bytes),
            byte_len: bytes.len(),
            classification,
            text: text.into(),
        })
    }

    pub fn permit_external_model_share(&self, document: &UntrustedDocument) -> Result<()> {
        if !self.allow_external_model_sharing
            || document.classification != DataClassification::Public
        {
            return Err(anyhow!("External-model sharing is not authorized"));
        }
        Ok(())
    }
}

pub fn safe_relative_path(root: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(anyhow!("Unsafe relative path"));
    }
    Ok(root.join(relative))
}
