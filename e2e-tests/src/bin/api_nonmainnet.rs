use ic_cdk::api::*;

#[export_name = "canister_query call_env_var_count"]
fn call_env_var_count() {
    let count = env_var_count();
    assert_eq!(count, 2);
    msg_reply(vec![]);
}

#[export_name = "canister_query call_env_var_name"]
fn call_env_var_name() {
    // This is expected to panic as no environment variables are set.
    assert_eq!(env_var_name(0), "key1");
    assert_eq!(env_var_name(1), "key2");
    msg_reply(vec![]);
}

#[export_name = "canister_query call_env_var_name_exists"]
fn call_env_var_name_exists() {
    assert!(env_var_name_exists("key1"));
    assert!(env_var_name_exists("key2"));
    assert!(!env_var_name_exists("non_existent_var"));
    msg_reply(vec![]);
}

#[export_name = "canister_query call_env_var_value"]
fn call_env_var_value() {
    assert_eq!(env_var_value("key1"), "value1");
    assert_eq!(env_var_value("key2"), "value2");
    msg_reply(vec![]);
}

fn main() {}
