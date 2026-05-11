#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]

use crate::Result;
use crate::ast::Value;
use crate::de;
use crate::ser;
use std::alloc::{Layout, alloc, dealloc};
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::slice;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum CValueKind {
    Null = 0,
    Bool = 1,
    Number = 2,
    String = 3,
    Array = 4,
    Map = 5,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CValue {
    pub kind: CValueKind,
    pub data: CValueData,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union CValueData {
    pub bool_val: bool,
    pub number_val: f64,
    pub string_val: *mut c_char,
    pub array_val: *mut CValueArray,
    pub map_val: *mut CValueMap,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CValueArray {
    pub len: usize,
    pub data: *mut CValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CValueMap {
    pub len: usize,
    pub entries: *mut CValueMapEntry,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CValueMapEntry {
    pub key: *mut c_char,
    pub value: CValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CResult {
    pub error_code: i32,
    pub payload: *mut c_char,
}

impl From<Result<String>> for CResult {
    fn from(res: Result<String>) -> Self {
        match res {
            Ok(s) => {
                let c_str = CString::new(s).unwrap();
                let ptr = c_str.into_raw();
                CResult {
                    error_code: 0,
                    payload: ptr,
                }
            }
            Err(e) => CResult {
                error_code: 1,
                payload: CString::new(e.to_string()).unwrap().into_raw(),
            },
        }
    }
}

fn allocate_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

fn deallocate_string(s: *mut c_char) {
    unsafe {
        let _ = CString::from_raw(s);
    }
}

fn write_cvalue_in_place(ptr: *mut CValue, value: Value) {
    unsafe {
        match value {
            Value::Bool(b) => {
                (*ptr).kind = CValueKind::Bool;
                (*ptr).data.bool_val = b;
            }
            Value::Number(n) => {
                (*ptr).kind = CValueKind::Number;
                (*ptr).data.number_val = n;
            }
            Value::String(s) => {
                (*ptr).kind = CValueKind::String;
                (*ptr).data.string_val = allocate_string(&s);
            }
            Value::Array(arr) => {
                let array_ptr = allocate_array(arr.len());
                for (i, item) in arr.into_iter().enumerate() {
                    write_cvalue_in_place((*array_ptr).data.add(i), item);
                }
                (*ptr).kind = CValueKind::Array;
                (*ptr).data.array_val = array_ptr;
            }
            Value::Map(map) => {
                let map_ptr = allocate_map(map.len());
                for (i, (key, val)) in map.into_iter().enumerate() {
                    let entry_ptr = (*map_ptr).entries.add(i);
                    ptr::write(&mut (*entry_ptr).key, allocate_string(&key));
                    write_cvalue_in_place(&mut (*entry_ptr).value, val);
                }
                (*ptr).kind = CValueKind::Map;
                (*ptr).data.map_val = map_ptr;
            }
        }
    }
}

fn value_to_cvalue(value: Value) -> *mut CValue {
    unsafe {
        let layout = Layout::new::<CValue>();
        let ptr = alloc(layout) as *mut CValue;
        write_cvalue_in_place(ptr, value);
        ptr
    }
}

fn allocate_array(len: usize) -> *mut CValueArray {
    unsafe {
        let data_size = len * size_of::<CValue>();
        let total_size = size_of::<CValueArray>() + data_size;
        let layout = Layout::from_size_align(total_size, align_of::<CValueArray>()).unwrap();
        let ptr = alloc(layout) as *mut CValueArray;

        ptr::write(&mut (*ptr).len, len);
        let data_ptr = (ptr as *mut u8).add(size_of::<CValueArray>()) as *mut CValue;
        ptr::write(&mut (*ptr).data, data_ptr);

        for i in 0..len {
            let item_ptr = data_ptr.add(i);
            ptr::write(
                item_ptr,
                CValue {
                    kind: CValueKind::Null,
                    data: CValueData {
                        string_val: ptr::null_mut(),
                    },
                },
            );
        }

        ptr
    }
}

fn allocate_map(len: usize) -> *mut CValueMap {
    unsafe {
        let entry_size = size_of::<CValueMapEntry>();
        let data_size = len * entry_size;
        let total_size = size_of::<CValueMap>() + data_size;
        let layout = Layout::from_size_align(total_size, align_of::<CValueMap>()).unwrap();
        let ptr = alloc(layout) as *mut CValueMap;

        ptr::write(&mut (*ptr).len, len);
        let entries_ptr = (ptr as *mut u8).add(size_of::<CValueMap>()) as *mut CValueMapEntry;
        ptr::write(&mut (*ptr).entries, entries_ptr);

        for i in 0..len {
            let entry_ptr = entries_ptr.add(i);
            ptr::write(&mut (*entry_ptr).key, ptr::null_mut());
            ptr::write(
                &mut (*entry_ptr).value,
                CValue {
                    kind: CValueKind::Null,
                    data: CValueData {
                        string_val: ptr::null_mut(),
                    },
                },
            );
        }

        ptr
    }
}

fn cvalue_to_value(ptr: *const CValue) -> Value {
    unsafe {
        match (*ptr).kind {
            CValueKind::Bool => Value::Bool((*ptr).data.bool_val),
            CValueKind::Number => Value::Number((*ptr).data.number_val),
            CValueKind::String => {
                let c_str = (*ptr).data.string_val;
                let len = strlen(c_str);
                let slice = slice::from_raw_parts(c_str as *const u8, len);
                Value::String(String::from_utf8_unchecked(slice.to_vec()))
            }
            CValueKind::Array => {
                let array_ptr = (*ptr).data.array_val;
                let len = (*array_ptr).len;
                let mut vec = thin_vec::ThinVec::with_capacity(len);
                for i in 0..len {
                    let item_ptr = (*array_ptr).data.add(i);
                    vec.push(cvalue_to_value(item_ptr));
                }
                Value::Array(vec)
            }
            CValueKind::Map => {
                let map_ptr = (*ptr).data.map_val;
                let len = (*map_ptr).len;
                let mut vec = thin_vec::ThinVec::with_capacity(len);
                for i in 0..len {
                    let entry = &*(*map_ptr).entries.add(i);
                    let key_len = strlen(entry.key);
                    let key_slice = slice::from_raw_parts(entry.key as *const u8, key_len);
                    let key = Box::from(String::from_utf8_unchecked(key_slice.to_vec()));
                    let value = cvalue_to_value(&entry.value);
                    vec.push((key, value));
                }
                Value::Map(vec)
            }
            CValueKind::Null => Value::String("null".to_string()),
        }
    }
}

fn strlen(s: *const c_char) -> usize {
    unsafe {
        let mut len = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        len
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_parse(input: *const c_char) -> *mut CResult {
    if input.is_null() {
        let result = Box::new(CResult {
            error_code: 1,
            payload: CString::new("null input").unwrap().into_raw(),
        });
        return Box::into_raw(result);
    }

    let input_str = unsafe {
        let len = strlen(input);
        let slice = slice::from_raw_parts(input as *const u8, len);
        String::from_utf8_unchecked(slice.to_vec())
    };

    match de::from_str::<Value>(&input_str) {
        Ok(value) => {
            let value_ptr = value_to_cvalue(value);
            let result = Box::new(CResult {
                error_code: 0,
                payload: value_ptr as *mut c_char,
            });
            Box::into_raw(result)
        }
        Err(e) => {
            let result = Box::new(CResult {
                error_code: 1,
                payload: CString::new(e.to_string()).unwrap().into_raw(),
            });
            Box::into_raw(result)
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_serialize(value: *const CValue) -> CResult {
    if value.is_null() {
        return CResult {
            error_code: 1,
            payload: CString::new("null value").unwrap().into_raw(),
        };
    }

    let rust_value = cvalue_to_value(value);
    ser::to_string(&rust_value).into()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_bool(ptr: *const CValue) -> bool {
    (*ptr).data.bool_val
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_number(ptr: *const CValue) -> f64 {
    (*ptr).data.number_val
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_string(ptr: *const CValue) -> *const c_char {
    (*ptr).data.string_val
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_array(ptr: *const CValue) -> *const CValueArray {
    (*ptr).data.string_val as *const CValueArray
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_map(ptr: *const CValue) -> *const CValueMap {
    (*ptr).data.string_val as *const CValueMap
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_kind(ptr: *const CValue) -> CValueKind {
    (*ptr).kind
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_array_len(arr: *const CValueArray) -> usize {
    (*arr).len
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_array_get(arr: *const CValueArray, index: usize) -> *const CValue {
    (*arr).data.add(index)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_map_len(map: *const CValueMap) -> usize {
    (*map).len
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_map_get(
    map: *const CValueMap,
    index: usize,
) -> *const CValueMapEntry {
    &*(*map).entries.add(index)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_map_entry_key(entry: *const CValueMapEntry) -> *const c_char {
    (*entry).key
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_map_entry_value(entry: *const CValueMapEntry) -> *const CValue {
    &(*entry).value
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_result_error_message(res: *const CResult) -> *const c_char {
    (*res).payload
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_result_value(res: *const CResult) -> *const CValue {
    if (*res).error_code != 0 {
        return std::ptr::null();
    }
    (*res).payload as *const CValue
}

fn free_cvalue_contents(ptr: *mut CValue) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        match (*ptr).kind {
            CValueKind::String => {
                deallocate_string((*ptr).data.string_val);
            }
            CValueKind::Array => {
                let array_ptr = (*ptr).data.array_val;
                let len = (*array_ptr).len;
                let data_ptr = (*array_ptr).data;
                for i in 0..len {
                    free_cvalue_contents(data_ptr.add(i));
                }
                let layout = Layout::from_size_align(
                    size_of::<CValueArray>() + len * size_of::<CValue>(),
                    align_of::<CValueArray>(),
                )
                .unwrap();
                dealloc(array_ptr as *mut u8, layout);
            }
            CValueKind::Map => {
                let map_ptr = (*ptr).data.map_val;
                let len = (*map_ptr).len;
                let entries_ptr = (*map_ptr).entries;
                for i in 0..len {
                    let entry_ptr = entries_ptr.add(i);
                    deallocate_string((*entry_ptr).key);
                    free_cvalue_contents(&mut (*entry_ptr).value);
                }
                let layout = Layout::from_size_align(
                    size_of::<CValueMap>() + len * size_of::<CValueMapEntry>(),
                    align_of::<CValueMap>(),
                )
                .unwrap();
                dealloc(map_ptr as *mut u8, layout);
            }
            _ => {}
        }
    }
}

fn free_cvalue(ptr: *mut CValue) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        free_cvalue_contents(ptr);
        dealloc(ptr as *mut u8, Layout::new::<CValue>());
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_result_free(res: *mut CResult) {
    if !res.is_null() {
        let error_code = (*res).error_code;
        if error_code == 0 {
            let value_ptr = (*res).payload as *mut CValue;
            free_cvalue(value_ptr);
        } else {
            deallocate_string((*res).payload);
        }
        let _ = Box::from_raw(res);
    }
}
