use log::error;

pub type GObjectID = usize;

pub const K_INVALID_GOBJECT_ID: GObjectID = GObjectID::MAX;

pub static mut M_NEXT_ID: GObjectID = 0;

pub fn alloc() -> GObjectID {
    unsafe { 
        let new_object_ret = M_NEXT_ID;
        M_NEXT_ID += 1;
        if M_NEXT_ID >= K_INVALID_GOBJECT_ID {
            error!("gobject id overflow");
            M_NEXT_ID = 0;
        }
        new_object_ret
    }
}

