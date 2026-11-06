use crate::security::DataClassification;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPolicy {
    pub retention_days: u32,
    pub encrypted_backups_required: bool,
    pub explicit_permission_required_for_private_export: bool,
}

impl PrivacyPolicy {
    pub fn authorize_source_export(
        &self,
        classification: &DataClassification,
        explicit_permission: bool,
    ) -> Result<()> {
        match classification {
            DataClassification::Public => Ok(()),
            DataClassification::Private
                if !self.explicit_permission_required_for_private_export || explicit_permission =>
            {
                Ok(())
            }
            DataClassification::Private => {
                Err(anyhow!("Private source export requires permission"))
            }
            DataClassification::Privileged | DataClassification::Restricted => Err(anyhow!(
                "Privileged or restricted source export is prohibited"
            )),
        }
    }
}
