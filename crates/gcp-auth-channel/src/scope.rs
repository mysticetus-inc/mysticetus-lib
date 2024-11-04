//! [`Scope`] enum for scopes used in `mysticetus-rs` workspace code

use std::cmp::Ordering;

use strum::{EnumIter, IntoEnumIterator};

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
        pub enum Scope { $($variant:ident = ($int:literal, $str_const:ident, $access:ident)),* $(,)? }) => {

        $(#[$attr])*
        pub enum Scope {
            $( $variant = $int ),*
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
                        Self::$variant => Self::$str_const,
                    )*
                }
            }

            #[inline]
            pub const fn from_int(int: u8) -> Option<Self> {
                match int {
                    $(
                        $int => Some(Self::$variant),
                    )*
                    _ => None,
                }
            }

            #[inline]
            pub const fn from_index(index: usize) -> Option<Self> {
                Self::from_int(index as u8)
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
    /// Enum with all auth scopes we need.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, EnumIter)]
    #[repr(u8)]
    pub enum Scope {
        CloudPlatformAdmin = (0, CLOUD_PLATFORM_ADMIN, Admin),
        CloudPlatformReadOnly = (1, CLOUD_PLATFORM_READ_ONLY, ReadOnly),

        BigQueryAdmin = (2, BIG_QUERY_ADMIN, Admin),
        BigQueryReadWrite = (3, BIG_QUERY_READ_WRITE, ReadWrite),
        BigQueryReadOnly = (4, BIG_QUERY_READ_ONLY, ReadOnly),

        Firestore = (5, FIRESTORE, Admin),

        GcsAdmin = (6, GCS_ADMIN, Admin),
        GcsReadWrite = (7, GCS_READ_WRITE, ReadWrite),
        GcsReadOnly = (8, GCS_READ_ONLY, ReadOnly),

        CloudTasks = (9, CLOUD_TASKS, Admin),

        PubSub = (10, PUB_SUB, Admin),

        SpannerAdmin = (11, SPANNER_ADMIN, Admin),
        SpannerData = (12, SPANNER_DATA, ReadWrite),

        FirestoreRealtimeDatabase = (13, FIREBASE_REALTIME_DATABASE, ReadWrite),
    }
}

impl Scope {
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

    pub const FIREBASE_REALTIME_DATABASE: &'static str =
        "https://www.googleapis.com/auth/firebase.database";

    #[inline(always)]
    pub const fn into_index(self) -> usize {
        self as u8 as usize
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
        use Scope::*;

        match (self, other) {
            (CloudPlatformAdmin, _) | (_, CloudPlatformAdmin) => CloudPlatformAdmin,
            (CloudPlatformReadOnly, other) | (other, CloudPlatformReadOnly)
                if other.is_read_only() =>
            {
                CloudPlatformReadOnly
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
    pub fn variants() -> ScopeIter {
        Self::iter()
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
            FirestoreRealtimeDatabase => "FirestoreRealtimeDatabase",
        }
    }

    #[inline]
    pub const fn scope_uri(&self) -> &'static str {
        use Scope::*;

        match self {
            CloudPlatformAdmin => Self::CLOUD_PLATFORM_ADMIN,
            CloudPlatformReadOnly => Self::CLOUD_PLATFORM_READ_ONLY,
            BigQueryAdmin => Self::BIG_QUERY_ADMIN,
            BigQueryReadWrite => Self::BIG_QUERY_READ_WRITE,
            BigQueryReadOnly => Self::BIG_QUERY_READ_ONLY,
            CloudTasks => Self::CLOUD_TASKS,
            Firestore => Self::FIRESTORE,
            GcsAdmin => Self::GCS_ADMIN,
            GcsReadWrite => Self::GCS_READ_WRITE,
            GcsReadOnly => Self::GCS_READ_ONLY,
            PubSub => Self::PUB_SUB,
            SpannerAdmin => Self::SPANNER_ADMIN,
            SpannerData => Self::SPANNER_DATA,
            FirestoreRealtimeDatabase => Self::FIREBASE_REALTIME_DATABASE,
        }
    }
}

// Ensures that Scope::ALL is sorted at compile time, without needing to run a test.
const _: () = {
    let mut index = 0;

    while index < Scope::ALL.len() - 1 {
        let first = Scope::ALL[index] as u8;
        let second = Scope::ALL[index + 1] as u8;

        let first_from_index = Scope::from_index(index).unwrap() as u8;
        let seocnd_from_index = Scope::from_index(index + 1).unwrap() as u8;

        if first_from_index != first || seocnd_from_index != second {
            panic!("Scope::ALL indexing doesn't return the same Scope");
        }

        if first > second {
            panic!("Scope::ALL is out of order");
        } else if first == second {
            panic!("Scope::ALL has duplicate elements?");
        } else if second - first != 1 {
            panic!("Scope::ALL has an element that jumps ahead incorrectly");
        }

        index += 1;
    }
};
