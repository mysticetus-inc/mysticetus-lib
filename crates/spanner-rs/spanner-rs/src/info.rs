use std::fmt;
use std::hash::{Hash, Hasher};

use gcp_auth_channel::{Auth, Scope};
use shared::Shared;

const PROJECTS_PREFIX: &str = "projects/";
const INSTANCE_PREFIX: &str = "/instances/";
const DATABASE_PREFIX: &str = "/databases/";

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Project {
    project_id: &'static str,
}

impl Project {
    #[inline]
    pub const fn new(project_id: &'static str) -> Self {
        Self { project_id }
    }

    #[inline]
    pub const fn project_id(&self) -> &'static str {
        self.project_id
    }

    #[inline]
    pub fn instance<I>(self, instance: I) -> Instance<I> {
        Instance {
            project_id: self.project_id,
            instance,
        }
    }

    pub fn fmt_qualified<F>(&self, writer: &mut F) -> fmt::Result
    where
        F: fmt::Write,
    {
        writer.write_str(PROJECTS_PREFIX)?;
        writer.write_str(self.project_id)
    }

    pub fn qualified_len(&self) -> usize {
        PROJECTS_PREFIX.len() + self.project_id.len()
    }

    pub fn build_qualified(&self) -> String {
        let mut dst = String::with_capacity(self.qualified_len());
        dst.push_str(PROJECTS_PREFIX);
        dst.push_str(self.project_id);
        dst
    }
}

impl fmt::Debug for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Project")
            .field(&Concat(&[PROJECTS_PREFIX, self.project_id.as_ref()]))
            .finish()
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_qualified(f)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Instance<I> {
    project_id: &'static str,
    instance: I,
}

impl<I> Instance<I> {
    #[inline]
    pub const fn as_project(&self) -> Project {
        Project {
            project_id: self.project_id,
        }
    }

    #[inline]
    pub const fn project_id(&self) -> &'static str {
        self.project_id
    }

    #[inline]
    pub const fn instance(&self) -> &I {
        &self.instance
    }
}

impl<I> Instance<I>
where
    I: AsRef<str>,
{
    pub fn fmt_qualified<F>(&self, writer: &mut F) -> fmt::Result
    where
        F: fmt::Write,
    {
        writer.write_str(PROJECTS_PREFIX)?;
        writer.write_str(self.project_id)?;
        writer.write_str(INSTANCE_PREFIX)?;
        writer.write_str(self.instance.as_ref())
    }

    /// Builder-style constructor for [`Database`] that just defers to [`Database::new`].
    #[inline]
    pub fn database<D: AsRef<str>>(&self, database: D) -> Database {
        Database::new(
            self.project_id.as_ref(),
            self.instance.as_ref(),
            database.as_ref(),
        )
    }

    pub fn qualified_len(&self) -> usize
    where
        I: AsRef<str>,
    {
        PROJECTS_PREFIX.len()
            + self.project_id.len()
            + INSTANCE_PREFIX.len()
            + self.instance.as_ref().len()
    }

    pub fn build_qualified(&self) -> String {
        let mut dst = String::with_capacity(self.qualified_len());

        dst.push_str(PROJECTS_PREFIX);
        dst.push_str(self.project_id);
        dst.push_str(INSTANCE_PREFIX);
        dst.push_str(self.instance.as_ref());
        dst
    }
}

impl<I: AsRef<str>> fmt::Debug for Instance<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Instance")
            .field(&Concat(&[
                PROJECTS_PREFIX,
                self.project_id,
                INSTANCE_PREFIX,
                self.instance.as_ref(),
            ]))
            .finish()
    }
}

impl<I: AsRef<str>> fmt::Display for Instance<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_qualified(f)
    }
}

// re-enable when const slice indexing comes back
/// Statically builds a [`Database`]
#[macro_export]
macro_rules! database {
    ($project_id:literal, $instance:literal, $database:literal) => {{
        $crate::info::Database::parse(
            $project_id,
            concat!(
                "projects/",
                $project_id,
                "/instances/",
                $instance,
                "/databases/",
                $database,
            ),
        )
    }};
}

#[derive(Clone, Copy)]
pub struct Database<S: AsRef<str> = Shared<str>> {
    /// The fully qualified database path, in the form:
    /// ```markdown
    /// `projects/<project>/instances/<instance>/databases/<database>`
    /// ```
    qualified: S,
    project_id: &'static str,
    /// Similar in concept to 'project_id_end', but since the instance name depends on the length
    /// of 'project_id', it's easier to just store the range of bytes in 'qualified' that contains
    /// the instance name.
    ///
    /// Instead of using [`std::ops::Range<usize>`], this is a tuple with the same meaning.
    /// (due to [`std::ops::Range<usize>`] not being [`Copy`])
    instance_range: (usize, usize),
    /// Again, similar to the above 2 fields, but since the database name is the final component
    /// of the string, we just need the start index.
    database_start: usize,
}

impl<A: AsRef<str>> Eq for Database<A> {}

impl<A: AsRef<str>> Hash for Database<A> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.qualified.as_ref().hash(state)
    }
}

impl<A, B> PartialEq<Database<A>> for Database<B>
where
    A: AsRef<str>,
    B: AsRef<str>,
{
    fn eq(&self, other: &Database<A>) -> bool {
        self.qualified.as_ref().eq(other.qualified.as_ref())
    }
}

impl<A, B> PartialOrd<Database<A>> for Database<B>
where
    A: AsRef<str>,
    B: AsRef<str>,
{
    fn partial_cmp(&self, other: &Database<A>) -> Option<std::cmp::Ordering> {
        Some(self.qualified.as_ref().cmp(other.qualified.as_ref()))
    }
}

impl<A> Ord for Database<A>
where
    A: AsRef<str>,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.qualified.as_ref().cmp(other.qualified.as_ref())
    }
}

macro_rules! assert_valid_parts {
    ($part:ident $(,)?) => {{
        // check for empty strings
        assert!(
            !$part.is_empty(),
            concat!("'", stringify!($part), "' must not be an empty string"),
        );
        // check for leading whitespace
        assert_eq!(
            $part.trim_start().len(), $part.len(),
            concat!("'", stringify!($part), "' must not have leading whitespace"),
        );
        // check for trailing whitespace
        assert_eq!(
            $part.trim_end().len(), $part.len(),
            concat!("'", stringify!($part), "' must not have trailing whitespace"),
        );
    }};
    ($($part:ident),* $(,)?) => {
        $(
            assert_valid_parts!($part);
        )*
    }


}

impl Database {
    /// Used by the [`database`] macro to construct a [`Database`] in a const context.
    #[doc(hidden)]
    #[inline(always)]
    pub const fn __from_parts(
        qualified: &'static str,
        project_id: &'static str,
        instance_range: (usize, usize),
        database_start: usize,
    ) -> Self {
        Self {
            project_id,
            instance_range,
            database_start,
            qualified: Shared::Static(qualified),
        }
    }

    #[inline]
    pub const fn builder(project_id: &'static str) -> Project {
        Project::new(project_id)
    }
}

impl<'a> Database<&'a str> {
    pub const fn parse_checked(
        project_id: &'static str,
        qualified: &'a str,
    ) -> Result<Self, &'static str> {
        // 6 comes from a minimum of 2 characters per user defined component (3x).
        // specified in google spanner proto comments.
        const MINIMUM_LEN: usize =
            PROJECTS_PREFIX.len() + INSTANCE_PREFIX.len() + DATABASE_PREFIX.len() + 6;

        let bytes = qualified.as_bytes();

        if bytes.len() < MINIMUM_LEN {
            return Err("invalid qualified database, not long enough");
        }

        // projects/ + Project Id

        if !util::eq(
            util::slice(bytes, 0, PROJECTS_PREFIX.len()),
            PROJECTS_PREFIX.as_bytes(),
        ) {
            return Err("leading component isn't 'projects/'");
        }

        let project_id_end =
            match util::find_next(util::slice(bytes, PROJECTS_PREFIX.len(), bytes.len()), b'/') {
                Some(offset) => PROJECTS_PREFIX.len() + offset,
                None => return Err("expected a path separator '/'"),
            };

        if let Err(error) = util::check_ident(
            util::slice(bytes, PROJECTS_PREFIX.len(), project_id_end),
            &[b'-'],
        ) {
            return Err(error);
        }

        // /instances/ + instance id

        let instance_id_start = project_id_end + INSTANCE_PREFIX.len();
        if !util::eq(
            util::slice(bytes, project_id_end, instance_id_start),
            INSTANCE_PREFIX.as_bytes(),
        ) {
            return Err("expected an intermediate '/instances/' component");
        }

        let instance_id_end =
            match util::find_next(util::slice(bytes, instance_id_start, bytes.len()), b'/') {
                Some(offset) => instance_id_start + offset,
                None => return Err("expected a path separator '/'"),
            };

        if let Err(error) = util::check_ident(
            util::slice(bytes, instance_id_start, instance_id_end),
            &[b'-'],
        ) {
            return Err(error);
        }

        // /databases/ + database id
        let database_start = instance_id_end + DATABASE_PREFIX.len();
        if !util::eq(
            util::slice(bytes, instance_id_end, database_start),
            DATABASE_PREFIX.as_bytes(),
        ) {
            return Err("expected an intermediate '/databases/' component");
        }

        if let Err(error) = util::check_ident(
            util::slice(bytes, database_start, bytes.len()),
            &[b'-', b'\\', b'_'],
        ) {
            return Err(error);
        }

        Ok(Self {
            qualified,
            project_id,
            instance_range: (instance_id_start, instance_id_end),
            database_start,
        })
    }

    pub const fn parse(project_id: &'static str, qualified: &'a str) -> Self {
        match Self::parse_checked(project_id, qualified) {
            Ok(parsed) => parsed,
            Err(message) => panic!("{}", message),
        }
    }
}

impl Database<&'static str> {
    pub fn new_leaked(project_id: &'static str, instance: &str, database: &str) -> Self {
        let Database {
            qualified,
            project_id,
            instance_range,
            database_start,
        } = Database::<String>::new(project_id, instance, database);

        Self {
            project_id,
            instance_range,
            database_start,
            qualified: Box::leak(qualified.into_boxed_str()),
        }
    }
}

impl<S: AsRef<str>> Database<S> {
    /// Creates a new [`Database`] info bundle.
    ///
    /// # Panics
    /// Panics if either of 'project_id', 'instance' or 'database'
    /// are empty strings or have leading/trailing whitespace
    #[inline]
    pub fn new(project_id: &'static str, instance: &str, database: &str) -> Self
    where
        S: From<String>,
    {
        assert_valid_parts!(project_id, instance, database);

        let instance_start = PROJECTS_PREFIX.len() + project_id.len() + INSTANCE_PREFIX.len();
        let instance_end = instance_start + instance.len();

        let database_start = instance_end + DATABASE_PREFIX.len();

        let capacity = database_start + database.len();

        let mut dst = String::with_capacity(capacity);
        dst.push_str(PROJECTS_PREFIX);
        dst.push_str(project_id);
        dst.push_str(INSTANCE_PREFIX);
        dst.push_str(instance);
        dst.push_str(DATABASE_PREFIX);
        dst.push_str(database);

        Self {
            qualified: S::from(dst),
            project_id,
            instance_range: (instance_start, instance_end),
            database_start,
        }
    }

    #[inline]
    pub fn as_project_id_builder(&self) -> Project {
        Project {
            project_id: self.project_id(),
        }
    }

    #[inline]
    pub fn as_instance_builder(&self) -> Instance<&str> {
        Instance {
            project_id: self.project_id(),
            instance: self.instance(),
        }
    }

    #[inline]
    pub fn convert_qualified<Dst>(self) -> Database<Dst>
    where
        Dst: From<S> + AsRef<str>,
    {
        Database {
            qualified: Dst::from(self.qualified),
            project_id: self.project_id,
            instance_range: self.instance_range,
            database_start: self.database_start,
        }
    }

    #[inline]
    pub const fn as_ref(&self) -> Database<&S> {
        Database {
            qualified: &self.qualified,
            project_id: self.project_id,
            instance_range: self.instance_range,
            database_start: self.database_start,
        }
    }

    #[inline]
    pub fn project_id(&self) -> &'static str {
        self.project_id
    }

    #[inline]
    pub fn instance(&self) -> &str {
        &self.qualified.as_ref()[self.instance_range.0..self.instance_range.1]
    }

    #[inline]
    pub fn database(&self) -> &str {
        &self.qualified.as_ref()[self.database_start..]
    }

    #[inline]
    pub fn qualified_project(&self) -> &str {
        &self.qualified.as_ref()[..PROJECTS_PREFIX.len() + self.project_id.len()]
    }

    #[inline]
    pub fn qualified_instance(&self) -> &str {
        &self.qualified.as_ref()[..self.instance_range.1]
    }

    #[inline]
    pub fn qualified_database(&self) -> &str {
        self.qualified.as_ref()
    }

    #[inline]
    pub async fn build_client(self, scope: Scope) -> crate::Result<crate::Client>
    where
        Shared<str>: From<S>,
    {
        crate::Client::new_inner(self.into_shared(), scope).await
    }

    #[inline]
    pub async fn build_client_from_auth(self, auth: Auth) -> crate::Result<crate::Client>
    where
        Shared<str>: From<S>,
    {
        crate::Client::new_loaded(self.into_shared(), auth).await
    }

    #[inline]
    pub async fn build_client_load_auth<F, E>(self, load_fut: F) -> crate::Result<crate::Client>
    where
        Shared<str>: From<S>,
        F: std::future::Future<Output = Result<Auth, E>>,
        E: Into<crate::Error>,
    {
        crate::Client::new_load_auth(self.into_shared(), load_fut).await
    }

    #[inline]
    pub fn into_shared(self) -> Database
    where
        Shared<str>: From<S>,
    {
        self.convert_qualified()
    }

    #[inline]
    pub fn to_shared(&self) -> Database {
        Database {
            qualified: Shared::Shared(std::sync::Arc::from(self.qualified.as_ref())),
            project_id: self.project_id,
            instance_range: self.instance_range,
            database_start: self.database_start,
        }
    }
}

impl<S: AsRef<str>> fmt::Debug for Database<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Database")
            .field(&Concat(&[self.qualified.as_ref()]))
            .finish()
    }
}

impl<S: AsRef<str>> fmt::Display for Database<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.qualified.as_ref())
    }
}

struct Concat<'a>(&'a [&'a str]);

impl fmt::Debug for Concat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;

        for item in self.0 {
            f.write_str(item)?;
        }

        f.write_str("\"")
    }
}

/// Constant helper functions for static [`Database`] parsing.
mod util {

    pub(super) const fn slice(bytes: &[u8], start_at: usize, end_before: usize) -> &[u8] {
        assert!(
            start_at < end_before,
            "start_at must be smaller than end_before"
        );
        if bytes.len() < end_before {
            panic!("whoops");
        }

        unsafe {
            let ptr = bytes.as_ptr().add(start_at);
            let len = end_before - start_at;
            std::slice::from_raw_parts(ptr, len)
        }
    }

    // Checks that 2 byte slices are equal, element-wise and length-wise
    pub(super) const fn eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let mut i = 0;
        while i < a.len() {
            if a[i] != b[i] {
                return false;
            }

            i += 1;
        }

        return true;
    }

    pub(super) const fn find_next(bytes: &[u8], target: u8) -> Option<usize> {
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == target {
                return Some(index);
            }

            index += 1;
        }

        None
    }

    /// checks that a byte slice is an allowed google identifier.
    /// can handle differing special characters that are allowed in the middle
    /// of an identifier via the 2nd argument.
    pub(super) const fn check_ident(
        ident: &[u8],
        allowed_middle_chars: &[u8],
    ) -> Result<(), &'static str> {
        if ident.len() < 2 {
            return Err("identifier too short (must be 2+ chracters)");
        }

        if !matches!(ident[0], b'a'..=b'z') {
            return Err("first character must start with a lowercase alphabetic letter");
        }

        let mut i = 1;
        while i < ident.len() {
            let byte = ident[i];

            if !matches!(byte, b'0'..=b'9' | b'a'..=b'z') {
                if i == ident.len() - 1 {
                    return Err("final character must be an ASCII alphanumeric character");
                }

                let mut allow_idx = 0;
                let mut found_allowed = false;
                while allow_idx < allowed_middle_chars.len() {
                    if byte == allowed_middle_chars[allow_idx] {
                        found_allowed = true;
                        break;
                    }
                    allow_idx += 1;
                }

                if !found_allowed {
                    return Err("found invalid character");
                }
            }

            i += 1;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Database, Project, DATABASE_PREFIX, INSTANCE_PREFIX, PROJECTS_PREFIX};

    const TEST_PROJECT_ID: &str = "test-project-id";
    const TEST_INSTANCE: &str = "test-instance";
    const TEST_DATABASE: &str = "test-database";

    const STATIC_DB: Database<&'static str> =
        crate::database!("test-project-id", "test-instance", "test-database");

    #[test]
    fn test_builders() {
        let mut buf = String::new();
        let mut expected = format!("{PROJECTS_PREFIX}{TEST_PROJECT_ID}");

        let project = Project::new(TEST_PROJECT_ID);

        // project formatting tests
        {
            assert_eq!(format!("{project:?}"), format!("Project({expected:?})"));

            project.fmt_qualified(&mut buf).unwrap();
            assert_eq!(buf, expected);
        }

        buf.clear();
        let instance = project.instance(TEST_INSTANCE);

        expected.push_str(INSTANCE_PREFIX);
        expected.push_str(TEST_INSTANCE);

        // instance formatting tests
        {
            assert_eq!(format!("{instance:?}"), format!("Instance({expected:?})"));

            instance.fmt_qualified(&mut buf).unwrap();
            assert_eq!(buf, expected);
        }

        buf.clear();
        let database = instance.database(TEST_DATABASE);

        expected.push_str(DATABASE_PREFIX);
        expected.push_str(TEST_DATABASE);

        // database formatting tests
        {
            assert_eq!(format!("{database:?}"), format!("Database({expected:?})"));
            assert_eq!(database.to_string(), expected);
        }

        // misc database function tests
        assert_eq!(database.project_id(), TEST_PROJECT_ID);
        assert_eq!(database.instance(), TEST_INSTANCE);
        assert_eq!(database.database(), TEST_DATABASE);

        assert_eq!(database.qualified_database(), &expected);

        assert_eq!(database.qualified_instance(), &instance.to_string());

        assert_eq!(database, STATIC_DB);
    }
}
