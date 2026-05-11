#![allow(unsafe_op_in_unsafe_fn)]

use crate::ast::Value;
use crate::de;
use crate::ser;
use crate::{Error, Result};
use std::alloc::{Layout, alloc, dealloc};
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

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
pub enum CResultKind {
    ParseSuccess = 0,
    SerializeSuccess = 1,
    Error = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union CResultPayload {
    pub value: *mut CValue,
    pub serialized: *mut c_char,
    pub error_message: *mut c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CResult {
    pub error_code: i32,
    pub kind: CResultKind,
    pub payload: CResultPayload,
}

impl CResult {
    fn from_ser(res: Result<String>) -> Self {
        match res {
            Ok(s) => Self {
                error_code: 0,
                kind: CResultKind::SerializeSuccess,
                payload: CResultPayload {
                    serialized: allocate_string(&s),
                },
            },
            Err(e) => Self::error(e),
        }
    }

    fn from_des(res: Result<Value>) -> Self {
        match res {
            Ok(value) => match value_to_cvalue(value) {
                Ok(ptr) => Self {
                    error_code: 0,
                    kind: CResultKind::ParseSuccess,
                    payload: CResultPayload { value: ptr },
                },
                Err(e) => Self::error(e),
            },
            Err(e) => Self::error(e),
        }
    }

    #[inline]
    fn error(e: Error) -> Self {
        Self {
            error_code: 1,
            kind: CResultKind::Error,
            payload: CResultPayload {
                error_message: allocate_string(&e.to_string()),
            },
        }
    }

    #[inline]
    fn null_input() -> Self {
        Self {
            error_code: 1,
            kind: CResultKind::Error,
            payload: CResultPayload {
                error_message: allocate_string("null input"),
            },
        }
    }
}

#[inline]
fn allocate_string(s: &str) -> *mut c_char {
    // Strip interior NULs to avoid panicking across the FFI boundary.
    let mut bytes = s.as_bytes().to_vec();
    bytes.retain(|&b| b != 0);
    unsafe { CString::from_vec_unchecked(bytes).into_raw() }
}

fn deallocate_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(s);
    }
}

fn array_layout(len: usize) -> Result<(Layout, usize)> {
    let (layout, offset) = Layout::new::<CValueArray>()
        .extend(Layout::array::<CValue>(len).map_err(|e| Error::CApi(e.to_string()))?)
        .map_err(|e| Error::CApi(e.to_string()))?;
    Ok((layout.pad_to_align(), offset))
}

fn map_layout(len: usize) -> Result<(Layout, usize)> {
    let (layout, offset) = Layout::new::<CValueMap>()
        .extend(Layout::array::<CValueMapEntry>(len).map_err(|e| Error::CApi(e.to_string()))?)
        .map_err(|e| Error::CApi(e.to_string()))?;
    Ok((layout.pad_to_align(), offset))
}

fn write_cvalue_in_place(ptr: *mut CValue, value: Value) -> Result<()> {
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
                let array_ptr = allocate_array(arr.len())?;
                (*ptr).kind = CValueKind::Array;
                (*ptr).data.array_val = array_ptr;

                for (i, item) in arr.into_iter().enumerate() {
                    write_cvalue_in_place((*array_ptr).data.add(i), item)?;
                }
            }
            Value::Map(map) => {
                let map_ptr = allocate_map(map.len())?;
                (*ptr).kind = CValueKind::Map;
                (*ptr).data.map_val = map_ptr;

                for (i, (key, val)) in map.into_iter().enumerate() {
                    let entry_ptr = (*map_ptr).entries.add(i);
                    ptr::write(&mut (*entry_ptr).key, allocate_string(&key));
                    write_cvalue_in_place(&mut (*entry_ptr).value, val)?;
                }
            }
        }
    }
    Ok(())
}

fn value_to_cvalue(value: Value) -> Result<*mut CValue> {
    unsafe {
        let layout = Layout::new::<CValue>();
        let ptr = alloc(layout) as *mut CValue;
        if ptr.is_null() {
            return Err(Error::CApi("out of memory".to_string()));
        }
        if let Err(e) = write_cvalue_in_place(ptr, value) {
            free_cvalue(ptr);
            return Err(e);
        }
        Ok(ptr)
    }
}

fn allocate_array(len: usize) -> Result<*mut CValueArray> {
    unsafe {
        let (layout, offset) = array_layout(len)?;
        let ptr = alloc(layout) as *mut CValueArray;
        if ptr.is_null() {
            return Err(Error::CApi("out of memory".to_string()));
        }

        ptr::write(&mut (*ptr).len, len);
        let data_ptr = (ptr as *mut u8).add(offset) as *mut CValue;
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

        Ok(ptr)
    }
}

fn allocate_map(len: usize) -> Result<*mut CValueMap> {
    unsafe {
        let (layout, offset) = map_layout(len)?;
        let ptr = alloc(layout) as *mut CValueMap;
        if ptr.is_null() {
            return Err(Error::CApi("out of memory".to_string()));
        }

        ptr::write(&mut (*ptr).len, len);
        let entries_ptr = (ptr as *mut u8).add(offset) as *mut CValueMapEntry;
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

        Ok(ptr)
    }
}

fn cvalue_to_value(ptr: *const CValue) -> Result<Value> {
    unsafe {
        Ok(match (*ptr).kind {
            CValueKind::Bool => Value::Bool((*ptr).data.bool_val),
            CValueKind::Number => Value::Number((*ptr).data.number_val),
            CValueKind::String => {
                let c_str = (*ptr).data.string_val;
                Value::String(CStr::from_ptr(c_str).to_string_lossy().to_string())
            }
            CValueKind::Array => {
                let array_ptr = (*ptr).data.array_val;
                let len = (*array_ptr).len;
                let mut vec = thin_vec::ThinVec::with_capacity(len);
                for i in 0..len {
                    let item_ptr = (*array_ptr).data.add(i);
                    vec.push(cvalue_to_value(item_ptr)?);
                }
                Value::Array(vec)
            }
            CValueKind::Map => {
                let map_ptr = (*ptr).data.map_val;
                let len = (*map_ptr).len;
                let mut vec = thin_vec::ThinVec::with_capacity(len);
                for i in 0..len {
                    let entry = &*(*map_ptr).entries.add(i);
                    let key = Box::from(CStr::from_ptr(entry.key).to_string_lossy().to_string());
                    let value = cvalue_to_value(&entry.value)?;
                    vec.push((key, value));
                }
                Value::Map(vec)
            }
            CValueKind::Null => {
                return Err(Error::CApi("Serializing null is not supported".to_string()));
            }
        })
    }
}

/// # Safety
///
/// - `input` may be null (returns an error result). If non-null, it must point to a valid
///   null-terminated C string.
/// - The caller owns the returned `CResult` and must free it via [`glass_result_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_parse(input: *const c_char) -> *mut CResult {
    if input.is_null() {
        return Box::into_raw(Box::new(CResult::null_input()));
    }

    let input_str = unsafe {
        let slice = CStr::from_ptr(input);
        slice.to_string_lossy().to_string()
    };

    Box::into_raw(Box::new(CResult::from_des(de::from_str(&input_str))))
}

/// # Safety
///
/// - `value` may be null (returns an error result). If non-null, it must point to a valid,
///   properly initialized `CValue`.
/// - The caller owns the returned `CResult` and must free it via [`glass_result_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_serialize(value: *const CValue) -> *mut CResult {
    if value.is_null() {
        return Box::into_raw(Box::new(CResult::null_input()));
    }

    let cvalue = match cvalue_to_value(value) {
        Ok(cvalue) => cvalue,
        Err(e) => return Box::into_raw(Box::new(CResult::error(e))),
    };

    Box::into_raw(Box::new(CResult::from_ser(ser::to_string(&cvalue))))
}

/// # Safety
///
/// `ptr` must be non-null and point to a valid `CValue` whose `kind` is `Bool`. Returns false if
/// ptr is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_bool(ptr: *const CValue) -> bool {
    if ptr.is_null() {
        return false;
    }
    (*ptr).data.bool_val
}

/// # Safety
///
/// `ptr` must be non-null and point to a valid `CValue` whose `kind` is `Number`. Returns f64::MAX
/// if ptr is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_number(ptr: *const CValue) -> f64 {
    if ptr.is_null() {
        return f64::MAX;
    }
    (*ptr).data.number_val
}

/// # Safety
///
/// `ptr` must be non-null and point to a valid `CValue` whose `kind` is `String`. The returned
/// pointer is valid until the owning [`CResult`] is freed via [`glass_result_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_string(ptr: *const CValue) -> *const c_char {
    if ptr.is_null() {
        return std::ptr::null();
    }
    (*ptr).data.string_val
}

/// # Safety
///
/// `ptr` must be non-null and point to a valid `CValue` whose `kind` is `Array`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_array(ptr: *const CValue) -> *const CValueArray {
    if ptr.is_null() {
        return std::ptr::null();
    }
    (*ptr).data.array_val
}

/// # Safety
///
/// `ptr` must be non-null and point to a valid `CValue` whose `kind` is `Map`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_map(ptr: *const CValue) -> *const CValueMap {
    if ptr.is_null() {
        return std::ptr::null();
    }
    (*ptr).data.map_val
}

/// # Safety
///
/// `ptr` must be non-null and point to a valid `CValue`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_value_get_kind(ptr: *const CValue) -> CValueKind {
    if ptr.is_null() {
        return CValueKind::Null;
    }
    (*ptr).kind
}

/// # Safety
///
/// `arr` must be non-null and point to a valid `CValueArray`. Returns usize::MAX if arr is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_array_len(arr: *const CValueArray) -> usize {
    if arr.is_null() {
        return usize::MAX;
    }
    (*arr).len
}

/// # Safety
///
/// `arr` must be non-null and point to a valid `CValueArray`. `index` must be less than `arr.len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_array_get(arr: *const CValueArray, index: usize) -> *const CValue {
    if arr.is_null() || index >= (*arr).len {
        return std::ptr::null();
    }
    (*arr).data.add(index)
}

/// # Safety
///
/// `map` must be non-null and point to a valid `CValueMap`. Returns usize::MAX if map is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_map_len(map: *const CValueMap) -> usize {
    if map.is_null() {
        return usize::MAX;
    }
    (*map).len
}

/// # Safety
///
/// `map` must be non-null and point to a valid `CValueMap`. `index` must be less than `map.len`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_map_get(
    map: *const CValueMap,
    index: usize,
) -> *const CValueMapEntry {
    if map.is_null() || index >= (*map).len {
        return std::ptr::null();
    }
    &*(*map).entries.add(index)
}

/// # Safety
///
/// `entry` must be non-null and point to a valid `CValueMapEntry`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_map_entry_key(entry: *const CValueMapEntry) -> *const c_char {
    if entry.is_null() {
        return std::ptr::null();
    }
    (*entry).key
}

/// # Safety
///
/// `entry` must be non-null and point to a valid `CValueMapEntry`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_map_entry_value(entry: *const CValueMapEntry) -> *const CValue {
    if entry.is_null() {
        return std::ptr::null();
    }
    &(*entry).value
}

/// # Safety
///
/// `res` must be non-null and point to a valid `CResult` whose `kind` is `Error`. The returned
/// pointer is valid until the result is freed via [`glass_result_free`]. Returns null if res is
/// null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_result_error_message(res: *const CResult) -> *const c_char {
    if res.is_null() || !matches!((*res).kind, CResultKind::Error) {
        return std::ptr::null();
    }
    (*res).payload.error_message
}

/// # Safety
///
/// `res` must be non-null and point to a valid `CResult`. When `kind` is `ParseSuccess`, the
/// returned pointer is valid until the result is freed via [`glass_result_free`]; otherwise
/// returns null. Returns null if res is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_result_value(res: *const CResult) -> *const CValue {
    if res.is_null() || !matches!((*res).kind, CResultKind::ParseSuccess) {
        return std::ptr::null();
    }
    (*res).payload.value as *const CValue
}

/// # Safety
///
/// `res` must be non-null and point to a valid `CResult` whose `kind` is `SerializeSuccess`. The returned
/// pointer is valid until the result is freed via [`glass_result_free`]. Returns null if res is
/// null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_result_serialized(res: *const CResult) -> *const c_char {
    if res.is_null() || !matches!((*res).kind, CResultKind::SerializeSuccess) {
        return std::ptr::null();
    }
    (*res).payload.serialized
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
                if array_ptr.is_null() {
                    return;
                }
                let len = (*array_ptr).len;
                let data_ptr = (*array_ptr).data;
                for i in 0..len {
                    free_cvalue_contents(data_ptr.add(i));
                }
                let (layout, _) = array_layout(len).expect("consistent layout");
                dealloc(array_ptr as *mut u8, layout);
            }
            CValueKind::Map => {
                let map_ptr = (*ptr).data.map_val;
                if map_ptr.is_null() {
                    return;
                }
                let len = (*map_ptr).len;
                let entries_ptr = (*map_ptr).entries;
                for i in 0..len {
                    let entry_ptr = entries_ptr.add(i);
                    deallocate_string((*entry_ptr).key);
                    free_cvalue_contents(&mut (*entry_ptr).value);
                }
                let (layout, _) = map_layout(len).expect("consistent layout");
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

/// # Safety
///
/// `res` must be non-null and point to a valid `CResult`. Returns CResultKind::Error if res is
/// null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_result_get_kind(res: *const CResult) -> CResultKind {
    if res.is_null() {
        return CResultKind::Error;
    }
    (*res).kind
}

/// # Safety
///
/// - `res` may be null (no-op). If non-null, it must point to a `CResult` previously returned by
///   [`glass_parse`] or [`glass_serialize`] and not yet freed. After calling this function the
///   pointer is invalidated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn glass_result_free(res: *mut CResult) {
    if !res.is_null() {
        match (*res).kind {
            CResultKind::ParseSuccess => {
                free_cvalue((*res).payload.value);
            }
            CResultKind::SerializeSuccess => {
                deallocate_string((*res).payload.serialized);
            }
            CResultKind::Error => {
                deallocate_string((*res).payload.error_message);
            }
        }
        let _ = Box::from_raw(res);
    }
}
