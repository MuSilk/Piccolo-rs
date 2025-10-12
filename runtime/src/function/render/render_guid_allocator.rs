use std::{collections::HashMap, hash::Hash};


const S_INVALID_GUID: usize = 0;

#[derive(Default)]
pub struct GuidAllocator<T: Eq + Hash + Clone + Default> {
    m_elements_guid_map: HashMap<T, usize>,
    m_guid_elements_map: HashMap<usize, T>,
}

impl<T: Eq + Hash + Clone + Default> GuidAllocator<T>  {
    pub fn is_valid_guid(guid: usize) -> bool {
        guid != S_INVALID_GUID
    }

    pub fn alloc_guid(&mut self, element: &T) -> usize {
        if self.m_elements_guid_map.contains_key(element) {
            return self.m_elements_guid_map[element];
        }
        for i in 0..self.m_guid_elements_map.len() + 1 {
            let guid = i + 1;
            if !self.m_guid_elements_map.contains_key(&guid) {
                self.m_guid_elements_map.insert(guid, element.clone());
                self.m_elements_guid_map.insert(element.clone(), guid);
                return guid;
            }
        }
        S_INVALID_GUID
    }

    pub fn get_guid_related_element(&self, guid: usize) -> Option<&T> {
        self.m_guid_elements_map.get(&guid)
    }

    pub fn get_element_guid(&self, element: &T) -> Option<usize> {
        self.m_elements_guid_map.get(element).cloned()
    }

    pub fn has_element(&self, element: &T) -> bool {
        self.m_elements_guid_map.contains_key(element)
    }

    pub fn free_guid(&mut self, guid: usize) {
        if let Some(element) = self.m_guid_elements_map.remove(&guid) {
            self.m_elements_guid_map.remove(&element);
        }
    }

    pub fn free_element(&mut self, element: &T) {
        if let Some(guid) = self.m_elements_guid_map.remove(element) {
            self.m_guid_elements_map.remove(&guid);
        }
    }

    pub fn get_allocated_guids(&self) -> Vec<usize> {
        self.m_guid_elements_map.keys().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.m_elements_guid_map.clear();
        self.m_guid_elements_map.clear();
    }
}