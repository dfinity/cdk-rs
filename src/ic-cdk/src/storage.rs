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

pub fn delete<T: Sized + Default + 'static>() -> bool {
    let type_id = std::any::TypeId::of::<T>();

    storage().remove(&type_id).is_some()
}

pub fn get_mut<T: Sized + Default + 'static>() -> &'static mut T {
    let type_id = std::any::TypeId::of::<T>();

    let store = storage();

    store
        .entry(type_id)
        .or_insert_with(|| Box::new(T::default()))
        .downcast_mut()
        .expect("Unexpected value of invalid type.")
}

pub fn get<T: Sized + Default + 'static>() -> &'static T {
    get_mut::<T>()
}
