//! This module defines an encoding of a RbTree that allows clients to marshal a tree to a blob and
//! unmarshal it later without recomputing any hashes.
//!
//! The V1 encoding has the following structure
//!
//! ```text
//! tree = record {
//!   magic : [u8; 3];
//!   version : u8;
//!   root : opt node;
//! }
//! node = record {
//!   flags : u8;
//!
//!   key_length : u32;
//!   value_length : u32;
//!   left_child_length : opt u32;
//!   right_child_length : opt u32;
//!
//!   subtree_hash : [u8; 32];
//!
//!   key : [u8; key_length];
//!   value : [u8; value_length];
//!   left_child : opt node (of left_child_length);
//!   right_child : opt node (of right_child_length);
//! ```
//!
//! The benefits of using a custom encoding instead of relying on a library:
//!   1. It's easy to support multiple versions of encoding.
//!   2. The custom encoding is structured in a way that allows us to construct
//!      witnesses without fully decoding the tree into memory.
//!   3. It's probably more compact and efficient.

use super::{is_balanced, AsHashTree, Color, Node, RbTree};
use std::convert::TryInto;

/// A flag indicating that the node being decoded is of red color.
const IS_RED: u8 = 1;
/// A flag indicating that the node being decoded has a left child.
const HAS_LEFT_CHILD: u8 = 1 << 1;
/// A flag indicating that the node being decoded has a right child.
const HAS_RIGHT_CHILD: u8 = 1 << 2;
/// Magic prefix that we use to mark the beginning of a tree encoding.
/// It also includes a version number as the last byte to allow us support multiple versions of
/// encoding in future without breaking backward compatibility.
const MAGIC: &[u8; 4] = b"RBT\x01";

/// A trait defining how to marshal/unmarshal a type to/from a byte array.
///
/// # Laws
///
/// * forall x: x.encode(&mut buf) => decode(&buf) = Ok(x)
pub trait Encode {
    type Error;

    // Append self to a byte array.
    //
    // Note that it's caller's responsibility to remember the length of the encoded message.
    fn encode(&self, buf: &mut Vec<u8>);

    // Decode self from a slice.
    // The slice must not contain any trailing bytes.
    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Encode for Vec<u8> {
    type Error = std::convert::Infallible;

    fn encode(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self)
    }
    fn decode(bytes: &[u8]) -> Result<Self, Self::Error> {
        Ok(bytes.to_vec())
    }
}

struct InputTooShort;

impl<KeyError, ValueError> From<InputTooShort> for RbTreeDecodeError<KeyError, ValueError> {
    fn from(_: InputTooShort) -> Self {
        RbTreeDecodeError::InputTooShort
    }
}

#[derive(Debug)]
pub enum RbTreeDecodeError<KeyError, ValueError> {
    BadMagic,
    InputTooShort,
    TrailingBytes,
    KeyDecodeError(KeyError),
    ValueDecodeError(ValueError),
}

impl<K, V> Encode for RbTree<K, V>
where
    K: 'static + Encode + AsRef<[u8]>,
    V: 'static + Encode + AsHashTree,
{
    type Error = RbTreeDecodeError<K::Error, V::Error>;

    fn encode(&self, buf: &mut Vec<u8>) {
        debug_assert!(buf.is_empty());
        buf.extend_from_slice(MAGIC);
        if let Some(root) = self.root.as_ref() {
            encode_node(root, buf);
        }
    }

    fn decode(buf: &[u8]) -> Result<Self, Self::Error> {
        if buf.len() < MAGIC.len() {
            return Err(RbTreeDecodeError::BadMagic);
        }
        if &buf[0..MAGIC.len()] != &MAGIC[..] {
            return Err(RbTreeDecodeError::BadMagic);
        }
        let buf = &buf[MAGIC.len()..];
        if buf.is_empty() {
            Ok(Self { root: None })
        } else {
            let root = Some(decode_node(buf)?);

            #[cfg(debug_assertions)]
            {
                root.as_ref().unwrap().validate_subtree_hash();
                assert!(is_balanced(&root));
            }

            Ok(Self { root })
        }
    }
}

fn encode_node<K: Encode, V: Encode>(node: &Node<K, V>, buf: &mut Vec<u8>) {
    fn reserve_slot_for_u32(buf: &mut Vec<u8>) -> usize {
        let offset = buf.len();
        buf.extend_from_slice(&0u32.to_le_bytes());
        offset
    }

    fn encode_nested<T>(f: fn(&T, &mut Vec<u8>), x: &T, buf: &mut Vec<u8>) -> usize {
        let start = buf.len();
        f(x, buf);
        buf.len() - start
    }

    fn set_field_len(len: u32, offset: usize, buf: &mut Vec<u8>) {
        buf[offset..offset + 4].copy_from_slice(&len.to_le_bytes())
    }

    // Flags are used to figure out how to interpret the rest of the data.
    let mut flags = 0;
    if node.color == Color::Red {
        flags |= IS_RED
    }
    if node.left.is_some() {
        flags |= HAS_LEFT_CHILD
    }
    if node.right.is_some() {
        flags |= HAS_RIGHT_CHILD
    }

    buf.push(flags);

    // Section with lengths of variable-length fields.
    // Lengths of left and right children are optional.

    let key_len_offset = reserve_slot_for_u32(buf);
    let val_len_offset = reserve_slot_for_u32(buf);
    let left_len_offset = node.left.as_ref().map(|_| reserve_slot_for_u32(buf));
    let right_len_offset = node.right.as_ref().map(|_| reserve_slot_for_u32(buf));

    // Hash of the sub-tree rooted at this node, 32-bytes long.

    buf.extend_from_slice(&node.subtree_hash);

    // Encode variable-length fields and update the corresponding lengths.
    // We reserved the place for all lengths in advance.

    let key_len = encode_nested(K::encode, &node.key, buf);
    set_field_len(key_len as u32, key_len_offset, buf);

    let val_len = encode_nested(V::encode, &node.value, buf);
    set_field_len(val_len as u32, val_len_offset, buf);

    if let Some(l) = node.left.as_ref() {
        debug_assert!(flags & HAS_LEFT_CHILD > 0);
        let left_len = encode_nested(encode_node, &*l, buf);
        set_field_len(left_len as u32, left_len_offset.unwrap(), buf);
    }

    if let Some(r) = node.right.as_ref() {
        debug_assert!(flags & HAS_RIGHT_CHILD > 0);
        let right_len = encode_nested(encode_node, &r, buf);
        set_field_len(right_len as u32, right_len_offset.unwrap(), buf);
    }
}

fn decode_node<K: Encode, V: Encode>(
    buf: &[u8],
) -> Result<Box<Node<K, V>>, RbTreeDecodeError<K::Error, V::Error>> {
    fn decode_length(buf: &[u8]) -> Result<(&[u8], u32), InputTooShort> {
        if buf.len() < 4 {
            Err(InputTooShort)
        } else {
            Ok((&buf[4..], u32::from_le_bytes(buf[0..4].try_into().unwrap())))
        }
    }
    if buf.is_empty() {
        return Err(RbTreeDecodeError::InputTooShort);
    }
    let flags = buf[0];
    let buf = &buf[1..];

    // Decode lengths of all variable-length fields.
    let (buf, key_len) = decode_length(buf)?;
    let (buf, val_len) = decode_length(buf)?;
    let (buf, left_len) = if flags & HAS_LEFT_CHILD > 0 {
        let (buf, len) = decode_length(buf)?;
        (buf, Some(len))
    } else {
        (buf, None)
    };
    let (buf, right_len) = if flags & HAS_RIGHT_CHILD > 0 {
        let (buf, len) = decode_length(buf)?;
        (buf, Some(len))
    } else {
        (buf, None)
    };

    // Decode sub-tree hash.
    if buf.len() < 32 {
        return Err(RbTreeDecodeError::InputTooShort);
    }
    let subtree_hash: [u8; 32] = buf[0..32].try_into().unwrap();
    let buf = &buf[32..];

    // Decode key.
    if buf.len() < key_len as usize {
        return Err(RbTreeDecodeError::InputTooShort);
    }
    let key =
        K::decode(&buf[0..key_len as usize]).map_err(|e| RbTreeDecodeError::KeyDecodeError(e))?;
    let buf = &buf[key_len as usize..];

    // Decode value.
    if buf.len() < val_len as usize {
        return Err(RbTreeDecodeError::InputTooShort);
    }
    let value =
        V::decode(&buf[0..val_len as usize]).map_err(|e| RbTreeDecodeError::ValueDecodeError(e))?;
    let buf = &buf[val_len as usize..];

    // Decode left child, if any.
    let (buf, left) = match left_len {
        None => (buf, None),
        Some(left_len) => {
            if buf.len() < left_len as usize {
                return Err(RbTreeDecodeError::InputTooShort);
            }
            (
                &buf[left_len as usize..],
                Some(decode_node(&buf[0..left_len as usize])?),
            )
        }
    };
    // Decode right child, if any.
    let (buf, right) = match right_len {
        None => (buf, None),
        Some(right_len) => {
            if buf.len() < right_len as usize {
                return Err(RbTreeDecodeError::InputTooShort);
            }
            (
                &buf[right_len as usize..],
                Some(decode_node(&buf[0..right_len as usize])?),
            )
        }
    };

    if !buf.is_empty() {
        return Err(RbTreeDecodeError::TrailingBytes);
    }

    Ok(Box::new(Node {
        key,
        value,
        left,
        right,
        color: if flags & IS_RED > 0 {
            Color::Red
        } else {
            Color::Black
        },
        subtree_hash,
    }))
}
