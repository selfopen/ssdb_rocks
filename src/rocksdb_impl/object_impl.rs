
use rocksdb::TransactionDB;

use crate::{Object, RrError};
use crate::rocksdb_impl::shared::make_key;

/// 这个集合适合字段数量比较少时使用，
/// 实现，把所有的字段名存放到一个key中，这样方便于对整个字段的管理，同样也会产生一个问题，就是不要有太多的字段
/// 每个字段的key生成方式为，为key生成一个唯一的id, 这样解决kv数据库中k冲突的问题
/// 在取回所有Key或value时，使用的是前缀遍历
pub struct ObjectImp {}

impl Object<TransactionDB> for ObjectImp {
    fn del(&self, t: &TransactionDB, key: &[u8], field: &[u8]) -> Result<(), RrError> {
        let new_key = make_key(key, field);
        t.delete(&new_key)?;
        Ok(())
    }

    fn dels(&self, t: &TransactionDB, key: &[u8], fields: &[&[u8]]) -> Result<i64, RrError> {
        let mut count = 0;
        for f in fields {
            let new_key = make_key(key, f);
            t.delete(&new_key)?;
            count += 1;
        }
        Ok(count)
    }

    fn exists(&self, t: &TransactionDB, key: &[u8], field: &[u8]) -> Result<bool, RrError> {
        let new_key = make_key(key, field);
        let old = t.get(&new_key)?;
        Ok(old.is_some())
    }

    fn get(&self, t: &TransactionDB, key: &[u8], field: &[u8]) -> Result<Option<Vec<u8>>, RrError> {
        let new_key = make_key(key, field);
        let v = t.get(&new_key)?;
        return Ok(v);
    }

    fn get_all(&self, t: &TransactionDB, key: &[u8]) -> Result<Option<Vec<(Vec<u8>, Vec<u8>)>>, RrError> {
        let mut re = Vec::with_capacity(10);
        let new_key = make_key(key, &[]);
        let it = t.prefix_iterator(new_key);
        for k in it {
            let kk = k?;
            re.push((kk.0.to_vec(), kk.1.to_vec()));
        }
        Ok(Some(re))
    }

    fn keys(&self, t: &TransactionDB, key: &[u8]) -> Result<Option<Vec<Vec<u8>>>, RrError> {
        let mut re = Vec::with_capacity(10);
        let new_key = make_key(key, &[]);
        let it = t.prefix_iterator(new_key);
        for k in it {
            let kk = k?;
            re.push(kk.0.to_vec());
        }
        Ok(Some(re))
    }

    fn len(&self, t: &TransactionDB, key: &[u8]) -> Result<Option<i64>, RrError> {
        let new_key = make_key(key, &[]);
        let it = t.prefix_iterator(new_key);
        let l = it.count();
        Ok(Some(l as i64))
    }

    fn mget(&self, t: &TransactionDB, key: &[u8], fields: &[&[u8]]) -> Result<Vec<Option<Vec<u8>>>, RrError> {
        let mut values = Vec::with_capacity(fields.len());
        for f in fields {
            let new_key = make_key(key, f);
            if let Some(v) = t.get(new_key)? {
                values.push(Some(v));
            } else {
                values.push(None);
            }
        }
        Ok(values)
    }

    fn set(&self, t: &TransactionDB, key: &[u8], field: &[u8], value: &[u8]) -> Result<(), RrError> {
        let new_key = make_key(key, field);
        t.put(&new_key, value)?;
        Ok(())
    }

    fn set_not_exist(&self, t: &TransactionDB, key: &[u8], field: &[u8], value: &[u8]) -> Result<i32, RrError> {
        let new_key = make_key(key, field);
        if let None = t.get(&new_key)? {
            t.put(new_key, value)?;
            return Ok(1);
        } else {
            return Ok(0);
        }
    }

    fn set_exist(&self, t: &TransactionDB, key: &[u8], field: &[u8], value: &[u8]) -> Result<i32, RrError> {
        let new_key = make_key(key, field);
        if let Some(_) = t.get(&new_key)? {
            t.put(new_key, value)?;
            return Ok(1);
        } else {
            return Ok(0);
        }
    }

    fn vals(&self, t: &TransactionDB, key: &[u8]) -> Result<Vec<Vec<u8>>, RrError> {
        let mut re = Vec::with_capacity(10);
        let new_key = make_key(key, &[]);
        let it = t.prefix_iterator(new_key);
        for k in it {
            let kk = k?;
            re.push(kk.1.to_vec());
        }
        Ok(re)
    }

    fn remove_key(&self, t: &TransactionDB, key: &[u8]) -> Result<(), RrError> {
        let new_key = make_key(key, &[]);
        let it = t.prefix_iterator(new_key);
        for k in it {
            let kk = k?;
            t.delete(kk.0)?;
        }
        Ok(())
    }
}

