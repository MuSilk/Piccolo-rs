use runtime::function::framework::resource::{component::mesh::MeshComponentRes, resource::Resource};
use serde::{Deserialize, Serialize};


#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BlockRes {
    pub m_mesh_res: MeshComponentRes,
}

#[typetag::serde]
impl Resource for BlockRes {

}