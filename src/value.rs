use std::any::Any;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::str;

use objc::{Encode, Encoding};
use objc_id::Id;

use {INSCopying, INSObject};

pub trait INSValue : INSObject {
    type Value: 'static + Copy + Encode;

    fn value(&self) -> Self::Value {
        assert!(Self::Value::encode() == self.encoding());
        unsafe {
            let mut value = mem::uninitialized::<Self::Value>();
            let value_ptr: *mut Self::Value = &mut value;
            let bytes = value_ptr as *mut c_void;
            let _: () = msg_send![self, getValue:bytes];
            value
        }
    }

    fn encoding(&self) -> Encoding {
        unsafe {
            let result: *const c_char = msg_send![self, objCType];
            let s = CStr::from_ptr(result);
            let s = str::from_utf8(s.to_bytes()).unwrap();
            Encoding::from_str(s)
        }
    }

    fn from_value(value: Self::Value) -> Id<Self> {
        let cls = Self::class();
        let value_ptr: *const Self::Value = &value;
        let bytes = value_ptr as *const c_void;
        let encoding = CString::new(Self::Value::encode().as_str()).unwrap();
        unsafe {
            let obj: *mut Self = msg_send![cls, alloc];
            let obj: *mut Self = msg_send![obj, initWithBytes:bytes
                                                     objCType:encoding.as_ptr()];
            Id::from_retained_ptr(obj)
        }
    }
}

#[derive(INSObject)]
pub struct NSValue<T> where T: Any {
    value: PhantomData<T>,
}

impl<T> INSValue for NSValue<T> where T: Any + Copy + Encode {
    type Value = T;
}

impl<T> INSCopying for NSValue<T> where T: Any {
    type Output = NSValue<T>;
}

#[cfg(test)]
mod tests {
    use objc::Encode;
    use {INSValue, NSValue};

    #[test]
    fn test_value() {
        let val = NSValue::from_value(13u32);
        assert!(val.value() == 13);
        assert!(u32::encode() == val.encoding());
    }
}
