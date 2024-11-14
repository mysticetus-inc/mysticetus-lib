mod bindings;
mod client;
pub mod dataset;
pub mod job;
pub mod table;
pub use client::BigQueryClient;
pub mod util;

macro_rules! route {
    ($inner:expr; $($arg:expr)*) => {{
        let mut url = $inner.base_url().to_string();
        $(
            $crate::rest::Identifier::insert_self(&$arg, &mut url);
        )+

        url
    }};
}

pub(self) use route;

/// Trait describing an identifier for a table or dataset.
///
/// This aims to somewhat merge the traits [`AsRef<str>`] and [`std::fmt::Display`],
/// that way both types + strings can be used seamlessly.
pub trait Identifier {
    /// insert the component into the path/url that's being constructed.
    fn insert_self(&self, partial_path: &mut String);
}

pub struct DisplayIdentifier<T>(pub T);

impl<T> Identifier for DisplayIdentifier<T>
where
    T: std::fmt::Display,
{
    fn insert_self(&self, partial_path: &mut String) {
        if !partial_path.ends_with('/') {
            partial_path.push_str("/");
        }
        std::fmt::Write::write_fmt(partial_path, format_args!("{}", self.0)).expect(
            "<String as fmt::Write>::write_fmt should never panic, since it's all in memory",
        )
    }
}

impl<T: Identifier + ?Sized> Identifier for &T {
    fn insert_self(&self, partial_path: &mut String) {
        T::insert_self(self, partial_path);
    }
}

impl Identifier for str {
    fn insert_self(&self, partial_path: &mut String) {
        if !partial_path.ends_with('/') {
            partial_path.push_str("/");
        }

        partial_path.push_str(self)
    }
}

static_assertions::assert_impl_all!(&str: Identifier);
static_assertions::assert_impl_all!(&&str: Identifier);

#[tokio::test]
async fn test_table_get() -> crate::Result<()> {
    let client = BigQueryClient::new(
        "mysticetus-oncloud",
        gcp_auth_channel::Scope::BigQueryReadOnly,
    )
    .await?;

    let table = client
        .dataset_ref("oncloud_production")
        .table_ref("geotracks")
        .get()
        .await?;

    println!("{table:#?}");

    Ok(())
}

#[tokio::test]
async fn test_table_create() -> crate::Result<()> {
    let client =
        BigQueryClient::new("mysticetus-oncloud", gcp_auth_channel::Scope::BigQueryAdmin).await?;

    let table_ref = client
        .dataset("oncloud_local_mrudisel_arch")
        .table("test-table2");

    /*
    let table = table_ref
        .builder()
        .add_field("ts")
        .required()
        .timestamp()
        .finish_field()
        .add_field("count")
        .required()
        .integer()
        .finish_field()
        .add_field("repeated")
        .repeated()
        .string()
        .finish_field()
        .add_field("position")
        .geography()
        .finish_field()
        .add_field("float")
        .float()
        .required()
        .description("test_desc")
        .finish_field()
        .create()
        .await?;

    println!("{table:#?}");
    */

    use serde_json::json;

    let pos = util::WkbPoint::from_point(120.0, 90.0);
    let resp = table_ref.insert_rows([
        json!({"ts": "2022-01-01T00:00:00", "count": 1, "repeated": ["1", "2", "3"], "position": pos, "float": 3.1415 }),
        json!({"ts": "2022-01-02T00:00:00", "count": 2, "repeated": ["2"], "position": null, "float": "NaN" }),
    ]).await?;

    println!("{resp:#?}");
    Ok(())
}
