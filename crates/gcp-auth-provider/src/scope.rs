//! [`Scope`] enum for scopes used in `mysticetus-rs` workspace code

use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccessLevel {
    ReadOnly,
    ReadWrite,
    Admin,
}

impl AccessLevel {
    #[inline]
    pub const fn is_read_write(self) -> bool {
        matches!(self, Self::ReadWrite)
    }

    #[inline]
    pub const fn const_cmp(self, other: Self) -> Ordering {
        use AccessLevel::*;
        match (self, other) {
            // equal cases
            (Admin, Admin) | (ReadWrite, ReadWrite) | (ReadOnly, ReadOnly) => Ordering::Equal,
            // admin cases
            (Admin, _) => Ordering::Greater,
            (_, Admin) => Ordering::Less,
            // leftover uncaught cases
            (ReadOnly, ReadWrite) => Ordering::Less,
            (ReadWrite, ReadOnly) => Ordering::Greater,
        }
    }

    pub const fn min(self, other: Self) -> Self {
        match self.const_cmp(other) {
            Ordering::Less | Ordering::Equal => self,
            Ordering::Greater => other,
        }
    }

    pub const fn max(self, other: Self) -> Self {
        match self.const_cmp(other) {
            Ordering::Greater | Ordering::Equal => self,
            Ordering::Less => other,
        }
    }

    #[inline]
    pub const fn is_read_only(self) -> bool {
        matches!(self, Self::ReadOnly)
    }

    #[inline]
    pub const fn is_admin(self) -> bool {
        matches!(self, Self::Admin)
    }

    #[inline]
    pub const fn can_write(self) -> bool {
        matches!(self, Self::ReadWrite | Self::Admin)
    }
}

impl PartialOrd for AccessLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for AccessLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.const_cmp(*other)
    }
}

macro_rules! count_idents {
    ($x:ident $(, $rest:ident)+ $(,)?) => {
        1 + count_idents!($($rest,)*)
    };
    ($x:ident $(,)?) => {
        1
    };
}

macro_rules! define_scope_enum {
    (
        $(#[$attr:meta])*
        pub enum Scope { $($variant:ident = ($int:expr, $const_name:ident, $access:ident)),* $(,)? }
    ) => {


        $(#[$attr])*
        #[repr(u16)]
        pub enum Scope {
            $(
                $variant = $int,
            )*
        }


        bitflags::bitflags! {
            $(#[$attr])*
            pub struct Scopes: u16 {
                $(
                    const $const_name = $int;
                )*
            }
        }

        impl From<Scope> for Scopes {
            #[inline]
            fn from(scope: Scope) -> Scopes {
                Scopes::from_bits(scope as u16)
                    .expect("enum value should have a known bit pattern")
            }
        }

        impl Scope {
            pub const COUNT: usize = count_idents!($($variant,)*);

            pub const ALL: [Self; Self::COUNT] = [
                $(Self::$variant),*
            ];

            #[inline]
            pub const fn scope_url(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => urls::$const_name,
                    )*
                }
            }

            #[inline]
            pub const fn from_int(int: u16) -> Option<Self> {
                match int {
                    $(
                        $int => Some(Self::$variant),
                    )*
                    _ => None,
                }
            }

            #[inline]
            pub const fn from_index(index: usize) -> Option<Self> {
                Self::from_int(1 << index)
            }

            #[inline]
            pub const fn as_int(self) -> u8 {
                self as u8
            }

            pub const fn access_level(&self) -> AccessLevel {
                match self {
                    $(
                        Self::$variant => AccessLevel::$access,
                    )*
                }
            }
        }

    };
}

define_scope_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Scope {
        CloudPlatformAdmin    = (0x0001, CLOUD_PLATFORM_ADMIN, Admin),
        CloudPlatformReadOnly = (0x0002, CLOUD_PLATFORM_READ_ONLY, ReadOnly),
        BigQueryAdmin         = (0x0004, BIG_QUERY_ADMIN, Admin),
        BigQueryReadWrite     = (0x0008, BIG_QUERY_READ_WRITE, ReadWrite),
        BigQueryReadOnly      = (0x0010, BIG_QUERY_READ_ONLY, ReadOnly),
        Firestore             = (0x0020, FIRESTORE, Admin),
        GcsAdmin              = (0x0040, GCS_ADMIN, Admin),
        GcsReadWrite          = (0x0080, GCS_READ_WRITE, ReadWrite),
        GcsReadOnly           = (0x0100, GCS_READ_ONLY, ReadOnly),
        CloudTasks            = (0x0200, CLOUD_TASKS, Admin),
        PubSub                = (0x0400, PUB_SUB, Admin),
        SpannerAdmin          = (0x0800, SPANNER_ADMIN, Admin),
        SpannerData           = (0x1000, SPANNER_DATA, ReadWrite),
        RealtimeDatabase      = (0x2000, REALTIME_DATABASE, ReadWrite),
    }
}

pub(crate) mod urls {
    pub const CLOUD_PLATFORM_ADMIN: &'static str = "https://www.googleapis.com/auth/cloud-platform";

    pub const CLOUD_PLATFORM_READ_ONLY: &'static str =
        "https://www.googleapis.com/auth/cloud-platform.read-only";

    pub const BIG_QUERY_ADMIN: &'static str = "https://www.googleapis.com/auth/bigquery";
    pub const BIG_QUERY_READ_ONLY: &'static str =
        "https://www.googleapis.com/auth/bigquery.readonly";
    pub const BIG_QUERY_READ_WRITE: &'static str =
        "https://www.googleapis.com/auth/bigquery.insertdata";
    pub const CLOUD_TASKS: &'static str = "https://www.googleapis.com/auth/cloud-tasks";
    pub const FIRESTORE: &'static str = "https://www.googleapis.com/auth/datastore";
    pub const GCS_ADMIN: &'static str = "https://www.googleapis.com/auth/devstorage.full_control";
    pub const GCS_READ_ONLY: &'static str = "https://www.googleapis.com/auth/devstorage.read_only";
    pub const GCS_READ_WRITE: &'static str =
        "https://www.googleapis.com/auth/devstorage.read_write";
    pub const PUB_SUB: &'static str = "https://www.googleapis.com/auth/pubsub";
    pub const SPANNER_ADMIN: &'static str = "https://www.googleapis.com/auth/spanner.admin";
    pub const SPANNER_DATA: &'static str = "https://www.googleapis.com/auth/spanner.data";

    pub const REALTIME_DATABASE: &'static str = "https://www.googleapis.com/auth/firebase.database";
}

impl Scope {
    #[inline(always)]
    pub const fn into_index(self) -> usize {
        (self as u16).trailing_zeros() as usize
    }

    #[inline(always)]
    pub const fn is_cloud_admin(&self) -> bool {
        matches!(self, Self::CloudPlatformAdmin)
    }

    #[inline(always)]
    pub const fn is_cloud_read_only(&self) -> bool {
        matches!(self, Self::CloudPlatformReadOnly)
    }

    /// Picks the most "useful" scope. Rules for this usefulness are:
    ///
    /// - [`CloudPlatformAdmin`]: always then most "useful", since it can be used across any
    ///   service.
    /// - [`CloudPlatformReadOnly`]: Takes precedence over any other read only scope, for the same
    ///   reason as the admin variant.
    /// - If neither of the above are true, [`Scope::max_by_access_level`] is called, and the scope
    ///   with the highest [`AccessLevel`] is considered more useful.
    ///
    /// [`CloudPlatformAdmin`]: Scope::CloudPlatformAdmin
    /// [`CloudPlatformReadOnly`]: Scope::CloudPlatformReadOnly
    #[inline]
    pub const fn most_useful(self, other: Self) -> Self {
        match (self, other) {
            (Self::CloudPlatformAdmin, _) | (_, Self::CloudPlatformAdmin) => {
                Self::CloudPlatformAdmin
            }
            (Self::CloudPlatformReadOnly, other) | (other, Self::CloudPlatformReadOnly)
                if other.is_read_only() =>
            {
                Self::CloudPlatformReadOnly
            }
            _ => self.max_by_access_level(other),
        }
    }

    pub const fn max_by_access_level(self, other: Self) -> Self {
        match self.access_level().const_cmp(other.access_level()) {
            Ordering::Equal | Ordering::Greater => self,
            Ordering::Less => other,
        }
    }

    pub const fn min_by_access_level(self, other: Self) -> Self {
        match self.access_level().const_cmp(other.access_level()) {
            Ordering::Equal | Ordering::Less => self,
            Ordering::Greater => other,
        }
    }

    #[inline(always)]
    pub const fn is_admin(&self) -> bool {
        self.access_level().is_admin()
    }

    #[inline(always)]
    pub const fn is_read_only(&self) -> bool {
        self.access_level().is_read_only()
    }

    #[inline(always)]
    pub fn variants() -> std::array::IntoIter<Self, { Self::COUNT }> {
        Self::ALL.into_iter()
    }

    #[inline]
    pub const fn as_str(&self) -> &'static str {
        use Scope::*;

        match self {
            CloudPlatformAdmin => "CloudPlatformAdmin",
            CloudPlatformReadOnly => "CloudPlatformReadOnly",
            BigQueryAdmin => "BigQueryAdmin",
            BigQueryReadWrite => "BigQueryReadWrite",
            BigQueryReadOnly => "BigQueryReadOnly",
            CloudTasks => "CloudTasks",
            Firestore => "Firestore",
            GcsAdmin => "GcsAdmin",
            GcsReadWrite => "GcsReadWrite",
            GcsReadOnly => "GcsReadOnly",
            PubSub => "PubSub",
            SpannerAdmin => "SpannerAdmin",
            SpannerData => "SpannerData",
            RealtimeDatabase => "FirestoreRealtimeDatabase",
        }
    }

    #[inline]
    pub const fn scope_uri(&self) -> &'static str {
        use Scope::*;

        match self {
            CloudPlatformAdmin => urls::CLOUD_PLATFORM_ADMIN,
            CloudPlatformReadOnly => urls::CLOUD_PLATFORM_READ_ONLY,
            BigQueryAdmin => urls::BIG_QUERY_ADMIN,
            BigQueryReadWrite => urls::BIG_QUERY_READ_WRITE,
            BigQueryReadOnly => urls::BIG_QUERY_READ_ONLY,
            CloudTasks => urls::CLOUD_TASKS,
            Firestore => urls::FIRESTORE,
            GcsAdmin => urls::GCS_ADMIN,
            GcsReadWrite => urls::GCS_READ_WRITE,
            GcsReadOnly => urls::GCS_READ_ONLY,
            PubSub => urls::PUB_SUB,
            SpannerAdmin => urls::SPANNER_ADMIN,
            SpannerData => urls::SPANNER_DATA,
            RealtimeDatabase => urls::REALTIME_DATABASE,
        }
    }
}

// Ensures that Scope::ALL is sorted at compile time, without needing to run a test.
const _: () = {
    let mut index = 0;

    while index < Scope::ALL.len() - 1 {
        let first = Scope::ALL[index];
        let second = Scope::ALL[index + 1];

        if Scope::ALL[index].into_index() != index {
            panic!("invalid index");
        }

        if Scope::ALL[index + 1].into_index() != index + 1 {
            panic!("invalid index");
        }

        let first_from_index = Scope::from_index(first.into_index()).unwrap();
        let second_from_index = Scope::from_index(second.into_index()).unwrap();

        if first_from_index as u16 != first as u16 || second_from_index as u16 != second as u16 {
            panic!("Scope::ALL indexing doesn't return the same Scope");
        }

        if first.into_index() > second.into_index() {
            panic!("Scope::ALL is out of order");
        } else if first.into_index() == second.into_index() {
            panic!("Scope::ALL has duplicate elements?");
        } else if second.into_index() - first.into_index() != 1 {
            panic!("Scope::ALL has an element that jumps ahead incorrectly");
        }

        index += 1;
    }
};

impl Scopes {
    #[inline]
    pub fn iter_scopes(self) -> ScopeIter {
        ScopeIter {
            inner_iter: self.into_iter(),
        }
    }
}

pub struct ScopeIter {
    inner_iter: bitflags::iter::Iter<Scopes>,
}

impl Iterator for ScopeIter {
    type Item = Scope;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let scope = self.inner_iter.next()?;
            if let Some(scope) = Scope::from_int(scope.bits()) {
                return Some(scope);
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner_iter.size_hint()
    }
}

pub(crate) fn serialize_scope_urls<S>(scopes: &Scopes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    struct ConcatScopeUrls(Scopes);

    impl fmt::Display for ConcatScopeUrls {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for (i, scope) in self.0.iter_scopes().enumerate() {
                if i > 0 {
                    f.write_str(" ")?;
                }

                f.write_str(scope.scope_uri())?;
            }

            Ok(())
        }
    }

    serializer.collect_str(&ConcatScopeUrls(*scopes))
}

pub(crate) fn serialize_scope_urls_as_array<S>(
    scopes: &Scopes,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;

    let count = scopes.bits().count_ones() as usize;
    let mut seq = serializer.serialize_seq(Some(count))?;

    for scope in scopes.iter_scopes() {
        seq.serialize_element(scope.scope_uri())?;
    }

    seq.end()
}
