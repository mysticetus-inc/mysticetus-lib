//! A simple, abstract file tree data structure
use std::collections::BTreeMap;
use std::collections::btree_map::Values;
use std::path::{Components, Path, PathBuf};

/// An abstract file tree.
///
/// Purposefully omits file links, and boils things down to [`Node`]'s, that can only be a nested
/// tree or a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileTree {
    inner: BTreeMap<String, Node>,
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    /// A file, where the path is the full path from the root of the top level [`FileTree`].
    File(PathBuf),
    /// A nested file tree.
    Tree(FileTree),
}

impl FileTree {
    /// Creates a new, empty [`FileTree`].
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    /// Takes an iterator of strings and inserts them into a newly created [`FileTree`]. Calls the
    /// safer [`insert_checked`] under the hood, which is the only potential source of [`Err`]'s.
    ///
    /// [`insert_checked`]: [`Self::insert_checked`]
    pub fn from_paths_checked<I, P>(iter: I) -> Result<Self, &'static str>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<str>,
    {
        let mut tree = Self::new();

        for path in iter.into_iter() {
            tree.insert_checked(path.as_ref())?;
        }

        Ok(tree)
    }

    /// Takes an iterator of strings and inserts them into a newly created [`FileTree`]. Calls the
    /// panic-possible [`insert`] method under the hood, which will panic if any of the paths are
    /// empty or contain invalid UTF-8. Used by the [`Extend`] impl under the hood.
    ///
    /// [`insert`]: [`Self::insert`]
    pub fn from_paths<I, P>(iter: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<str>,
    {
        let mut tree = Self::new();

        for path in iter.into_iter() {
            tree.insert(path.as_ref());
        }

        tree
    }

    /// Inserts a file (as a path) into the tree. Only returns [`Err`] if the path is empty, or
    /// contains invalid UTF-8.
    pub fn insert_checked<P>(&mut self, path: P) -> Result<(), &'static str>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();
        let mut components = path.components();

        // Pops the filename, leaving the base path in 'components', that is then used in
        // [`walk_and_insert`]
        let filename = components
            .next_back()
            .ok_or("no final component in the path")?
            .as_os_str()
            .to_str()
            .ok_or("invalid UTF-8 in path")?
            .to_owned();

        self.walk_and_insert(components)
            .unwrap()
            .inner
            .insert(filename, Node::File(path));

        Ok(())
    }

    /// Inserts a file (as a path) into the tree. Panics if the path is invalid (is either empty
    /// or containing invalid UTF-8). Returns a mutable reference to facilitate a builder patter
    /// of sorts.
    pub fn insert<P>(&mut self, path: P) -> &mut Self
    where
        P: AsRef<Path>,
    {
        self.insert_checked(path).expect("invalid path encountered");
        self
    }

    /// Internal helper for inserting nodes. Inserts intermediate [`FileTree`]'s if they dont
    /// already exist. If there was already a file at one of the intermediate paths, returns
    /// [`None`]. Otherwise, the [`Some`] variant is a mutable reference to the tree at the
    /// end of the path that generated the [`Components`].
    fn walk_and_insert(&mut self, mut components: Components<'_>) -> Option<&mut FileTree> {
        let comp = match components.next() {
            Some(comp) => comp.as_os_str().to_str().unwrap(),
            // return if we're at the end
            None => return Some(self),
        };

        match self
            .inner
            .entry(comp.to_owned())
            .or_insert_with(|| Node::Tree(FileTree::new()))
        {
            Node::Tree(inner_tree) => inner_tree.walk_and_insert(components),
            _ => None,
        }
    }

    /// Internal helper for walking the inner tree.
    fn walk_tree(&self, mut components: Components<'_>) -> Option<&FileTree> {
        let comp = match components.next() {
            Some(comp) => comp.as_os_str().to_str().unwrap(),
            // return if we're at the end
            None => return Some(self),
        };

        match self.inner.get(comp) {
            // if the node is a tree, recursively walk the remaining components
            Some(Node::Tree(tree)) => tree.walk_tree(components),
            // otherwise, bail early.
            _ => None,
        }
    }

    /// Gets the node at a given path, if it exists.
    pub fn get_node<P>(&self, path: P) -> Option<&Node>
    where
        P: AsRef<Path>,
    {
        let mut components = path.as_ref().components();

        // pops the filename, that way 'components' ends at the containing directory.
        let filename = components.next_back()?;

        self.walk_tree(components)?
            .inner
            .get(filename.as_os_str().to_str().unwrap())
    }

    pub fn get_tree<P>(&self, tree_path: P) -> Option<&FileTree>
    where
        P: AsRef<Path>,
    {
        self.walk_tree(tree_path.as_ref().components())
    }

    pub fn iter_files<P>(&self, tree_path: P) -> Option<FileIter<'_>>
    where
        P: AsRef<Path>,
    {
        let tree = self.get_tree(tree_path)?;

        Some(FileIter {
            inner_iter: tree.inner.values(),
        })
    }

    /// Checks if a path exists in the tree. Does not differentiate between the 2 [`Node`]
    /// variants. If that matters, use [`contains_file`] or [`contains_tree`], which further
    /// checks that the [`Node`] is a [`Node::File`] or [`Node::Tree`] respectively.
    ///
    /// [`contains_file`]: [`Self::contains_file`]
    /// [`contains_tree`]: [`Self::contains_tree`]
    pub fn contains<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.get_node(path).is_some()
    }

    /// Checks if there is a file at a certain path. This makes sure the [`Node`] variant at that
    /// path is indeeed a [`Node::File`].
    pub fn contains_file<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        matches!(self.get_node(path), Some(Node::File(_)))
    }

    /// Checks if there is a nested tree at a certain path. This makes sure the [`Node`] variant
    /// at that path is indeeed a [`Node::Tree`].
    pub fn contains_tree<P>(&self, path: P) -> bool
    where
        P: AsRef<Path>,
    {
        matches!(self.get_node(path), Some(Node::Tree(_)))
    }
}

impl<S> FromIterator<S> for FileTree
where
    S: AsRef<str>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = S>,
    {
        FileTree::from_paths(iter)
    }
}

impl<S> Extend<S> for FileTree
where
    S: AsRef<str>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = S>,
    {
        for item in iter.into_iter() {
            self.insert(item.as_ref());
        }
    }
}

pub struct FileIter<'a> {
    inner_iter: Values<'a, String, Node>,
}

impl<'a> Iterator for FileIter<'a> {
    type Item = &'a Path;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner_iter.next() {
                Some(Node::File(path)) => return Some(path.as_ref()),
                Some(_) => (),
                None => return None,
            }
        }
    }
}
