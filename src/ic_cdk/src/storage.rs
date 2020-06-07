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

pub fn get<T: Sized + Default + 'static>() -> &'static mut T {
    let type_id = std::any::TypeId::of::<T>();

    let store = storage();

    if store.contains_key(&type_id) {
        let v = store.get_mut(&type_id).unwrap();

        (v.as_mut())
            .downcast_mut::<T>()
            .expect("Unexpected value of invalid type.")
    } else {
        let value = Box::new(T::default());
        store.insert(type_id, value);

        get::<T>()
    }
}
