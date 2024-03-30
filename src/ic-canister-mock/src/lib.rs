use std::collections::HashMap;
use std::path::Path;
use std::{cell::RefCell, collections::VecDeque};

use candid::{
    utils::{encode_args, ArgumentEncoder},
    Principal,
};
use ic_cdk::api::call::RejectionCode;
use libloading::Library;

mod implementation;
mod interface;

thread_local! {
    static QUEUE: RefCell<VecDeque<Message>> = <_>::default();
    static ACTIVE_MESSAGE: RefCell<Option<Message>> = <_>::default();
    static CALLBACK_CONTEXT: RefCell<Option<CallbackContext>> = <_>::default();
    static LOADED_CANISTERS: RefCell<HashMap<Principal, Canister>> = <_>::default();
}

#[derive(Debug)]
struct Message {
    from: Principal,
    to: Principal,
    method: String,
    payload: Option<Vec<u8>>,
    cycles: u128,
    callback: Option<Callback>,
    status: MessageStatus,
}

#[derive(Debug)]
struct Callback {
    reply_callback: unsafe extern "C-unwind" fn(env: *mut ()),
    reject_callback: unsafe extern "C-unwind" fn(env: *mut ()),
    cleanup_callback: Option<unsafe extern "C-unwind" fn(env: *mut ())>,
    reply_env: *mut (),
    reject_env: *mut (),
    cleanup_env: Option<*mut ()>,
    message_in_progress: Option<Box<Message>>,
}

struct CallbackContext {
    reject_code: Option<RejectionCode>,
    reject_message: Option<Vec<u8>>,
}

#[derive(Debug)]
enum MessageStatus {
    Pending,
    Continuing,
    Replied(Vec<u8>),
    Rejected(Vec<u8>, RejectionCode),
}

enum Canister {
    Dll(libloading::Library),
    //todo
}

pub unsafe fn load_canister(
    module: impl AsRef<Path>,
    principal: Principal,
    init_args: impl ArgumentEncoder,
) -> Result<(), ()> {
    let module = module.as_ref();
    let canister = if module == Path::new("@") {
        Canister::Dll(
            {
                #[cfg(windows)]
                {
                    libloading::os::windows::Library::this()
                }
                #[cfg(unix)]
                {
                    libloading::os::unix::Library::this()
                }
            }
            .into(),
        )
    } else {
        let library = unsafe { Library::new(module) }.map_err(|_| ())?;
        Canister::Dll(library)
    };
    let init_message = Message {
        from: Principal::management_canister(),
        to: principal,
        method: "#[init]".to_string(),
        callback: None,
        payload: Some(encode_args(init_args).map_err(|_| ())?),
        cycles: 0,
        status: MessageStatus::Pending,
    };
    LOADED_CANISTERS.with_borrow_mut(|c| c.insert(principal, canister));
    enqueue_message(init_message);
    Ok(())
}

fn enqueue_message(message: Message) {
    QUEUE.with_borrow_mut(|q| q.push_back(message));
}

fn with_active_message<T>(f: impl FnOnce(&Message) -> T) -> T {
    ACTIVE_MESSAGE.with_borrow(|m| {
        let m = m
            .as_ref()
            .unwrap_or_else(|| panic!("no active canister message"));
        f(m)
    })
}

fn with_active_message_mut<T>(f: impl FnOnce(&mut Message) -> T) -> T {
    ACTIVE_MESSAGE.with_borrow_mut(|m| {
        let m = m
            .as_mut()
            .unwrap_or_else(|| panic!("no active canister message"));
        f(m)
    })
}

fn run_all_messages() {
    while run_one_message() {}
}

// todo catch_unwind
fn run_one_message() -> bool {
    let Some(mut message) = QUEUE.with_borrow_mut(|q| q.pop_front()) else {
        return false;
    };
    let to = message.to;
    let method = &message.method;
    let function: unsafe extern "C-unwind" fn() = LOADED_CANISTERS.with_borrow(|l| {
        let canister = l
            .get(&to)
            .unwrap_or_else(|| panic!("no canister with ID {to}"));
        match canister {
            Canister::Dll(library) => unsafe {
                if let Some(method) = method.strip_prefix("#[") {
                    *library.get(format!("canister_{}\0", &method[..method.len() - 1]).as_bytes()).unwrap_or_else(|e| {
                        panic!("error looking up {method} in canister {to}: {e}")
                    })
                } else {
                    match library.get(format!("canister_query${method}\0").as_bytes()) {
                        Ok(sym) => *sym,
                        Err(e1) => *library
                            .get(format!("canister_update${method}\0").as_bytes())
                            .unwrap_or_else(|e2| {
                                panic!("error looking up {method} in canister {to}: either '{e1}' or '{e2}'")
                            }),
                    }
                }
            },
        }
    });
    ACTIVE_MESSAGE.with_borrow_mut(|m| *m = Some(message));
    unsafe { function() };
    let mut message = ACTIVE_MESSAGE
        .with_borrow_mut(|m| m.take())
        .unwrap_or_else(|| panic!("internal state error: lost the active message"));
    let method = &message.method;
    if matches!(message.status, MessageStatus::Pending) {
        panic!("canister {to} did not reply in method {method}")
    }
    while let Some(callback) = message.callback {
        match message.status {
            MessageStatus::Continuing => {
                panic!("internal state error: callback invoked without a return")
            }
            MessageStatus::Pending => unreachable!(),
            MessageStatus::Replied(value) => unsafe {
                message = *callback.message_in_progress.unwrap(); // todo
                message.payload = Some(value);
                ACTIVE_MESSAGE.with_borrow_mut(|m| *m = Some(message));
                CALLBACK_CONTEXT.with_borrow_mut(|c| {
                    *c = Some(CallbackContext {
                        reject_code: None,
                        reject_message: None,
                    })
                });
                (callback.reply_callback)(callback.reply_env);
                CALLBACK_CONTEXT.with_borrow_mut(|c| *c = None);
                message = ACTIVE_MESSAGE
                    .with_borrow_mut(|m| m.take())
                    .unwrap_or_else(|| {
                        panic!("internal state error: lost the active message (callback)")
                    });
            },
            MessageStatus::Rejected(reject_message, reject_code) => unsafe {
                message = *callback.message_in_progress.unwrap(); // todo
                message.payload = None;
                ACTIVE_MESSAGE.with_borrow_mut(|m| *m = Some(message));
                CALLBACK_CONTEXT.with_borrow_mut(|c| {
                    *c = Some(CallbackContext {
                        reject_message: Some(reject_message),
                        reject_code: Some(reject_code),
                    })
                });
                (callback.reject_callback)(callback.reject_env);
                CALLBACK_CONTEXT.with_borrow_mut(|c| *c = None);
                message = ACTIVE_MESSAGE
                    .with_borrow_mut(|m| m.take())
                    .unwrap_or_else(|| {
                        panic!("internal state error: lost the active message (callback)")
                    });
            },
        }
    }
    true
}
