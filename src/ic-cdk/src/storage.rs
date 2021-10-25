//! Tools for managing storage of data types in canister.
//!
//! Each data type `T` will have one storage for it.
use crate::api::stable;
use std::any::{Any, TypeId};
use std::collections::BTreeMap;

type StorageTree = BTreeMap<TypeId, Box<dyn Any>>;

static mut STORAGE: Option<StorageTree> = None;

fn storage() -> &'static mut StorageTree {
    unsafe {
        if let Some(s) = &mut STORAGE {
            s
        } else {
            STORAGE = Some(BTreeMap::new());
            storage()
        }
    }
}

/// Deletes the storage.
pub fn delete<T: Sized + Default + 'static>() -> bool {
    let type_id = std::any::TypeId::of::<T>();

    storage().remove(&type_id).is_some()
}

/// Returns a mutable reference of the storage.
///
/// This will create the storage if it doesn't exist.
pub fn get_mut<T: Sized + Default + 'static>() -> &'static mut T {
    let type_id = std::any::TypeId::of::<T>();

    let store = storage();

    store
        .entry(type_id)
        .or_insert_with(|| Box::new(T::default()))
        .downcast_mut()
        .expect("Unexpected value of invalid type.")
}

/// Returns a share reference of the storage.
///
/// This will create the storage if it doesn't exist.
pub fn get<T: Sized + Default + 'static>() -> &'static T {
    get_mut::<T>()
}

/// Saves the storage into the stable memory.
///
/// This will override any value previously stored in stable memory.
pub fn stable_save<T>(t: T) -> Result<(), candid::Error>
where
    T: candid::utils::ArgumentEncoder,
{
    candid::write_args(&mut stable::StableWriter::default(), t)
}

/// Restores a value from the stable memory to the storage.
///
/// There can only be one value in stable memory, currently.
pub fn stable_restore<T>() -> Result<T, String>
where
    T: for<'de> candid::utils::ArgumentDecoder<'de>,
{
    let bytes = stable::stable_bytes();

    let mut de =
        candid::de::IDLDeserialize::new(bytes.as_slice()).map_err(|e| format!("{:?}", e))?;
    let res = candid::utils::ArgumentDecoder::decode(&mut de).map_err(|e| format!("{:?}", e))?;
    // The idea here is to ignore an error that comes from Candid, because we have trailing
    // bytes.
    let _ = de.done();
    Ok(res)
}
