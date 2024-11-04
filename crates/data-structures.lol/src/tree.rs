use std::collections::LinkedList;
use std::fmt;
use std::marker::PhantomData;
use std::ptr::{self, NonNull};

pub struct Tree<T> {
    root: Option<NonNull<Node<T>>>,
    len: usize,
    _marker: PhantomData<T>,
}

// since [`Tree`] is very similar to [`LinkedList`], this
// uses the same bounds for [`Send`]/[`Sync`]
unsafe impl<T: Send> Send for Tree<T> {}
unsafe impl<T: Sync> Sync for Tree<T> {}

impl<T: serde::Serialize> serde::Serialize for Tree<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(root) = self.root.as_ref() {
            SerializableNode(unsafe { root.as_ref() }).serialize(serializer)
        } else {
            serializer.serialize_unit()
        }
    }
}

impl<T: Clone + PartialEq> Clone for Tree<T> {
    fn clone(&self) -> Self {
        let pairs = std::cell::RefCell::new(Vec::with_capacity(self.len));

        self.visit_preorder_pairs(|a, b| pairs.borrow_mut().push((a.clone(), b.clone())));

        match Tree::from_preorder_pairs(pairs.into_inner()) {
            Ok(tree) => tree,
            Err(_) => panic!("a valid tree will always produce a valid set of preorder pairs"),
        }
    }
}

impl<T: Eq> Eq for Tree<T> {}

impl<T: PartialEq> PartialEq for Tree<T> {
    fn eq(&self, other: &Self) -> bool {
        // quick check to see if they have the same number of nodes
        if self.len != other.len {
            return false;
        }

        fn recurse_node_partial_eq<T: PartialEq>(
            a: &NonNull<Node<T>>,
            b: &NonNull<Node<T>>,
        ) -> bool {
            unsafe {
                // check values
                if a.as_ref().value != b.as_ref().value {
                    return false;
                }

                // check children count next
                if a.as_ref().children.len() != b.as_ref().children.len() {
                    return false;
                }

                for (child_a, child_b) in a.as_ref().children.iter().zip(b.as_ref().children.iter())
                {
                    if !recurse_node_partial_eq(child_a, child_b) {
                        return false;
                    }
                }

                true
            }
        }

        match (self.root, other.root) {
            (Some(a), Some(b)) => recurse_node_partial_eq(&a, &b),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tree")
            .field("node_count", &self.len)
            .field("root", &self.root.map(|n| unsafe { n.as_ref() }))
            .finish()
    }
}

#[cfg(feature = "ptree")]
impl<T: fmt::Display + Clone> fmt::Display for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Adapter<'a, 'b>(&'a mut fmt::Formatter<'b>);

        impl std::io::Write for Adapter<'_, '_> {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                match std::str::from_utf8(buf) {
                    Ok(s) => self
                        .0
                        .write_str(s)
                        .map(|_| buf.len())
                        .map_err(|err| std::io::Error::other(err)),
                    Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
                }
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        self.write_to(&mut Adapter(f)).map_err(|_| fmt::Error)
    }
}

fn count_children<T>(node: &Node<T>) -> usize {
    fn inner<A>(node: &Node<A>, count: &mut usize) {
        for child in node.children.iter() {
            *count += 1;
            inner(unsafe { child.as_ref() }, count);
        }
    }

    let mut count = 0;
    inner(node, &mut count);
    count
}

/// From a given node, iterate upwards until we find the root node.
fn get_root<T>(mut node: NonNull<Node<T>>) -> NonNull<Node<T>> {
    while let Some(parent) = unsafe { node.as_ref().parent } {
        node = parent;
    }

    node
}

/// Takes a node, and detaches the parent. If there is no parent (aka we're the root), this handles
/// detaching from the parent [`Tree`] properly as well. Returns a [`NonNull`] ptr to the parent
/// node (or [`None`] if we're the root).
fn detatch_parent<T>(node: &mut Node<T>, tree: &mut Tree<T>) -> Option<NonNull<Node<T>>> {
    let mut parent = match node.parent.take() {
        Some(parent) => parent,
        None => {
            // if we have no parent, we're the root node.
            debug_assert_eq!(0, node.level);
            tree.root = None;
            tree.len = 0;
            return None;
        }
    };

    tree.len -= count_children(&*node) + 1;

    let mut cursor = unsafe { parent.as_mut().children.cursor_front_mut() };

    loop {
        match cursor.current().copied() {
            Some(curr) if ptr::eq(ptr::addr_of!(*node), curr.as_ptr()) => {
                cursor.remove_current();
                return Some(parent);
            }
            Some(_) => cursor.move_next(),
            None => panic!("parent wasn't attached to child"),
        }
    }
}

/// Iterates upwards from a given node and finds the first parent with a matching value. Returns
/// [`None`] if we hit the root node without finding a match.
fn find_parent<U, T: PartialEq<U>>(
    mut node: NonNull<Node<T>>,
    value: &U,
) -> Option<NonNull<Node<T>>> {
    unsafe {
        loop {
            match node.as_ref().parent {
                Some(parent) => node = parent,
                None => return None,
            }

            if node.as_ref().value == *value {
                return Some(node);
            }
        }
    }
}

struct SerializableNode<'a, T>(&'a Node<T>);

impl<T: serde::Serialize> serde::Serialize for SerializableNode<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let len = if self.0.children.is_empty() { 1 } else { 2 };

        let mut map = serializer.serialize_map(Some(len))?;

        map.serialize_entry("value", &self.0.value)?;

        if !self.0.children.is_empty() {
            map.serialize_entry("children", &SerializeChildren(&self.0.children))?;
        }

        map.end()
    }
}

struct SerializeChildren<'a, T>(&'a LinkedList<NonNull<Node<T>>>);

impl<T: serde::Serialize> serde::Serialize for SerializeChildren<'_, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;

        for elem in self.0 {
            seq.serialize_element(&SerializableNode(unsafe { elem.as_ref() }))?;
        }

        seq.end()
    }
}

impl<T> Tree<T> {
    /// Removes the first node that would be hit in a post order traversal (the leftmost leaf).
    pub fn pop_post_order_leaf(&mut self) -> Option<Box<Node<T>>> {
        let mut current = self.root?;

        unsafe {
            // if the root is the only node (aka a leaf), set the tree root to null and
            // return it.

            loop {
                let front_node = match current.as_mut().children.front().copied() {
                    Some(node) => node,
                    None => {
                        // this branch can only be hit when the tree root node has no children, so
                        // set root to None
                        self.root = None;
                        self.len = 0;
                        return Some(Box::from_raw(current.as_ptr()));
                    }
                };

                // if the node has no children, pop it from its parent and return it back in a Box.
                if front_node.as_ref().children.is_empty() {
                    // pop the node from the list
                    current.as_mut().children.pop_front();
                    self.len -= 1;
                    return Some(Box::from_raw(front_node.as_ptr()));
                }

                // move down a level.
                current = front_node;
            }
        }
    }

    /// Visits all nodes in a preorder traversal.
    pub fn visit_preorder<F>(&self, f: F)
    where
        F: Fn(&T),
    {
        if let Some(node) = self.root {
            visit_preorder(node, &f);
        };
    }

    pub fn try_visit_preorder<F, E>(&self, f: F) -> Result<(), E>
    where
        F: Fn(&T) -> Result<(), E>,
    {
        if let Some(node) = self.root {
            try_visit_preorder(node, &f)?;
        };

        Ok(())
    }

    /// Visits all nodes in a preorder traversal, as pairs. pseudo-inverse of
    /// [`Tree::from_preorder_pairs`].
    pub fn visit_preorder_pairs<F>(&self, f: F)
    where
        F: Fn(&T, &T),
    {
        if let Some(node) = self.root {
            visit_preorder_pairs(node, &f);
        };
    }

    pub fn visit_preorder_pairs_short_curcuit<F, O>(&self, f: F) -> Option<O>
    where
        F: Fn(&T, &T) -> Option<O>,
    {
        match self.root {
            Some(root) => visit_preorder_pairs_short_circuit(root, &f),
            None => None,
        }
    }

    /// Constructs a [`Tree`] from a set of pairs representing the preorder traversal sequence.
    pub fn from_preorder_pairs<I>(pairs: I) -> Result<Self, (T, T)>
    where
        I: IntoIterator<Item = (T, T)>,
        T: PartialEq,
    {
        // let mut builder = Self::builder();
        //builder.insert_pairs(pairs)?;
        // Ok(builder.finalize())

        let mut iter = pairs.into_iter();

        let (mut current, mut current_child) = match iter.next() {
            Some((root, child)) => {
                let root = NonNull::from(Box::leak(Box::new(Node {
                    parent: None,
                    children: LinkedList::new(),
                    level: 0,
                    value: root,
                })));

                (root, insert_child(root, child))
            }
            None => {
                return Ok(Self {
                    root: None,
                    len: 0,
                    _marker: PhantomData,
                });
            }
        };

        // start at 2, since the root + immediate child are already put together.
        let mut len = 2;

        // grab a copy of the root node ptr now so we dont need to traverse upwards later on
        let root = current;
        unsafe {
            for (next_root, next_child) in iter {
                // if we're moving down a level:
                if current_child.as_ref().value == next_root {
                    current = current_child;
                }
                // if the current node isnt the same value, we need to work up to find the parent
                // that does match the value.
                else if current.as_ref().value != next_root {
                    if let Some(parent) = find_parent(current, &next_root) {
                        current = parent;
                    } else {
                        return Err((next_root, next_child));
                    }
                }
                // if next_root == current.value, we're adding another child, so no need to change
                // current.

                current_child = insert_child(current, next_child);
                len += 1;
            }
        }

        Ok(Self {
            root: Some(root),
            len,
            _marker: PhantomData,
        })
    }

    pub fn is_empty(&self) -> bool {
        let no_root = self.root.is_none();

        #[cfg(debug_assertions)]
        if no_root {
            debug_assert!(self.len == 0);
        }

        no_root
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[cfg(feature = "ptree")]
    pub fn write_to<W: std::io::Write>(&self, dst: &mut W) -> std::io::Result<()>
    where
        T: std::fmt::Display + Clone,
    {
        ptree::output::write_tree(&self, dst)
    }

    /// Returns a [`Cursor`] to the root node (or [`None`] if the [`Tree`] is empty).
    pub fn cursor(&mut self) -> Option<Cursor<'_, T>> {
        self.root.map(|current| Cursor {
            current,
            tree: self,
        })
    }

    pub fn into_post_order_iter(self) -> IntoPostOrderIter<T> {
        IntoPostOrderIter { tree: self }
    }
}

pub struct IntoPostOrderIter<T> {
    tree: Tree<T>,
}

impl<T> Iterator for IntoPostOrderIter<T> {
    type Item = Box<Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.tree.pop_post_order_leaf()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.tree.len;
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for IntoPostOrderIter<T> {}

pub struct Cursor<'a, T> {
    current: NonNull<Node<T>>,
    tree: &'a mut Tree<T>,
}

impl<'a, T> Cursor<'a, T> {
    pub fn value(&self) -> &T {
        unsafe { &self.current.as_ref().value }
    }
    pub fn value_mut(&mut self) -> &mut T {
        unsafe { &mut self.current.as_mut().value }
    }

    pub fn is_leaf(&self) -> bool {
        unsafe { self.current.as_ref().children.is_empty() }
    }

    pub fn is_root(&self) -> bool {
        unsafe { self.current.as_ref().parent.is_none() }
    }

    pub fn push_child(&mut self, value: T) {
        insert_child(self.current, value);
        self.tree.len += 1;
    }

    pub fn level(&self) -> u16 {
        unsafe { self.current.as_ref().level }
    }

    pub fn move_root(self) -> Self {
        Self {
            current: get_root(self.current),
            tree: self.tree,
        }
    }

    pub fn push_child_move(self, value: T) -> Self {
        let child = insert_child(self.current, value);
        self.tree.len += 1;

        Self {
            current: child,
            tree: self.tree,
        }
    }

    pub fn first_leaf(mut self) -> Self {
        loop {
            if self.is_leaf() {
                return self;
            }

            // we know there's a child, since is_leaf is false (and by extension
            // self.current.chilren.is_empty() is true).
            self = unsafe { self.first_child().unwrap_unchecked() };
        }
    }

    fn remove_inner(mut self) -> (NonNull<Node<T>>, Option<Self>) {
        let parent =
            detatch_parent(unsafe { self.current.as_mut() }, self.tree).map(|current| Cursor {
                current,
                tree: self.tree,
            });

        (self.current, parent)
    }

    pub fn split_into_tree(self) -> (Option<Self>, Tree<T>) {
        let init_len = self.tree.len;

        let (root, parent) = self.remove_inner();

        let nodes_left = parent.as_ref().map(|cursor| cursor.tree.len).unwrap_or(0);

        let new_len = init_len - nodes_left;

        (parent, Tree {
            root: Some(root),
            len: new_len,
            _marker: PhantomData,
        })
    }

    pub fn remove_leaf(self) -> Result<(Box<Node<T>>, Option<Self>), Self> {
        if !self.is_leaf() {
            return Err(self);
        }

        unsafe {
            let (leaf, parent) = self.remove_inner();
            Ok((Box::from_raw(leaf.as_ptr()), parent))
        }
    }

    pub fn first_child(self) -> Result<Self, Self> {
        unsafe {
            match self.current.as_ref().children.front().copied() {
                Some(current) => Ok(Self {
                    current,
                    tree: self.tree,
                }),
                None => Err(self),
            }
        }
    }

    pub fn move_parent(self) -> Result<Self, Self> {
        unsafe {
            match self.current.as_ref().parent {
                Some(parent) => Ok(Self {
                    current: parent,
                    tree: self.tree,
                }),
                None => Err(self),
            }
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct Node<T> {
    parent: Option<NonNull<Node<T>>>,
    children: LinkedList<NonNull<Node<T>>>,
    level: u16,
    value: T,
}

impl<T: fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let children = self
            .children
            .iter()
            .map(|s| unsafe { s.as_ref() })
            .collect::<Vec<_>>();

        f.debug_struct("Node")
            .field("value", &self.value)
            .field("level", &self.level)
            .field("children", &children)
            .finish()
    }
}

/// insert a new node from a value. Handles linking the nodes together, and returns a [`NonNull`]
/// ptr to the new child node.
fn insert_child<T>(mut parent: NonNull<Node<T>>, value: T) -> NonNull<Node<T>> {
    let node = NonNull::from(Box::leak(Box::new(Node {
        value,
        parent: Some(parent),
        level: unsafe { parent.as_ref().level + 1 },
        children: LinkedList::new(),
    })));

    unsafe { parent.as_mut().children.push_back(node) };
    node
}

fn visit_preorder<T, F>(node: NonNull<Node<T>>, f: F)
where
    F: Fn(&T) + Copy,
{
    let node_ref = unsafe { node.as_ref() };
    f(&node_ref.value);

    for child in node_ref.children.iter() {
        visit_preorder(*child, f);
    }
}

fn try_visit_preorder<T, F, E>(node: NonNull<Node<T>>, f: F) -> Result<(), E>
where
    F: Fn(&T) -> Result<(), E> + Copy,
{
    let node_ref = unsafe { node.as_ref() };
    f(&node_ref.value)?;

    for child in node_ref.children.iter() {
        try_visit_preorder(*child, f)?;
    }

    Ok(())
}

fn visit_preorder_pairs<T, F>(node: NonNull<Node<T>>, f: F)
where
    F: Fn(&T, &T) + Copy,
{
    let node_ref = unsafe { node.as_ref() };

    for child in node_ref.children.iter() {
        f(&node_ref.value, unsafe { &child.as_ref().value });
        visit_preorder_pairs(*child, f);
    }
}

fn visit_preorder_pairs_short_circuit<T, F, O>(node: NonNull<Node<T>>, f: F) -> Option<O>
where
    F: Fn(&T, &T) -> Option<O> + Copy,
{
    let node_ref = unsafe { node.as_ref() };

    for child in node_ref.children.iter() {
        if let Some(out) = f(&node_ref.value, unsafe { &child.as_ref().value })
            .or_else(|| visit_preorder_pairs_short_circuit(*child, f))
        {
            return Some(out);
        }
    }

    None
}

impl<T> Drop for Tree<T> {
    fn drop(&mut self) {
        let mut cursor = match self.cursor() {
            Some(cursor) => cursor,
            None => return,
        };

        loop {
            cursor = cursor.first_leaf();

            let (_, parent) = match cursor.remove_leaf() {
                Ok((node, parent)) => (node, parent),
                Err(_) => panic!(
                    "remove leaf must be ok if we're on a leaf, and the optional 'parent' handles \
                     hitting the root"
                ),
            };

            match parent {
                Some(parent) => cursor = parent,
                None => {
                    self.root = None;
                    debug_assert!(self.len == 0);
                    return;
                }
            }
        }
    }
}

#[cfg(feature = "ptree")]
impl<'a, T: std::fmt::Display + Clone> ptree::TreeItem for &'a Tree<T> {
    type Child = NodeWrapper<'a, T>;

    fn children(&self) -> std::borrow::Cow<[Self::Child]> {
        match self.root.as_ref() {
            Some(root) => std::borrow::Cow::Owned(vec![NodeWrapper(root)]),
            None => std::borrow::Cow::Borrowed(&[]),
        }
    }

    fn write_self<W: std::io::Write>(
        &self,
        _f: &mut W,
        _style: &ptree::Style,
    ) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "ptree")]
#[derive(Clone)]
pub struct NodeWrapper<'a, T>(&'a NonNull<Node<T>>);

#[cfg(feature = "ptree")]
impl<'a, T: std::fmt::Display + Clone> ptree::TreeItem for NodeWrapper<'a, T> {
    type Child = Self;

    fn children(&self) -> std::borrow::Cow<[Self::Child]> {
        std::borrow::Cow::Owned(unsafe {
            self.0
                .as_ref()
                .children
                .iter()
                .map(NodeWrapper)
                .collect::<Vec<_>>()
        })
    }

    fn write_self<W: std::io::Write>(
        &self,
        f: &mut W,
        style: &ptree::Style,
    ) -> std::io::Result<()> {
        write!(f, "{}", style.paint(unsafe { &self.0.as_ref().value }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Comes from an example in the ISO8211 spec. This builds a tree that looks like the following:
    ///
    ///       R
    ///       |
    ///       H
    ///    /  |  \
    ///   E   F   G
    ///  / \     / \
    /// A   B   C   D
    const TEST_PAIRS: [(char, char); 8] = [
        ('R', 'H'),
        ('H', 'E'),
        ('E', 'A'),
        ('E', 'B'),
        ('H', 'F'),
        ('H', 'G'),
        ('G', 'C'),
        ('G', 'D'),
    ];

    const PREORDER: [char; 9] = ['R', 'H', 'E', 'A', 'B', 'F', 'G', 'C', 'D'];

    const POSTORDER: [char; 9] = ['A', 'B', 'E', 'F', 'C', 'D', 'G', 'H', 'R'];

    #[test]
    fn test_tree_basics() {
        let tree = Tree::from_preorder_pairs(TEST_PAIRS).unwrap();

        let clone = tree.clone();

        assert_eq!(tree.len, clone.len);
        assert_eq!(tree, clone);

        let preorder_visited = std::cell::RefCell::new(Vec::new());
        tree.visit_preorder(|f| {
            preorder_visited.borrow_mut().push(*f);
        });
        assert_eq!(&PREORDER, preorder_visited.into_inner().as_slice());

        let preorder_pairs_visited = std::cell::RefCell::new(Vec::new());
        tree.visit_preorder_pairs(|a, b| {
            preorder_pairs_visited.borrow_mut().push((*a, *b));
        });

        assert_eq!(&TEST_PAIRS, preorder_pairs_visited.into_inner().as_slice());

        println!("{tree:#?}");

        #[cfg(all(feature = "ptree", not(miri)))]
        println!("{tree}");
        let postorder_iter = tree.into_post_order_iter().map(|node| node.value);

        assert!(postorder_iter.eq(POSTORDER.iter().copied()));
    }

    #[test]
    fn test_cursor() {
        let _tree = Tree::from_preorder_pairs(TEST_PAIRS).unwrap();
    }

    #[test]
    fn test_drop() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        #[derive(Debug, Clone)]
        pub struct DecrOnDrop(char, Arc<AtomicUsize>);

        impl PartialEq for DecrOnDrop {
            fn eq(&self, other: &Self) -> bool {
                self.0.eq(&other.0)
            }
        }

        impl Drop for DecrOnDrop {
            fn drop(&mut self) {
                println!("dropping: {}", self.0);
                self.1.fetch_sub(1, Ordering::SeqCst);
            }
        }
        let count = Arc::new(AtomicUsize::new(0));

        let mapped_pairs = TEST_PAIRS.map(|(a, b)| {
            (
                DecrOnDrop(a, Arc::clone(&count)),
                DecrOnDrop(b, Arc::clone(&count)),
            )
        });

        // sanity check counting of how many nodes there are.
        let tree = Tree::from_preorder_pairs(mapped_pairs).unwrap();

        let total_count = std::cell::Cell::new(0_usize);
        tree.visit_preorder(|_| total_count.set(total_count.get() + 1));

        assert_eq!(total_count.get(), PREORDER.len());

        // store the count
        count.store(total_count.get(), Ordering::SeqCst);

        // run Tree drop
        drop(tree);

        // make sure we decremented back to 0.
        assert_eq!(count.load(Ordering::SeqCst), 0);
    }
}
