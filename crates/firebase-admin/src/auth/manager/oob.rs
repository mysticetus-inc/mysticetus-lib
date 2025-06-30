#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OobRequest<'a, Ty: OobRequestType = EmailSignIn> {
    email: &'a str,
    continue_url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    tenant_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_project_id: Option<&'a str>,
    #[serde(serialize_with = "OobRequestType::serialize")]
    request_type: Ty,
    return_oob_link: True,
}

impl<'a, Ty: OobRequestType> OobRequest<'a, Ty> {
    pub fn new(email: &'a str, continue_url: &'a str) -> Self {
        Self {
            email,
            continue_url,
            ..Default::default()
        }
    }

    pub fn tenant_id(mut self, tenant_id: &'a str) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn target_project_id(mut self, project_id: &'a str) -> Self {
        self.target_project_id = Some(project_id);
        self
    }

    /// Convinence method that just calls [`AuthManager::send_oob_code`]
    ///
    /// [`AuthManager::send_oob_code`]: [`crate::auth::AuthManager::send_oob_code`]
    pub async fn send(&self, manager: &crate::auth::AuthManager) -> crate::Result<OobResponse> {
        manager.send_oob_code(self).await
    }
}

pub trait OobRequestType: Default + Copy {
    const NAME: &str;

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(Self::NAME)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OobResponse {
    pub oob_code: Box<str>,
    pub oob_link: Box<str>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmailSignIn;

impl OobRequestType for EmailSignIn {
    const NAME: &str = "EMAIL_SIGNIN";
}

macro_rules! impl_const_marker_type {
    ($name:ident => $value:literal) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name;

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serde::Serialize::serialize(&$value, serializer)
            }
        }
    };
}

impl_const_marker_type!(True => true);
