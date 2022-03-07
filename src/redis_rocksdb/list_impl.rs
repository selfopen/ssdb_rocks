use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ckb_rocksdb::prelude::{Get, Put, TransactionBegin};
use ckb_rocksdb::WriteOptions;

use crate::{Bytes, Direction, Error, LenType, MetaKey, RedisList, RedisRocksdb};
use crate::redis_rocksdb::quick_list::QuickList;
use crate::redis_rocksdb::quick_list_node::QuickListNode;
use crate::redis_rocksdb::zip_list::ZipList;

/// [see] (https://xindoo.blog.csdn.net/article/details/109150975)
/// ssdb没有实现list，只实现了queue
///
/// redis中的list使用quicklist与ziplist实现
impl RedisList for RedisRocksdb {
    fn blpop<K: Bytes, V: Bytes>(&mut self, key: K, timeout: i64) -> Result<V, Error> {
        todo!()
    }

    fn brpop<K: Bytes, V: Bytes>(&mut self, key: K, timeout: i64) -> Result<V, Error> {
        todo!()
    }

    fn brpoplpush<K: Bytes, V: Bytes>(&mut self, srckey: K, dstkey: K, timeout: i64) -> Result<V, Error> {
        todo!()
    }

    fn lindex<K: Bytes>(&self, key: K, index: i32) -> Result<Vec<u8>, Error> {
        let t = QuickList::get(&self.db, key.as_ref())?.ok_or(Error::not_find("key of list"))?;
        if index >= t.len_list() as i32 {
            return Err(Error::not_find(&format!("the index {}", index)));
        }
        //todo read only
        let tr = self.db.transaction_default();
        let node_key = t.left().ok_or(Error::none_error("left of quick list"))?;
        let mut node = QuickListNode::get(&tr, node_key.as_ref())?.ok_or(Error::none_error("left node"))?;
        let mut it_index = 0i32;
        it_index += node.len_list() as i32;
        while index >= it_index {
            let next_key = node.right().ok_or(Error::none_error("right node"))?;
            node = QuickListNode::get(&tr, next_key.as_ref())?.ok_or(Error::none_error("next node"))?;
            it_index += node.len_list() as i32;
        }

        let value_key = node.values_key().ok_or(Error::none_error("value key"))?;
        let zip = ZipList::get(&tr, value_key.as_ref())?.ok_or(Error::none_error("zip list"))?;
        let zip_index = index - (it_index - node.len_list() as i32);
        let v = zip.index(zip_index).ok_or(Error::not_find(&format!("the index {}", index)))?;
        tr.commit()?;
        Ok(v.to_vec())
    }

    fn linsert_before<K: Bytes, P: Bytes, V: Bytes>(&mut self, key: K, pivot: P, value: V) -> Result<(), Error> {
        todo!()
    }

    fn linsert_after<K: Bytes, P: Bytes, V: Bytes>(&mut self, key: K, pivot: P, value: V) -> Result<(), Error> {
        todo!()
    }

    fn llen<K: Bytes>(&self, key: K) -> Result<i32, Error> {
        match QuickList::get(&self.db, key.as_ref())? {
            None => Ok(-1),
            Some(quick) => {
                Ok(quick.len_list() as i32)
            }
        }
    }

    fn lmove<K: Bytes, V: Bytes>(&mut self, srckey: K, dstkey: K, src_dir: Direction, dst_dir: Direction) -> Result<V, Error> {
        todo!()
    }

    fn lmpop<K: Bytes>(&mut self, numkeys: i32, key: K, dir: Direction, count: i32) {
        todo!()
    }

    fn lpop<K: Bytes, V: Bytes>(&mut self, key: K) -> Result<V, Error> {
        todo!()
    }

    fn lpush<K: Bytes, V: Bytes>(&mut self, list_key: K, value: V) -> Result<i32, Error> {
        let tr = self.db.transaction_default();
        let mut quick = match QuickList::get(&self.db, list_key.as_ref())? {
            None => {
                let mut q = QuickList::new();
                q.init_meta_key(list_key.as_ref());
                q
            }
            Some(q) => q
        };
        let re = quick.lpush(&tr, list_key.as_ref(), value.as_ref())?;
        tr.commit()?;
        Ok(re)
    }

    fn lpush_exists<K: Bytes, V: Bytes>(&mut self, list_key: K, value: V) -> Result<i32, Error> {
        let tr = self.db.transaction_default();
        let mut quick = match QuickList::get(&self.db, list_key.as_ref())? {
            None => return Ok(-1),
            Some(q) => q
        };
        let re = quick.lpush(&tr, list_key.as_ref(), value.as_ref())?;
        tr.commit()?;
        Ok(re)
    }


    fn lrange<K: Bytes, V: Bytes>(&self, key: K, start: i32, stop: i32) -> Result<Vec<V>, Error> {
        todo!()
    }

    fn lrem<K: Bytes, V: Bytes>(&mut self, key: K, count: i32, value: V) -> Result<V, Error> {
        todo!()
    }

    fn ltrim<K: Bytes>(&mut self, key: K, start: i32, stop: i32) -> Result<i32, Error> {
        todo!()
    }

    fn lset<K: Bytes, V: Bytes>(&mut self, key: K, index: i32, value: V) {
        todo!()
    }

    fn rpop<K: Bytes>(&mut self, key: K, count: Option<i32>) {
        todo!()
    }

    fn rpoplpush<K: Bytes, V: Bytes>(&mut self, key: K, dstkey: K) -> Result<V, Error> {
        todo!()
    }

    fn rpush<K: Bytes, V: Bytes>(&mut self, key: K, value: V) -> Result<(), Error> {
        todo!()
    }

    fn rpush_exists<K: Bytes, V: Bytes>(&mut self, key: K, value: V) -> Result<(), Error> {
        todo!()
    }
}
