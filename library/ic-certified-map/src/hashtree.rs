#[cfg(test)]
mod test;

use serde::{ser::SerializeSeq, Serialize, Serializer};
use serde_bytes::Bytes;
use sha2::{Digest, Sha256};
use std::borrow::Cow;

/// SHA-256 hash bytes.
pub type Hash = [u8; 32];

/// HashTree as defined in the [interfaces spec](https://internetcomputer.org/docs/current/references/ic-interface-spec#certificate).
#[derive(Debug, Clone, Default)]
pub enum HashTree<'a> {
    /// No child nodes; a proof of absence.
    #[default]
    Empty,
    /// Left and right child branches.
    Fork(Box<(HashTree<'a>, HashTree<'a>)>),
    /// A labeled child node.
    Labeled(&'a [u8], Box<HashTree<'a>>),
    /// A leaf node containing a value or hash.
    Leaf(Cow<'a, [u8]>),
    /// A branch that has been removed from this view of the tree, but is not necessarily absent.
    Pruned(Hash),
}

/// Shorthand for [`HashTree::Fork`].
pub fn fork<'a>(l: HashTree<'a>, r: HashTree<'a>) -> HashTree<'a> {
    HashTree::Fork(Box::new((l, r)))
}

/// Shorthand for [`HashTree::Labeled`].
pub fn labeled<'a>(l: &'a [u8], t: HashTree<'a>) -> HashTree<'a> {
    HashTree::Labeled(l, Box::new(t))
}

/// Identifiably hashes a fork in the branch. Used for hashing [`HashTree::Fork`].
pub fn fork_hash(l: &Hash, r: &Hash) -> Hash {
    let mut h = domain_sep("ic-hashtree-fork");
    h.update(&l[..]);
    h.update(&r[..]);
    h.finalize().into()
}

/// Identifiably hashes a leaf node's data. Used for hashing [`HashTree::Leaf`].
pub fn leaf_hash(data: &[u8]) -> Hash {
    let mut h = domain_sep("ic-hashtree-leaf");
    h.update(data);
    h.finalize().into()
}

/// Identifiably hashes a label for this branch. Used for hashing [`HashTree::Labeled`].
pub fn labeled_hash(label: &[u8], content_hash: &Hash) -> Hash {
    let mut h = domain_sep("ic-hashtree-labeled");
    h.update(label);
    h.update(&content_hash[..]);
    h.finalize().into()
}

impl HashTree<'_> {
    /// Produces the root hash of the tree.
    pub fn reconstruct(&self) -> Hash {
        match self {
            Self::Empty => domain_sep("ic-hashtree-empty").finalize().into(),
            Self::Fork(f) => fork_hash(&f.0.reconstruct(), &f.1.reconstruct()),
            Self::Labeled(l, t) => {
                let thash = t.reconstruct();
                labeled_hash(l, &thash)
            }
            Self::Leaf(data) => leaf_hash(data),
            Self::Pruned(h) => *h,
        }
    }
}

impl Serialize for HashTree<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self {
            HashTree::Empty => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(&0u8)?;
                seq.end()
            }
            HashTree::Fork(p) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(&1u8)?;
                seq.serialize_element(&p.0)?;
                seq.serialize_element(&p.1)?;
                seq.end()
            }
            HashTree::Labeled(label, tree) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(&2u8)?;
                seq.serialize_element(Bytes::new(label))?;
                seq.serialize_element(&tree)?;
                seq.end()
            }
            HashTree::Leaf(leaf_bytes) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(&3u8)?;
                seq.serialize_element(Bytes::new(leaf_bytes.as_ref()))?;
                seq.end()
            }
            HashTree::Pruned(digest) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(&4u8)?;
                seq.serialize_element(Bytes::new(&digest[..]))?;
                seq.end()
            }
        }
    }
}

fn domain_sep(s: &str) -> sha2::Sha256 {
    let buf: [u8; 1] = [s.len() as u8];
    let mut h = Sha256::new();
    h.update(&buf[..]);
    h.update(s.as_bytes());
    h
}
