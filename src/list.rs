use crate::{Bytes, LenType};
use crate::RrError;

pub trait RedisList {
    fn list_blpop<K: Bytes, V: Bytes>(&mut self, key: &K, timeout: i64) -> Result<V, RrError>;
    fn list_brpop<K: Bytes, V: Bytes>(&mut self, key: &K, timeout: i64) -> Result<V, RrError>;
    fn list_brpoplpush<K: Bytes, V: Bytes>(&mut self, srckey: &K, dstkey: &K, timeout: i64) -> Result<V, RrError>;
    fn list_index<K: Bytes>(&self, key: &K, index: i32) -> Result<Vec<u8>, RrError>;

    /// 如果命令执行成功，返回插入操作完成之后，列表的长度。
    /// 如果没有找到指定元素 ，返回 -1 。
    /// 如果 key 不存在或为空列表，返回 0
    fn list_insert_before<K: Bytes, V: Bytes>(&mut self, key: &K, pivot: &V, value: &V) -> Result<i32, RrError>;
    /// 如果命令执行成功，返回插入操作完成之后，列表的长度。
    /// 如果没有找到指定元素 ，返回 -1 。
    /// 如果 key 不存在或为空列表，返回 0
    fn list_insert_after<K: Bytes, V: Bytes>(&mut self, key: &K, pivot: &V, value: &V) -> Result<i32, RrError>;

    // 返回值为-1表示还没有这个list
    fn list_len<K: Bytes>(&self, key: &K) -> Result<i32, RrError>;

    /// 对应redis的 lpop，由于lpop命名，不是很明确，所以改名
    fn list_pop_front<K: Bytes>(&mut self, key: &K) -> Result<Option<Vec<u8>>, RrError>;
    /// 返回len of list
    /// 对应redis的lpush，由于lpush命名，不是很明确，所以改名
    fn list_push_front<K: Bytes, V: Bytes>(&mut self, key: &K, value: &V) -> Result<i32, RrError>;

    // /// 返回len of list
    // fn lpushs<K: Bytes, V: Bytes>(&mut self, key: &K, values: &[&V]) -> Result<i32, RrError>;

    /// 返回len of list，如果list不存在返回值为 0
    /// 对应redis的lpushx
    fn list_push_front_exists<K: Bytes, V: Bytes>(&mut self, key: &K, value: &V) -> Result<i32, RrError>;
    /// 返回在range范围内的元素，所以start与stop可能会在list的下标之外。range是包含stop的
    /// 如果一个都没有找到，返回为len为0的Vec
    /// 0表示第一个元素
    /// -1表示倒数第一个元素
    /// -100 100表示从到数100个元素到第101个元素。如果这时list中只有3个元素，返回所有的值，因为这3个都在 range的范围之内
    fn list_range<K: Bytes>(&self, key: &K, start: i32, stop: i32) -> Result<Vec<Vec<u8>>, RrError>;
    /// 返回值为删除的数量
    /// COUNT 的值可以是以下几种：
    /// count > 0 : 从表头开始向表尾搜索，移除与 VALUE 相等的元素，数量为 COUNT。
    /// count < 0 : 从表尾开始向表头搜索，移除与 VALUE 相等的元素，数量为 COUNT 的绝对值。
    /// count = 0 : 移除表中所有与 VALUE 相等的值
    fn list_rem<K: Bytes, V: Bytes>(&mut self, key: &K, count: i32, value: &V) -> Result<LenType, RrError>;
    /// 保留指定区间内的元素，不在指定区间之内的元素都将被删除, 反回删除的元素数量
    fn list_trim<K: Bytes>(&mut self, key: K, start: i32, stop: i32) -> Result<i32, RrError>;

    /// index无效或list为空时，返回错误。其余返回原来的值
    fn list_set<K: Bytes, V: Bytes>(&mut self, key: &K, index: i32, value: &V) -> Result<Vec<u8>, RrError>;
    /// 移除列表的最后一个元素
    fn list_pop_back<K: Bytes>(&mut self, key: &K) -> Result<Option<Vec<u8>>, RrError>;
    /// 移除列表的最后一个元素，并将该元素添加到另一个列表并返回
    /// 对应redis的rpoplpush
    fn list_replace_back<K: Bytes, V: Bytes>(&mut self, key: &K, dstkey: &K) -> Result<V, RrError>;
    /// 返回len of list
    /// 对应redis的rpush
    fn list_push_back<K: Bytes, V: Bytes>(&mut self, key: &K, value: &V) -> Result<i32, RrError>;
    // /// 返回len of list
    // fn rpushs<K: Bytes, V: Bytes>(&mut self, key: &K, value: &[&V]) -> Result<i32, RrError>;
    /// 为已经存在的列表添加值， 添加到尾部
    /// 对应redis的rpushx
    fn list_push_back_exists<K: Bytes, V: Bytes>(&mut self, key: &K, value: &V) -> Result<i32, RrError>;

    /// 返回len of list
    fn list_clear<K: Bytes>(&mut self, key: &K) -> Result<i32, RrError>;
}