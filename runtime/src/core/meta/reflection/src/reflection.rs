use std::{cell::LazyCell, collections::HashMap, os::raw::c_void};

pub type SetFunction = fn(*mut c_void, *const c_void);
pub type GetFunction = fn(*const c_void) -> *const c_void;
pub type GetNameFunction = fn() -> &'static str;
pub type GetBoolFunction = fn(*const c_void) -> bool;


pub type FieldFunctionTuple = (SetFunction, GetFunction, GetNameFunction, GetNameFunction, GetNameFunction, GetBoolFunction);
pub type MethodFunctionTuple = ();
pub type ClassFunctionTuple = ();
pub type ArrayFunctionTuple = ();

static mut M_CLASS_MAP : LazyCell<HashMap<&'static str, ClassFunctionTuple>> = LazyCell::new(|| HashMap::new());
static mut M_FIELD_MAP : LazyCell<HashMap<&'static str, Vec<FieldFunctionTuple>>> = LazyCell::new(|| HashMap::new());

pub struct TypeMetaRegisterInterface;

impl TypeMetaRegisterInterface {

    #[allow(static_mut_refs)]
    pub fn register_to_field_map(name: &'static str, field_function_tuple: FieldFunctionTuple) {
        unsafe{
            M_FIELD_MAP.entry(name).or_default().push(field_function_tuple);
        }
    }

    #[allow(static_mut_refs)]
    pub fn register_to_class_map(name: &'static str, class_function_tuple: ClassFunctionTuple) {
        unsafe{
            if !M_CLASS_MAP.contains_key(name) {
                M_CLASS_MAP.insert(name, class_function_tuple);
            }
            else{
                M_CLASS_MAP.remove(name);
            }
            
        }
    }

    #[allow(static_mut_refs)]
    pub fn unregister_all() {
        unsafe {
            M_CLASS_MAP.clear();
            M_FIELD_MAP.clear();
        }
    }
}

#[derive(Default)]
pub struct TypeMeta {
    m_fileds: Vec<FieldAccessor>,
    m_methods: Vec<MethodAccessor>,
    m_type_name: String,
    m_is_valid: bool,
}

impl TypeMeta {
    
}

struct FieldAccessor {
    m_functions: FieldFunctionTuple,
    m_field_name: &'static str,
    m_field_type: &'static str,
} 

impl FieldAccessor {
    fn get(){

    }

    fn set(){

    }

    fn get_owner_type_meta(){

    }

    fn get_type_meta(){

    }

    fn get_field_name(){

    }

    fn get_field_type_name(){

    }

    fn is_array_type(){

    }
}

struct MethodAccessor {
    m_functions: MethodFunctionTuple,
    m_method_name: &'static str,
}

struct ArrayAccessor {
    m_function: ArrayFunctionTuple,
    m_array_type_name: &'static str,
    m_element_type_name: &'static str,
}


#[derive(Default)]
pub struct ReflectionInstance{
    pub m_meta: TypeMeta,
    pub m_instance: *mut c_void,
}

impl ReflectionInstance {
    pub fn new(meta: TypeMeta, instance: *mut c_void) -> Self {
        Self {
            m_meta: meta,
            m_instance: instance,
        }
    }
}

#[derive(Default, Clone)]
pub struct ReflectionPtr<T> {
    m_type_name: &'static str,
    m_instance: *mut T,
}

impl<T> ReflectionPtr<T> {
    pub fn new(type_name: &'static str, instance: *mut T) -> Self {
        Self {
            m_type_name: type_name,
            m_instance: instance,
        }
    }

    pub fn cast<U>(&self) -> ReflectionPtr<U> {
        ReflectionPtr {
            m_type_name: self.m_type_name.clone(),
            m_instance: self.m_instance as *mut U,
        }
    }
}