use crate::hashtree::{
    fork, fork_hash, labeled, labeled_hash, leaf_hash, Hash,
    HashTree::{self, Empty, Leaf, Pruned},
};
use std::borrow::Cow;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::fmt;

#[cfg(test)]
pub(crate) mod debug_alloc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    Red,
    Black,
}

impl Color {
    fn flip(self) -> Self {
        match self {
            Self::Red => Self::Black,
            Self::Black => Self::Red,
        }
    }
}

pub trait AsHashTree {
    /// Returns the root hash of the tree without constructing it.
    /// Must be equivalent to `as_hash_tree().reconstruct()`.
    fn root_hash(&self) -> Hash;

    /// Constructs a hash tree corresponding to the data.
    fn as_hash_tree(&self) -> HashTree<'_>;
}

impl AsHashTree for Vec<u8> {
    fn root_hash(&self) -> Hash {
        leaf_hash(&self[..])
    }

    fn as_hash_tree(&self) -> HashTree<'_> {
        Leaf(Cow::from(&self[..]))
    }
}

impl AsHashTree for Hash {
    fn root_hash(&self) -> Hash {
        leaf_hash(&self[..])
    }

    fn as_hash_tree(&self) -> HashTree<'_> {
        Leaf(Cow::from(&self[..]))
    }
}

impl<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> AsHashTree for RbTree<K, V> {
    fn root_hash(&self) -> Hash {
        if self.root.is_null() {
            Empty.reconstruct()
        } else {
            unsafe { (*self.root).subtree_hash }
        }
    }

    fn as_hash_tree(&self) -> HashTree<'_> {
        unsafe { Node::full_witness_tree(self.root, Node::data_tree) }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum KeyBound<'a> {
    Exact(&'a [u8]),
    Neighbor(&'a [u8]),
}

impl<'a> AsRef<[u8]> for KeyBound<'a> {
    fn as_ref(&self) -> &'a [u8] {
        match self {
            KeyBound::Exact(key) => key,
            KeyBound::Neighbor(key) => key,
        }
    }
}

// 1. All leaves are black.
// 2. Children of a red node are black.
// 3. Every path from a node goes through the same number of black
//    nodes.
struct Node<K, V> {
    key: K,
    value: V,
    left: *mut Node<K, V>,
    right: *mut Node<K, V>,
    color: Color,

    /// Hash of the full hash tree built from this node and its
    /// children. It needs to be recomputed after every rotation.
    subtree_hash: Hash,
}

impl<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> Node<K, V> {
    #[allow(clippy::let_and_return)]
    fn new(key: K, value: V) -> *mut Self {
        let value_hash = value.root_hash();
        let data_hash = labeled_hash(key.as_ref(), &value_hash);
        let node = Box::into_raw(Box::new(Self {
            key,
            value,
            left: Node::null(),
            right: Node::null(),
            color: Color::Red,
            subtree_hash: data_hash,
        }));

        #[cfg(test)]
        debug_alloc::mark_pointer_allocated(node);

        node
    }

    unsafe fn data_hash(n: *mut Self) -> Hash {
        debug_assert!(!n.is_null());
        labeled_hash((*n).key.as_ref(), &(*n).value.root_hash())
    }

    unsafe fn left_hash_tree<'a>(n: *mut Self) -> HashTree<'a> {
        debug_assert!(!n.is_null());
        if (*n).left.is_null() {
            Empty
        } else {
            Pruned((*(*n).left).subtree_hash)
        }
    }

    unsafe fn right_hash_tree<'a>(n: *mut Self) -> HashTree<'a> {
        debug_assert!(!n.is_null());
        if (*n).right.is_null() {
            Empty
        } else {
            Pruned((*(*n).right).subtree_hash)
        }
    }

    fn null() -> *mut Self {
        std::ptr::null::<Self>() as *mut Node<K, V>
    }

    unsafe fn data_tree<'a>(n: *mut Self) -> HashTree<'a> {
        debug_assert!(!n.is_null());

        labeled((*n).key.as_ref(), (*n).value.as_hash_tree())
    }

    unsafe fn subtree_with<'a>(
        n: *mut Self,
        f: impl FnOnce(&'a V) -> HashTree<'a>,
    ) -> HashTree<'a> {
        debug_assert!(!n.is_null());

        labeled((*n).key.as_ref(), f(&(*n).value))
    }

    unsafe fn witness_tree<'a>(n: *mut Self) -> HashTree<'a> {
        debug_assert!(!n.is_null());
        let value_hash = (*n).value.root_hash();
        labeled((*n).key.as_ref(), Pruned(value_hash))
    }

    unsafe fn full_witness_tree<'a>(
        n: *mut Self,
        f: unsafe fn(*mut Self) -> HashTree<'a>,
    ) -> HashTree<'a> {
        if n.is_null() {
            return Empty;
        }
        three_way_fork(
            Self::full_witness_tree((*n).left, f),
            f(n),
            Self::full_witness_tree((*n).right, f),
        )
    }

    unsafe fn delete(n: *mut Self) {
        if n.is_null() {
            return;
        }
        Self::delete((*n).left);
        Self::delete((*n).right);
        let _ = Box::from_raw(n);

        #[cfg(test)]
        debug_alloc::mark_pointer_deleted(n);
    }

    unsafe fn subtree_hash(n: *mut Self) -> Hash {
        if n.is_null() {
            return Empty.reconstruct();
        }

        let h = Node::data_hash(n);

        match ((*n).left.is_null(), (*n).right.is_null()) {
            (true, true) => h,
            (false, true) => fork_hash(&(*(*n).left).subtree_hash, &h),
            (true, false) => fork_hash(&h, &(*(*n).right).subtree_hash),
            (false, false) => fork_hash(
                &(*(*n).left).subtree_hash,
                &fork_hash(&h, &(*(*n).right).subtree_hash),
            ),
        }
    }
}

/// Implements mutable Leaf-leaning red-black trees as defined in
/// https://www.cs.princeton.edu/~rs/talks/LLRB/LLRB.pdf
pub struct RbTree<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> {
    root: *mut Node<K, V>,
}

impl<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> Drop for RbTree<K, V> {
    fn drop(&mut self) {
        unsafe { Node::delete(self.root) }
    }
}

impl<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> Default for RbTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static> RbTree<K, V> {
    pub fn new() -> Self {
        Self { root: Node::null() }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }

    pub fn get(&self, key: &[u8]) -> Option<&V> {
        unsafe {
            let mut root = self.root;
            while !root.is_null() {
                match key.cmp((*root).key.as_ref()) {
                    Equal => return Some(&(*root).value),
                    Less => root = (*root).left,
                    Greater => root = (*root).right,
                }
            }
            None
        }
    }

    /// Updates the value corresponding to the specified key.
    pub fn modify(&mut self, key: &[u8], f: impl FnOnce(&mut V)) {
        unsafe fn go<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            mut h: *mut Node<K, V>,
            k: &[u8],
            f: impl FnOnce(&mut V),
        ) {
            if h.is_null() {
                return;
            }

            match k.as_ref().cmp((*h).key.as_ref()) {
                Equal => {
                    f(&mut (*h).value);
                    (*h).subtree_hash = Node::subtree_hash(h);
                }
                Less => {
                    go((*h).left, k, f);
                    (*h).subtree_hash = Node::subtree_hash(h);
                }
                Greater => {
                    go((*h).right, k, f);
                    (*h).subtree_hash = Node::subtree_hash(h);
                }
            }
        }
        unsafe { go(self.root, key, f) }
    }

    fn range_witness<'a>(
        &'a self,
        left: Option<KeyBound<'a>>,
        right: Option<KeyBound<'a>>,
        f: unsafe fn(*mut Node<K, V>) -> HashTree<'a>,
    ) -> HashTree<'a> {
        match (left, right) {
            (None, None) => unsafe { Node::full_witness_tree(self.root, f) },
            (Some(l), None) => self.witness_range_above(l, f),
            (None, Some(r)) => self.witness_range_below(r, f),
            (Some(l), Some(r)) => self.witness_range_between(l, r, f),
        }
    }

    /// Constructs a hash tree that acts as a proof that there is a
    /// entry with the specified key in this map.  The proof also
    /// contains the value in question.
    ///
    /// If the key is not in the map, returns a proof of absence.
    pub fn witness<'a>(&'a self, key: &[u8]) -> HashTree<'a> {
        self.nested_witness(key, |v| v.as_hash_tree())
    }

    /// Like `witness`, but gives the caller more control over the
    /// construction of the value witness.  This method is useful for
    /// constructing witnesses for nested certified maps.
    pub fn nested_witness<'a>(
        &'a self,
        key: &[u8],
        f: impl FnOnce(&'a V) -> HashTree<'a>,
    ) -> HashTree<'a> {
        if let Some(t) = self.lookup_and_build_witness(key, f) {
            return t;
        }
        self.range_witness(
            self.lower_bound(key),
            self.upper_bound(key),
            Node::witness_tree,
        )
    }

    /// Returns a witness enumerating all the keys in this map.  The
    /// resulting tree doesn't include values, they are replaced with
    /// "Pruned" nodes.
    pub fn keys(&self) -> HashTree<'_> {
        unsafe { Node::full_witness_tree(self.root, Node::witness_tree) }
    }

    /// Returns a witness for the keys in the specified range.  The
    /// resulting tree doesn't include values, they are replaced with
    /// "Pruned" nodes.
    pub fn key_range(&self, first: &[u8], last: &[u8]) -> HashTree<'_> {
        self.range_witness(
            self.lower_bound(first),
            self.upper_bound(last),
            Node::witness_tree,
        )
    }

    /// Returns a witness for the key-value pairs in the specified range.
    /// The resulting tree contains both keys and values.
    pub fn value_range(&self, first: &[u8], last: &[u8]) -> HashTree<'_> {
        self.range_witness(
            self.lower_bound(first),
            self.upper_bound(last),
            Node::data_tree,
        )
    }

    /// Returns a witness that enumerates all the keys starting with
    /// the specified prefix.
    pub fn keys_with_prefix(&self, prefix: &[u8]) -> HashTree<'_> {
        self.range_witness(
            self.lower_bound(prefix),
            self.right_prefix_neighbor(prefix),
            Node::witness_tree,
        )
    }

    /// Enumerates all the key-value pairs in the tree.
    pub fn for_each<'a, F>(&'a self, mut f: F)
    where
        F: 'a + FnMut(&'a [u8], &'a V),
    {
        unsafe fn visit<'a, K, V, F>(n: *mut Node<K, V>, f: &mut F)
        where
            F: 'a + FnMut(&'a [u8], &'a V),
            K: 'static + AsRef<[u8]>,
            V: 'a + AsHashTree,
        {
            debug_assert!(!n.is_null());
            if !(*n).left.is_null() {
                visit((*n).left, f)
            }
            (*f)((*n).key.as_ref(), &(*n).value);
            if !(*n).right.is_null() {
                visit((*n).right, f)
            }
        }
        if self.root.is_null() {
            return;
        }
        unsafe { visit(self.root, &mut f) }
    }

    fn witness_range_above<'a>(
        &'a self,
        lo: KeyBound<'a>,
        f: unsafe fn(*mut Node<K, V>) -> HashTree<'a>,
    ) -> HashTree<'a> {
        unsafe fn go<'a, K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            n: *mut Node<K, V>,
            lo: KeyBound<'a>,
            f: unsafe fn(*mut Node<K, V>) -> HashTree<'a>,
        ) -> HashTree<'a> {
            if n.is_null() {
                return Empty;
            }
            match (*n).key.as_ref().cmp(lo.as_ref()) {
                Equal => three_way_fork(
                    Node::left_hash_tree(n),
                    match lo {
                        KeyBound::Exact(_) => f(n),
                        KeyBound::Neighbor(_) => Node::witness_tree(n),
                    },
                    Node::full_witness_tree((*n).right, f),
                ),
                Less => three_way_fork(
                    Node::left_hash_tree(n),
                    Pruned(Node::data_hash(n)),
                    go((*n).right, lo, f),
                ),
                Greater => three_way_fork(
                    go((*n).left, lo, f),
                    f(n),
                    Node::full_witness_tree((*n).right, f),
                ),
            }
        }
        unsafe { go(self.root, lo, f) }
    }

    fn witness_range_below<'a>(
        &'a self,
        hi: KeyBound<'a>,
        f: unsafe fn(*mut Node<K, V>) -> HashTree<'a>,
    ) -> HashTree<'a> {
        unsafe fn go<'a, K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            n: *mut Node<K, V>,
            hi: KeyBound<'a>,
            f: unsafe fn(*mut Node<K, V>) -> HashTree<'a>,
        ) -> HashTree<'a> {
            if n.is_null() {
                return Empty;
            }
            match (*n).key.as_ref().cmp(hi.as_ref()) {
                Equal => three_way_fork(
                    Node::full_witness_tree((*n).left, f),
                    match hi {
                        KeyBound::Exact(_) => f(n),
                        KeyBound::Neighbor(_) => Node::witness_tree(n),
                    },
                    Node::right_hash_tree(n),
                ),
                Greater => three_way_fork(
                    go((*n).left, hi, f),
                    Pruned(Node::data_hash(n)),
                    Node::right_hash_tree(n),
                ),
                Less => three_way_fork(
                    Node::full_witness_tree((*n).left, f),
                    f(n),
                    go((*n).right, hi, f),
                ),
            }
        }
        unsafe { go(self.root, hi, f) }
    }

    fn witness_range_between<'a>(
        &'a self,
        lo: KeyBound<'a>,
        hi: KeyBound<'a>,
        f: unsafe fn(*mut Node<K, V>) -> HashTree<'a>,
    ) -> HashTree<'a> {
        debug_assert!(
            lo.as_ref() <= hi.as_ref(),
            "lo = {:?} > hi = {:?}",
            lo.as_ref(),
            hi.as_ref()
        );
        unsafe fn go<'a, K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            n: *mut Node<K, V>,
            lo: KeyBound<'a>,
            hi: KeyBound<'a>,
            f: unsafe fn(*mut Node<K, V>) -> HashTree<'a>,
        ) -> HashTree<'a> {
            if n.is_null() {
                return Empty;
            }
            let k = (*n).key.as_ref();
            match (lo.as_ref().cmp(k), k.cmp(hi.as_ref())) {
                (Less, Less) => {
                    three_way_fork(go((*n).left, lo, hi, f), f(n), go((*n).right, lo, hi, f))
                }
                (Equal, Equal) => three_way_fork(
                    Node::left_hash_tree(n),
                    match (lo, hi) {
                        (KeyBound::Exact(_), _) => f(n),
                        (_, KeyBound::Exact(_)) => f(n),
                        _ => Node::witness_tree(n),
                    },
                    Node::right_hash_tree(n),
                ),
                (_, Equal) => three_way_fork(
                    go((*n).left, lo, hi, f),
                    match hi {
                        KeyBound::Exact(_) => f(n),
                        KeyBound::Neighbor(_) => Node::witness_tree(n),
                    },
                    Node::right_hash_tree(n),
                ),
                (Equal, _) => three_way_fork(
                    Node::left_hash_tree(n),
                    match lo {
                        KeyBound::Exact(_) => f(n),
                        KeyBound::Neighbor(_) => Node::witness_tree(n),
                    },
                    go((*n).right, lo, hi, f),
                ),
                (Less, Greater) => three_way_fork(
                    go((*n).left, lo, hi, f),
                    Pruned(Node::data_hash(n)),
                    Node::right_hash_tree(n),
                ),
                (Greater, Less) => three_way_fork(
                    Node::left_hash_tree(n),
                    Pruned(Node::data_hash(n)),
                    go((*n).right, lo, hi, f),
                ),
                _ => Pruned((*n).subtree_hash),
            }
        }
        unsafe { go(self.root, lo, hi, f) }
    }

    fn lower_bound<'a>(&'a self, key: &[u8]) -> Option<KeyBound<'a>> {
        unsafe fn go<'a, K: 'static + AsRef<[u8]>, V>(
            n: *mut Node<K, V>,
            key: &[u8],
        ) -> Option<KeyBound<'a>> {
            if n.is_null() {
                return None;
            }
            let node_key = (*n).key.as_ref();
            match node_key.cmp(key) {
                Less => go((*n).right, key).or(Some(KeyBound::Neighbor(node_key))),
                Equal => Some(KeyBound::Exact(node_key)),
                Greater => go((*n).left, key),
            }
        }
        unsafe { go(self.root, key) }
    }

    fn upper_bound<'a>(&'a self, key: &[u8]) -> Option<KeyBound<'a>> {
        unsafe fn go<'a, K: 'static + AsRef<[u8]>, V>(
            n: *mut Node<K, V>,
            key: &[u8],
        ) -> Option<KeyBound<'a>> {
            if n.is_null() {
                return None;
            }
            let node_key = (*n).key.as_ref();
            match node_key.cmp(key) {
                Less => go((*n).right, key),
                Equal => Some(KeyBound::Exact(node_key)),
                Greater => go((*n).left, key).or(Some(KeyBound::Neighbor(node_key))),
            }
        }
        unsafe { go(self.root, key) }
    }

    fn right_prefix_neighbor<'a>(&'a self, prefix: &[u8]) -> Option<KeyBound<'a>> {
        fn is_prefix_of(p: &[u8], x: &[u8]) -> bool {
            if p.len() > x.len() {
                return false;
            }
            &x[0..p.len()] == p
        }
        unsafe fn go<'a, K: 'static + AsRef<[u8]>, V>(
            n: *mut Node<K, V>,
            prefix: &[u8],
        ) -> Option<KeyBound<'a>> {
            if n.is_null() {
                return None;
            }
            let node_key = (*n).key.as_ref();
            match node_key.cmp(prefix) {
                Greater if is_prefix_of(prefix, node_key) => go((*n).right, prefix),
                Greater => go((*n).left, prefix).or(Some(KeyBound::Neighbor(node_key))),
                Less | Equal => go((*n).right, prefix),
            }
        }
        unsafe { go(self.root, prefix) }
    }

    fn lookup_and_build_witness<'a>(
        &'a self,
        key: &[u8],
        f: impl FnOnce(&'a V) -> HashTree<'a>,
    ) -> Option<HashTree<'a>> {
        unsafe fn go<'a, K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            n: *mut Node<K, V>,
            key: &[u8],
            f: impl FnOnce(&'a V) -> HashTree<'a>,
        ) -> Option<HashTree<'a>> {
            if n.is_null() {
                return None;
            }
            match key.cmp((*n).key.as_ref()) {
                Equal => Some(three_way_fork(
                    Node::left_hash_tree(n),
                    Node::subtree_with(n, f),
                    Node::right_hash_tree(n),
                )),
                Less => {
                    let subtree = go((*n).left, key, f)?;
                    Some(three_way_fork(
                        subtree,
                        Pruned(Node::data_hash(n)),
                        Node::right_hash_tree(n),
                    ))
                }
                Greater => {
                    let subtree = go((*n).right, key, f)?;
                    Some(three_way_fork(
                        Node::left_hash_tree(n),
                        Pruned(Node::data_hash(n)),
                        subtree,
                    ))
                }
            }
        }
        unsafe { go(self.root, key, f) }
    }

    /// Inserts a key-value entry into the map.
    pub fn insert(&mut self, key: K, value: V) {
        unsafe fn go<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            mut h: *mut Node<K, V>,
            k: K,
            v: V,
        ) -> *mut Node<K, V> {
            if h.is_null() {
                return Node::new(k, v);
            }

            match k.as_ref().cmp((*h).key.as_ref()) {
                Equal => {
                    (*h).value = v;
                    (*h).subtree_hash = Node::subtree_hash(h);
                }
                Less => {
                    (*h).left = go((*h).left, k, v);
                    (*h).subtree_hash = Node::subtree_hash(h);
                }
                Greater => {
                    (*h).right = go((*h).right, k, v);
                    (*h).subtree_hash = Node::subtree_hash(h);
                }
            }
            balance(h)
        }
        unsafe {
            let mut root = go(self.root, key, value);
            (*root).color = Color::Black;

            #[cfg(test)]
            debug_assert!(
                is_balanced(root),
                "the tree is not balanced:\n{:?}",
                DebugView(root)
            );
            #[cfg(test)]
            debug_assert!(!has_dangling_pointers(root));

            self.root = root;
        }
    }

    /// Removes the specified key from the map.
    pub fn delete(&mut self, key: &[u8]) {
        unsafe fn move_red_left<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            mut h: *mut Node<K, V>,
        ) -> *mut Node<K, V> {
            flip_colors(h);
            if is_red((*(*h).right).left) {
                (*h).right = rotate_right((*h).right);
                h = rotate_left(h);
                flip_colors(h);
            }
            h
        }

        unsafe fn move_red_right<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            mut h: *mut Node<K, V>,
        ) -> *mut Node<K, V> {
            flip_colors(h);
            if is_red((*(*h).left).left) {
                h = rotate_right(h);
                flip_colors(h);
            }
            h
        }

        #[inline]
        unsafe fn min<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            mut h: *mut Node<K, V>,
        ) -> *mut Node<K, V> {
            while !(*h).left.is_null() {
                h = (*h).left;
            }
            h
        }

        unsafe fn delete_min<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            mut h: *mut Node<K, V>,
        ) -> *mut Node<K, V> {
            if (*h).left.is_null() {
                debug_assert!((*h).right.is_null());
                Node::delete(h);
                return Node::null();
            }
            if !is_red((*h).left) && !is_red((*(*h).left).left) {
                h = move_red_left(h);
            }
            (*h).left = delete_min((*h).left);
            (*h).subtree_hash = Node::subtree_hash(h);
            balance(h)
        }

        unsafe fn go<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
            mut h: *mut Node<K, V>,
            key: &[u8],
        ) -> *mut Node<K, V> {
            if key < (*h).key.as_ref() {
                if !is_red((*h).left) && !is_red((*(*h).left).left) {
                    h = move_red_left(h);
                }
                (*h).left = go((*h).left, key);
            } else {
                if is_red((*h).left) {
                    h = rotate_right(h);
                }
                if key == (*h).key.as_ref() && (*h).right.is_null() {
                    debug_assert!((*h).left.is_null());
                    Node::delete(h);
                    return Node::null();
                }

                if !is_red((*h).right) && !is_red((*(*h).right).left) {
                    h = move_red_right(h);
                }

                if key == (*h).key.as_ref() {
                    let m = min((*h).right);
                    std::mem::swap(&mut (*h).key, &mut (*m).key);
                    std::mem::swap(&mut (*h).value, &mut (*m).value);
                    (*h).right = delete_min((*h).right);
                } else {
                    (*h).right = go((*h).right, key);
                }
            }
            (*h).subtree_hash = Node::subtree_hash(h);
            balance(h)
        }

        unsafe {
            if self.get(key).is_none() {
                return;
            }
            if !is_red((*self.root).left) && !is_red((*self.root).right) {
                (*self.root).color = Color::Red;
            }
            self.root = go(self.root, key);
            if !self.root.is_null() {
                (*self.root).color = Color::Black;
            }

            #[cfg(test)]
            debug_assert!(
                is_balanced(self.root),
                "unbalanced map: {:?}",
                DebugView(self.root)
            );

            debug_assert!(self.get(key).is_none());
        }
    }
}

fn three_way_fork<'a>(l: HashTree<'a>, m: HashTree<'a>, r: HashTree<'a>) -> HashTree<'a> {
    match (l, m, r) {
        (Empty, m, Empty) => m,
        (l, m, Empty) => fork(l, m),
        (Empty, m, r) => fork(m, r),
        (Pruned(lhash), Pruned(mhash), Pruned(rhash)) => {
            Pruned(fork_hash(&lhash, &fork_hash(&mhash, &rhash)))
        }
        (l, Pruned(mhash), Pruned(rhash)) => fork(l, Pruned(fork_hash(&mhash, &rhash))),
        (l, m, r) => fork(l, fork(m, r)),
    }
}

// helper functions
unsafe fn is_red<K, V>(x: *const Node<K, V>) -> bool {
    if x.is_null() {
        false
    } else {
        (*x).color == Color::Red
    }
}

unsafe fn balance<K: AsRef<[u8]> + 'static, V: AsHashTree + 'static>(
    mut h: *mut Node<K, V>,
) -> *mut Node<K, V> {
    assert!(!h.is_null());

    if is_red((*h).right) && !is_red((*h).left) {
        h = rotate_left(h);
    }
    if is_red((*h).left) && is_red((*(*h).left).left) {
        h = rotate_right(h);
    }
    if is_red((*h).left) && is_red((*h).right) {
        flip_colors(h)
    }
    h
}

/// Make a left-leaning link lean to the right.
unsafe fn rotate_right<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
    h: *mut Node<K, V>,
) -> *mut Node<K, V> {
    debug_assert!(!h.is_null());
    debug_assert!(is_red((*h).left));

    let mut x = (*h).left;
    (*h).left = (*x).right;
    (*x).right = h;
    (*x).color = (*(*x).right).color;
    (*(*x).right).color = Color::Red;

    (*h).subtree_hash = Node::subtree_hash(h);
    (*x).subtree_hash = Node::subtree_hash(x);

    x
}

unsafe fn rotate_left<K: 'static + AsRef<[u8]>, V: AsHashTree + 'static>(
    h: *mut Node<K, V>,
) -> *mut Node<K, V> {
    debug_assert!(!h.is_null());
    debug_assert!(is_red((*h).right));

    let mut x = (*h).right;
    (*h).right = (*x).left;
    (*x).left = h;
    (*x).color = (*(*x).left).color;
    (*(*x).left).color = Color::Red;

    (*h).subtree_hash = Node::subtree_hash(h);
    (*x).subtree_hash = Node::subtree_hash(x);

    x
}

unsafe fn flip_colors<K, V>(h: *mut Node<K, V>) {
    (*h).color = (*h).color.flip();
    (*(*h).left).color = (*(*h).left).color.flip();
    (*(*h).right).color = (*(*h).right).color.flip();
}

#[cfg(test)]
unsafe fn is_balanced<K, V>(root: *mut Node<K, V>) -> bool {
    unsafe fn go<K, V>(node: *mut Node<K, V>, mut num_black: usize) -> bool {
        if node.is_null() {
            return num_black == 0;
        }
        if !is_red(node) {
            debug_assert!(num_black > 0);
            num_black -= 1;
        } else {
            assert!(!is_red((*node).left));
            assert!(!is_red((*node).right));
        }
        go((*node).left, num_black) && go((*node).right, num_black)
    }

    let mut num_black = 0;
    let mut x = root;
    while !x.is_null() {
        if !is_red(x) {
            num_black += 1;
        }
        x = (*x).left;
    }
    go(root, num_black)
}

#[cfg(test)]
unsafe fn has_dangling_pointers<K, V>(root: *mut Node<K, V>) -> bool {
    if root.is_null() {
        return false;
    }

    !debug_alloc::is_live(root)
        || has_dangling_pointers((*root).left)
        || has_dangling_pointers((*root).right)
}

struct DebugView<K, V>(*const Node<K, V>);

impl<K: AsRef<[u8]>, V> fmt::Debug for DebugView<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe fn go<K: AsRef<[u8]>, V>(
            f: &mut fmt::Formatter<'_>,
            h: *const Node<K, V>,
            offset: usize,
        ) -> fmt::Result {
            if h.is_null() {
                writeln!(f, "{:width$}[B] <null>", "", width = offset)
            } else {
                writeln!(
                    f,
                    "{:width$}[{}] {:?}",
                    "",
                    if is_red(h) { "R" } else { "B" },
                    (*h).key.as_ref(),
                    width = offset
                )?;
                go(f, (*h).left, offset + 2)?;
                go(f, (*h).right, offset + 2)
            }
        }
        unsafe { go(f, self.0, 0) }
    }
}

#[cfg(test)]
mod test;
