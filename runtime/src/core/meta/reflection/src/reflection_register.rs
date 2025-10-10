use linkme::distributed_slice;

use crate::reflection::TypeMetaRegisterInterface;

#[distributed_slice]
pub static REFLECT_REGISTER_FUNCTION_LIST: [fn()];
pub fn meta_register() {
    for register_function in REFLECT_REGISTER_FUNCTION_LIST {
        register_function();
    }
} 

pub fn meta_unregister() {
    TypeMetaRegisterInterface::unregister_all();
}