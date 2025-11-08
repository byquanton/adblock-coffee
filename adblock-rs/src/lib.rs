extern crate alloc;

mod adblock;
mod errors;
mod logger;
mod wrapper;

use jni::objects::{JObject, JObjectArray, JString};
use jni::sys::{jboolean, jlong, jobject};
use jni::JNIEnv;

use crate::wrapper::*;

const RUST_EXCEPTION_CLASS: &str = "eu/byquanton/adblock/exception/RustException";

#[no_mangle]
pub extern "system" fn Java_eu_byquanton_adblock_AdvtBlocker_initObject(
    mut env: JNIEnv,
    _class: JObject,
    rules: JObjectArray,
) -> jlong {
    match init_object_wrapped(&mut env, &rules) {
        Ok(ptr) => ptr,
        Err(err) => {
            env.throw_new(RUST_EXCEPTION_CLASS, err.to_string())
                .expect("failed to find RustException java class");
            -1_i64 as jlong
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_eu_byquanton_adblock_AdvtBlocker_destroyObject(
    mut env: JNIEnv,
    _class: JObject,
    ptr: jlong,
) -> jboolean {
    match destroy_object_wrapped(&mut env, ptr) {
        Ok(status) => status,
        Err(err) => {
            env.throw_new(RUST_EXCEPTION_CLASS, err.to_string())
                .expect("failed to find RustException java class");
            false as jboolean
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_eu_byquanton_adblock_AdvtBlocker_checkNetworkUrls(
    mut env: JNIEnv,
    _class: JObject,
    ptr: jlong,
    url: JString,
    src_url: JString,
    req_type: JString,
) -> jboolean {
    match check_net_urls_wrapped(&mut env, ptr, &url, &src_url, &req_type) {
        Ok(result) => result,
        Err(err) => {
            env.throw_new(RUST_EXCEPTION_CLASS, err.to_string())
                .expect("failed to find RustException java class");
            false as jboolean
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_eu_byquanton_adblock_AdvtBlocker_getUrlCosmeticResourcesNative(
    mut env: JNIEnv,
    _class: JObject,
    ptr: jlong,
    url: JString,
) -> jobject {
    match url_cosmetic_resources_wrapped(&mut env, &_class, ptr, &url) {
        Ok(obj) => obj,
        Err(err) => {
            env.throw_new(RUST_EXCEPTION_CLASS, err.to_string())
                .expect("failed to find RustException java class");
            std::ptr::null_mut()
        }
    }
}
