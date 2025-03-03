use std::{convert::TryFrom, mem::size_of};

use super::{error::Error, node_type::NodeType};
use crate::{
    datas::VecBytes,
    read_int_ptr,
    rocksdb_impl::bptree::{children::Children, db_key::DbKey, kits::new_db_key, leaf_data::LeafData, node_type::Keys},
    BytesType, LenType,
};

/// Node represents a node in the BTree occupied by a single page in memory.
#[derive(Clone, Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub data: Vec<u8>,
}

// Node represents a node in the B-Tree.
impl Node {
    /// Common Node header layout (Ten bytes in total)
    pub const OFFSET_NODE_TYPE: isize = 0;
    pub const OFFSET_DB_KEY: isize = Node::OFFSET_NODE_TYPE + size_of::<u8>() as isize;
    pub const OFFSET_PARENT_DB_KEY: isize = Node::OFFSET_DB_KEY + DbKey::LEN_DB_KEY as isize;
    pub const OFFSET_NODE_DATA: isize = Node::OFFSET_PARENT_DB_KEY + DbKey::LEN_DB_KEY as isize;

    pub fn new(node_type: NodeType) -> Node {
        let mut data = Vec::with_capacity(Node::OFFSET_NODE_DATA as usize);
        data.resize(data.capacity(), 0);
        data[0] = u8::from(&node_type);
        let mut node = Node { node_type, data };
        node.set_node_type(&[]);
        node.make_db_key();
        node
    }

    pub fn set_node_type(&mut self, data: &[u8]) {
        let node_type = &self.node_type;
        match node_type {
            NodeType::None => {
                unsafe {
                    self.data.set_len(Node::OFFSET_NODE_DATA as usize);
                }
                let t = [0 as u8; Node::OFFSET_NODE_DATA as usize];
                self.data.extend_from_slice(&t);
            }
            NodeType::Internal(_children, _keys) => {
                if data.is_empty() {
                    unsafe {
                        self.data.set_len(Node::OFFSET_NODE_DATA as usize);
                    }
                    let t = [0 as u8; Node::OFFSET_NODE_DATA as usize];
                    self.data.extend_from_slice(&t);
                } else {
                    self.data.reserve(data.len() - self.data.len());
                    unsafe {
                        self.data.set_len(data.len());
                    }
                    self.data.copy_from_slice(data);
                }
            }
            NodeType::Leaf(_leaf) => {
                if data.is_empty() {
                    unsafe {
                        self.data.set_len(Node::OFFSET_NODE_DATA as usize);
                    }
                    let t = [0 as u8; Node::OFFSET_NODE_DATA as usize];
                    self.data.extend_from_slice(&t);
                } else {
                    self.data.reserve(data.len() - self.data.len());
                    unsafe {
                        self.data.set_len(data.len());
                    }
                    self.data.copy_from_slice(data);
                }
            }
        }
    }

    pub fn is_root(&self) -> bool {
        let parent = self.parent_db_key();
        parent.key().eq(&DbKey::ZERO_KEY)
    }

    pub fn make_db_key(&mut self) {
        let n = new_db_key();
        unsafe { std::ptr::copy(n.as_ptr(), self.data.as_mut_ptr().offset(Node::OFFSET_DB_KEY), n.len()) }
    }

    pub fn db_key(&self) -> DbKey {
        DbKey::from(&self.data[Node::OFFSET_DB_KEY as usize..Node::OFFSET_DB_KEY as usize + DbKey::LEN_DB_KEY])
    }

    pub fn set_db_key(&mut self, key: &[u8]) {
        unsafe {
            std::ptr::copy(key.as_ptr(), self.data.as_mut_ptr().offset(Node::OFFSET_DB_KEY), key.len());
        }
    }

    pub fn parent_db_key(&self) -> DbKey {
        DbKey::from(&self.data[Node::OFFSET_PARENT_DB_KEY as usize..Node::OFFSET_PARENT_DB_KEY as usize + DbKey::LEN_DB_KEY])
    }
    pub fn set_parent_db_key(&mut self, key: &[u8]) {
        Node::set_parent_db_key_data(&mut self.data, key);
    }

    pub fn set_parent_none(&mut self) {
        Node::set_parent_db_key_data(&mut self.data, &DbKey::ZERO_KEY);
    }

    pub fn set_parent_db_key_data(data: &mut [u8], key: &[u8]) {
        unsafe {
            std::ptr::copy(key.as_ptr(), data.as_mut_ptr().offset(Node::OFFSET_PARENT_DB_KEY), key.len());
        }
    }

    /// split creates a sibling node from a given node by splitting the node in two around a median.
    /// split will split the child at b leaving the [0, b-1] keys
    /// while moving the set of [b, 2b-1] keys to the sibling.
    pub fn split(&mut self, at: u64) -> Result<(Vec<u8>, Node), Error> {
        let node = self;
        let at = at as usize;
        match &mut node.node_type {
            NodeType::Internal(children, keys) => {
                let mut new_children = Children::new();
                let mut new_keys = VecBytes::new();
                let mut mid_key = vec![];

                let mut new_data = Vec::with_capacity(Node::OFFSET_NODE_DATA as usize + node.data.len() / 2);
                // let mut re = Vec::with_capacity(keys.bytes_number as usize);
                let new_offset = Node::OFFSET_NODE_DATA as isize;
                unsafe {
                    new_children.set_number_children(children.number_children - at as LenType, &mut new_data);
                    children.set_number_children(at as LenType, &mut node.data);

                    let start = new_offset + Children::OFFSET_DATA + children.number_children as isize * DbKey::LEN_DB_KEY as isize;
                    let count = new_children.number_children as usize * DbKey::LEN_DB_KEY;
                    std::ptr::copy_nonoverlapping(
                        node.data.as_ptr().offset(start),
                        new_data.as_mut_ptr().offset(Node::OFFSET_NODE_DATA as isize + Children::OFFSET_DATA),
                        count,
                    );

                    std::ptr::copy(
                        node.data.as_ptr().offset(start + count as isize),
                        node.data.as_mut_ptr().offset(start),
                        node.data.len() - start as usize - count,
                    );
                    node.data.set_len(node.data.len() - count);
                }
                new_keys.offset = new_children.offset_keys();
                new_keys.set_number_keys(at as LenType - 1, &mut new_data);

                let mut offset_original = keys.offset + Keys::OFFSET_DATA;
                unsafe {
                    for i in 0..keys.number_keys {
                        if i as usize == at {
                            let b = read_int_ptr::<BytesType>(node.data.as_ptr().offset(offset_original));
                            mid_key.reserve_exact(b as usize);
                            mid_key.set_len(b as usize);
                            std::ptr::copy_nonoverlapping(
                                node.data.as_ptr().offset(offset_original + size_of::<BytesType>() as isize),
                                new_data.as_mut_ptr(),
                                mid_key.len(),
                            );
                            let temp_offset = offset_original + b as isize + size_of::<BytesType>() as isize;

                            let new_offset = new_keys.offset + Keys::OFFSET_DATA;
                            new_keys.set_bytes_data(node.data.len() as BytesType - temp_offset as BytesType, &mut new_data);
                            std::ptr::copy_nonoverlapping(
                                node.data.as_ptr().offset(temp_offset),
                                new_data.as_mut_ptr().offset(new_offset),
                                new_keys.bytes_data as usize,
                            );
                            break;
                        }
                        let b = read_int_ptr::<BytesType>(node.data.as_ptr().offset(offset_original));
                        offset_original += b as isize + size_of::<BytesType>() as isize;
                    }
                    keys.set_number_keys(at as LenType - 1, &mut node.data);
                    keys.set_bytes_data((offset_original - keys.offset - Keys::OFFSET_DATA) as BytesType, &mut node.data);
                    node.data.set_len(offset_original as usize);
                }
                Ok((
                    mid_key,
                    Node {
                        node_type: NodeType::Internal(new_children, new_keys),
                        data: new_data,
                    },
                ))
            }
            NodeType::Leaf(_leaf) => {
                //todo
                let new_leaf = LeafData::new();
                let new_data = Vec::with_capacity(Node::OFFSET_NODE_DATA as usize);
                Ok((
                    vec![],
                    Node {
                        data: new_data,
                        node_type: NodeType::Leaf(new_leaf),
                    },
                ))
            }
            NodeType::None => Err(Error::UnexpectedError),
        }
    }
}

/// Implement TryFrom<Page> for Node allowing for easier
/// deserialization of data from a Page.
impl TryFrom<Vec<u8>> for Node {
    type Error = Error;
    fn try_from(data: Vec<u8>) -> Result<Node, Error> {
        let raw = data.as_slice();
        let node_type = NodeType::from(raw[Node::OFFSET_NODE_TYPE as usize]);

        match node_type {
            NodeType::Internal(mut children, mut keys) => {
                children.read_from(raw);
                keys.read_from(raw, children.offset_keys());
                Ok(Node {
                    node_type: NodeType::Internal(children, keys),
                    data,
                })
            }

            NodeType::Leaf(mut leaf) => {
                leaf.read_from(raw, Node::OFFSET_NODE_DATA);
                Ok(Node {
                    node_type: NodeType::Leaf(leaf),
                    data,
                })
            }
            NodeType::None => Err(Error::UnexpectedError),
        }
    }
}
