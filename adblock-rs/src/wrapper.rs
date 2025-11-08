use jni::objects::{GlobalRef, JClass, JList, JObject, JObjectArray, JString};
use jni::sys::jobject;
use jni::sys::{jboolean, jint, jlong};
use jni::JNIEnv;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

use crate::adblock::AdvtBlocker;
use crate::errors::{Result, RustException};

lazy_static! {
    static ref INSTANCE_POOL: Mutex<HashMap<jlong, AdvtBlocker>> = Mutex::new(HashMap::new());
    static ref NEXT_ID: AtomicI64 = AtomicI64::new(1);
    static ref RETURNED_GLOBAL_REFS: Mutex<Vec<GlobalRef>> = Mutex::new(Vec::new());
}

pub(crate) fn init_object_wrapped(env: &mut JNIEnv, rules: &JObjectArray) -> Result<jlong> {
    let conv_rules = extract_list_str(env, rules)?;

    let advt_instance = AdvtBlocker::new(conv_rules);

    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst) as jlong;

    let mut instance_lock = INSTANCE_POOL.lock()?;
    instance_lock.insert(id, advt_instance);

    Ok(id)
}

pub(crate) fn destroy_object_wrapped(_env: &mut JNIEnv, ptr: jlong) -> Result<jboolean> {
    let mut instance_lock = INSTANCE_POOL.lock()?;

    let Some(instance) = instance_lock.remove(&ptr) else {
        let msg = format!("failed to remove instance: {ptr:?}");
        return Err(RustException::InstanceAccess(msg));
    };

    drop(instance);
    Ok(true as jboolean)
}

pub(crate) fn check_net_urls_wrapped(
    env: &mut JNIEnv,
    ptr: jlong,
    url: &JString,
    src_url: &JString,
    req_type: &JString,
) -> Result<jboolean> {
    let instance_lock = INSTANCE_POOL.lock()?;
    let Some(advt_blocker) = instance_lock.get(&ptr) else {
        let msg = format!("failed to get instance: {ptr:?}");
        return Err(RustException::InstanceAccess(msg));
    };

    let url_str = extract_str(env, url)?;
    let src_url_str = extract_str(env, src_url)?;
    let req_type_str = extract_str(env, req_type)?;

    advt_blocker
        .check_network_urls(&url_str, &src_url_str, &req_type_str)
        .map(|result| result as jboolean)
}

pub(crate) fn url_cosmetic_resources_wrapped(
    env: &mut JNIEnv,
    caller: &JObject,
    ptr: jlong,
    url: &JString,
) -> Result<jobject> {
    let instance_lock = INSTANCE_POOL.lock()?;
    let Some(advt_blocker) = instance_lock.get(&ptr) else {
        let msg = format!("failed to get instance: {ptr:?}");
        return Err(RustException::InstanceAccess(msg));
    };

    let url_str = extract_str(env, url)?;

    let result = advt_blocker.url_cosmetic_resources(&url_str)?;

    let raw = create_cosmetic_resources(
        env,
        Some(caller),
        result.hide_selectors,
        result.injected_script,
        result.exceptions,
        result.generichide,
    )?;

    Ok(raw)
}

fn extract_list_str<'env>(
    env: &'env mut JNIEnv,
    j_obj_arr: &'env JObjectArray,
) -> Result<Vec<String>> {
    let j_list = env.get_list(j_obj_arr)?;
    let j_list_size = j_list.size(env)?;

    let mut list_data = Vec::with_capacity(j_list_size as usize);
    for index in 0..j_list_size {
        match extract_entity(env, &j_list, index) {
            Ok(data) => list_data.push(data),
            Err(err) => {
                log::error!("failed to extract str from java object: {err:#?}");
                continue;
            }
        }
    }

    Ok(list_data)
}

fn extract_entity(env: &mut JNIEnv, j_list: &JList, index: jint) -> Result<String> {
    let j_obj_opt = j_list.get(env, index)?;
    let j_str = match j_obj_opt {
        Some(j_obj) => JString::from(j_obj),
        None => {
            let msg = format!("parsed rule is none: {j_obj_opt:?}. skipped...");
            return Err(RustException::ParseJavaObject(msg));
        }
    };

    extract_str(env, &j_str)
}

fn extract_str<'env>(env: &'env mut JNIEnv, j_obj: &'env JString) -> Result<String> {
    let j_str = env.get_string(j_obj)?;
    let str_obj = j_str.to_str()?;
    Ok(str_obj.to_string())
}

pub fn create_cosmetic_resources(
    env: &mut JNIEnv,
    caller: Option<&JObject>,
    hide: HashSet<String>,
    script: String,
    exceptions: HashSet<String>,
    generic_hide: bool,
) -> Result<jobject> {
    let mut new_hash_set = |items: &HashSet<String>| -> Result<JObject> {
        let hash_set_class = match env.find_class("java/util/HashSet") {
            Ok(c) => {
                if c.is_null() {
                    log::error!("HashSet class found but is null");
                    return Err(RustException::ParseJavaObject(
                        "HashSet class is null".into(),
                    ));
                }
                c
            }
            Err(e) => {
                log::error!("Failed to find HashSet class: {:?}", e);
                return Err(RustException::ParseJavaObject(format!(
                    "Failed to find HashSet class: {:?}",
                    e
                )));
            }
        };

        let set = match env.new_object(hash_set_class, "()V", &[]) {
            Ok(s) => {
                if s.is_null() {
                    log::error!("HashSet instance created but is null");
                    return Err(RustException::ParseJavaObject(
                        "HashSet instance is null".into(),
                    ));
                }
                s
            }
            Err(e) => {
                log::error!("Failed to create HashSet instance: {:?}", e);
                return Err(RustException::ParseJavaObject(format!(
                    "Failed to create HashSet instance: {:?}",
                    e
                )));
            }
        };

        for item in items.iter() {
            match env.new_string(item) {
                Ok(jstr) => {
                    let jobj = JObject::from(jstr);
                    match env.call_method(&set, "add", "(Ljava/lang/Object;)Z", &[(&jobj).into()]) {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(RustException::ParseJavaObject(format!(
                                "Failed to add item to HashSet: {:?}",
                                e
                            )));
                        }
                    }
                }
                Err(e) => {
                    return Err(RustException::ParseJavaObject(format!(
                        "Failed to create Java string: {:?}",
                        e
                    )));
                }
            }
        }

        Ok(set)
    };

    let hide_set = new_hash_set(&hide)?;
    let exceptions_set = new_hash_set(&exceptions)?;

    let j_script: JString = env.new_string(script)?;

    let class: JClass = if let Some(caller_obj) = caller {
        let caller_cls = env.get_object_class(caller_obj)?;
        let loader_val = env.call_method(
            caller_cls,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )?;
        let loader_obj = loader_val.l()?;
        let class_name = env.new_string("eu.byquanton.adblock.CosmeticResources")?;
        let loaded = env.call_method(
            loader_obj,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[(&class_name).into()],
        )?;
        let class_obj = loaded.l()?;
        JClass::from(class_obj)
    } else {
        match env.find_class("eu/byquanton/adblock/CosmeticResources") {
            Ok(c) => c,
            Err(e) => {
                return Err(RustException::ParseJavaObject(format!(
                    "Failed to find CosmeticResources class: {:?}",
                    e
                )));
            }
        }
    };

    let obj = env.new_object(
        class,
        "(Ljava/util/Set;Ljava/lang/String;Ljava/util/Set;Z)V",
        &[
            (&hide_set).into(),
            (&JObject::from(j_script)).into(),
            (&exceptions_set).into(),
            jboolean::from(generic_hide).into(),
        ],
    )?;

    if obj.is_null() {
        return Err(RustException::ParseJavaObject(
            "Failed to create CosmeticResources instance".into(),
        ));
    }

    let global = env.new_global_ref(obj)?;
    {
        let mut vec = RETURNED_GLOBAL_REFS.lock()?;
        vec.push(global);
        let stored = vec.last().expect("");
        Ok((**stored.as_obj()).into())
    }
}
