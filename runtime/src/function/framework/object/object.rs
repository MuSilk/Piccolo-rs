use crate::function::framework::object::object_id_allocator::GObjectID;

pub struct GObject {
    pub m_id: GObjectID,
    pub m_name: String,
    pub m_definition_url: String,
}