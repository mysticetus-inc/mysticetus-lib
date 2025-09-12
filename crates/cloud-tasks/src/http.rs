use std::collections::HashMap;

use protos::tasks::{HttpMethod, HttpRequest, OAuthToken, OidcToken, http_request};

pub struct HttpRequestBuilder<Auth = http_request::AuthorizationHeader, B = bytes::Bytes> {
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    auth: Auth,
    body: B,
}

macro_rules! impl_method_fn {
    ($($fn_name:ident($method:ident)),* $(,)?) => {
        $(
            pub fn $fn_name(mut self) -> Self {
                self.method = HttpMethod::$method;
                self
            }
        )*
    };
}

impl HttpRequestBuilder<(), ()> {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            auth: (),
            body: (),
        }
    }
}

impl<A, B> HttpRequestBuilder<A, B> {
    pub fn method(mut self, method: ::http::Method) -> Self {
        self.method = match method {
            ::http::Method::POST => HttpMethod::Post,
            ::http::Method::GET => HttpMethod::Get,
            ::http::Method::HEAD => HttpMethod::Head,
            ::http::Method::PUT => HttpMethod::Put,
            ::http::Method::DELETE => HttpMethod::Delete,
            ::http::Method::PATCH => HttpMethod::Patch,
            ::http::Method::OPTIONS => HttpMethod::Options,
            _ => panic!("unsupported http method for cloud tasks: {method:?}"),
        };
        self
    }

    impl_method_fn! {
        post(Post),
        patch(Patch),
        get(Get),
        head(Head),
        delete(Delete),
        options(Options),
    }

    pub fn header(mut self, key: ::http::header::HeaderName, value: impl Into<String>) -> Self {
        self.headers.insert(key.as_str().to_owned(), value.into());
        self
    }
}

impl HttpRequestBuilder<(), ()> {
    fn with_auth(
        self,
        auth: http_request::AuthorizationHeader,
    ) -> HttpRequestBuilder<http_request::AuthorizationHeader, ()> {
        HttpRequestBuilder {
            url: self.url,
            method: self.method,
            headers: self.headers,
            auth,
            body: self.body,
        }
    }

    pub fn oauth_token(
        self,
        service_account_email: impl Into<String>,
    ) -> HttpRequestBuilder<http_request::AuthorizationHeader, ()> {
        self.with_auth(http_request::AuthorizationHeader::OauthToken(OAuthToken {
            service_account_email: service_account_email.into(),
            scope: String::new(),
        }))
    }

    pub fn oauth_token_with_scope(
        self,
        service_account_email: impl Into<String>,
        scope: gcp_auth_channel::Scope,
    ) -> HttpRequestBuilder<http_request::AuthorizationHeader, ()> {
        self.with_auth(http_request::AuthorizationHeader::OauthToken(OAuthToken {
            service_account_email: service_account_email.into(),
            scope: scope.scope_uri().to_owned(),
        }))
    }

    pub fn oidc_token(
        self,
        service_account_email: impl Into<String>,
    ) -> HttpRequestBuilder<http_request::AuthorizationHeader, ()> {
        self.with_auth(http_request::AuthorizationHeader::OidcToken(OidcToken {
            service_account_email: service_account_email.into(),
            audience: String::new(),
        }))
    }

    pub fn oidc_token_with_audience(
        self,
        service_account_email: impl Into<String>,
        audience: impl Into<String>,
    ) -> HttpRequestBuilder<http_request::AuthorizationHeader, ()> {
        self.with_auth(http_request::AuthorizationHeader::OidcToken(OidcToken {
            service_account_email: service_account_email.into(),
            audience: audience.into(),
        }))
    }
}

impl HttpRequestBuilder<http_request::AuthorizationHeader, ()> {
    pub fn with_body(self, body: bytes::Bytes) -> HttpRequestBuilder {
        HttpRequestBuilder {
            url: self.url,
            method: self.method,
            headers: self.headers,
            auth: self.auth,
            body,
        }
    }

    pub fn serialize_json<S>(
        self,
        json: &S,
    ) -> ::core::result::Result<HttpRequestBuilder, serde_json::Error>
    where
        S: serde::Serialize + ?Sized,
    {
        serde_json::to_vec(json).map(|bytes| {
            self.header(::http::header::CONTENT_TYPE, "application/json")
                .with_body(bytes.into())
        })
    }
}

impl HttpRequestBuilder {
    pub(crate) fn into_proto(self) -> HttpRequest {
        HttpRequest {
            url: self.url,
            http_method: self.method as i32,
            headers: self.headers,
            body: self.body,
            authorization_header: Some(self.auth),
        }
    }
}
