use std::{cmp::Ordering, mem, mem::ManuallyDrop, ptr, slice};

use compare::Compare;

use crate::{read_int, read_int_ptr, write_int, write_int_ptr, LenType};

#[derive(Clone)]
pub(crate) struct MaxHeapCompare {
    pub(crate) heap: *mut FieldHeap<MaxHeapCompare>,
}

impl Compare<FieldMeta> for MaxHeapCompare {
    fn compare(&self, l: &FieldMeta, r: &FieldMeta) -> Ordering {
        unsafe {
            let field_heap = &(*self.heap);
            let p = field_heap.data.as_ptr().offset(field_heap.bst_capt + FieldHeap::<MaxHeapCompare>::BST_OFFSET);
            let l_len = read_int_ptr::<SizeField>(p.offset(l.offset)) as usize;
            let l_v = slice::from_raw_parts(p.offset(l_len as isize + l.offset), l_len);
            let r_len = read_int_ptr::<SizeField>(p.offset(r.offset)) as usize;
            let r_v = slice::from_raw_parts(p.offset(r_len as isize + r.offset), r_len);
            l_v.cmp(r_v)
        }
    }
}

#[derive(Clone)]
pub(crate) struct MinHeapCompare {
    pub(crate) heap: *mut FieldHeap<MinHeapCompare>,
}

impl Compare<FieldMeta> for MinHeapCompare {
    fn compare(&self, l: &FieldMeta, r: &FieldMeta) -> Ordering {
        unsafe {
            let field_heap = &(*self.heap);
            let p = field_heap.data.as_ptr().offset(field_heap.bst_capt + FieldHeap::<MaxHeapCompare>::BST_OFFSET);
            let l_len = read_int_ptr::<SizeField>(p.offset(l.offset)) as usize;
            let l_v = slice::from_raw_parts(p.offset(l_len as isize + l.offset), l_len);
            let r_len = read_int_ptr::<SizeField>(p.offset(r.offset)) as usize;
            let r_v = slice::from_raw_parts(p.offset(r_len as isize + r.offset), r_len);
            //由于是最小堆，所以反过比较
            r_v.cmp(l_v)
        }
    }
}

///所有的field连续存入一遍连续的内存区中
/// [C++ Binary Search Tree array implementation](https://www.daniweb.com/programming/software-development/threads/466340/c-binary-search-tree-array-implementation)
/// [ArrayBinarySearchTree.java](http://faculty.washington.edu/moishe/javademos/jss2/ArrayBinarySearchTree.java)
/// [binary-search-tree(not array)](https://www.geeksforgeeks.org/binary-search-tree-set-1-search-and-insertion/?ref=lbp)
/// [binary-search-tree(not array)](https://www.javatpoint.com/binary-search-tree)
pub(crate) struct FieldHeap<T: Compare<FieldMeta> + Clone> {
    pub data: Vec<u8>,
    /// 为bst分配置的空间大小， 默认为256，增加方式 每次增加256个
    bst_capt: isize,
    comparer: Option<T>,
}

//存放字段名的数据大小
pub(crate) type SizeField = i32;

pub(crate) struct FieldMeta {
    pub offset: isize,
}

impl<T: Compare<FieldMeta> + Clone> FieldHeap<T> {
    pub const SIZE: usize = mem::size_of::<SizeField>();
    pub const BST_OFFSET: isize = 2 * (mem::size_of::<LenType>() as isize);
    pub const BST_EXPAND: isize = 64 * (mem::size_of::<FieldMeta>() as isize);

    pub fn new(data: Vec<u8>) -> Self {
        let mut data = data;
        let mut bst_capt = Self::BST_EXPAND;
        if data.is_empty() {
            data.resize(Self::BST_OFFSET as usize + bst_capt as usize, 0);
            unsafe { write_int_ptr(data.as_mut_ptr().offset(mem::size_of::<LenType>() as isize), bst_capt as LenType) }
        } else {
            unsafe {
                bst_capt = read_int_ptr::<LenType>(data.as_ptr().offset(mem::size_of::<LenType>() as isize)) as isize;
            }
        };
        FieldHeap {
            data,
            bst_capt,
            comparer: None,
        }
    }

    pub fn init(&mut self, comparer: T) {
        self.comparer = Some(comparer);
    }

    fn make_heap(&mut self) -> binary_heap_plus::BinaryHeap<FieldMeta, T> {
        let head_array = unsafe {
            Vec::from_raw_parts(
                self.data.as_mut_ptr().offset(Self::BST_OFFSET as isize) as *mut FieldMeta,
                self.len(),
                self.bst_capt as usize / mem::size_of::<FieldMeta>(),
            )
        };
        let t = unsafe { binary_heap_plus::BinaryHeap::from_vec_cmp_raw(head_array, self.comparer.as_ref().expect("").clone(), false) };
        return t;
    }

    fn drop_heap(&mut self, heap: binary_heap_plus::BinaryHeap<FieldMeta, T>) {
        let data = heap.into_vec();
        let _ = ManuallyDrop::new(data);
    }

    /// 计算字段的偏移位置
    fn field_offset(&self) -> isize {
        Self::BST_OFFSET + self.bst_capt
    }
    pub fn peek(&mut self) -> Option<Vec<u8>> {
        let heap = self.make_heap();
        let v = heap.peek();
        let pop_v = if let Some(v) = v {
            let start = v.offset + Self::BST_OFFSET + self.bst_capt;
            let field_size = unsafe { read_int_ptr::<SizeField>(self.data.as_ptr().offset(start)) };
            let end = start + Self::SIZE as isize + field_size as isize;
            let re = self.data[start as usize + Self::SIZE as usize..end as usize].to_vec();
            Some(re)
        } else {
            None
        };
        self.drop_heap(heap);
        pop_v
    }

    pub fn pop(&mut self) -> Option<Vec<u8>> {
        let mut heap = self.make_heap();
        let v = heap.pop();
        self.drop_heap(heap);
        if let Some(v) = v {
            let len_field = self.len() - 1;
            self.set_len(len_field);
            let start = v.offset + Self::BST_OFFSET + self.bst_capt;
            let field_size = unsafe { read_int_ptr::<SizeField>(self.data.as_ptr().offset(start)) };
            let end = start + Self::SIZE as isize + field_size as isize;
            let re = self.data[start as usize + Self::SIZE as usize..end as usize].to_vec();
            //如果删除的数据，等于或超过一次扩展的数据，那么进行清理，把没有使用的空间删除（压缩数据）
            if self.bst_capt as usize - len_field * mem::size_of::<FieldMeta>() > Self::BST_EXPAND as usize {
                self.reduce();
            }
            Some(re)
        } else {
            None
        }
    }
    /// 由于head结构查找很慢，所以不能插入相同的key
    pub fn push(&mut self, field: &[u8]) {
        //把字段加入最后
        //检查是否有heap的空间是否够大
        let len = self.len();
        if len * mem::size_of::<FieldMeta>() >= self.bst_capt as usize {
            self.expand();
        }

        let add = Self::SIZE + field.len();
        self.data.reserve(add);
        let len_data = self.data.len();
        unsafe {
            let p = self.data.as_mut_ptr().offset(len_data as isize);
            //写入字段的bytes数量
            write_int_ptr(p, field.len() as SizeField);
            //写入字段
            ptr::copy_nonoverlapping(field.as_ptr(), p.offset(Self::SIZE as isize), field.len());
            self.data.set_len(self.data.len() + add)
        }

        let mut heap = self.make_heap();
        heap.push(FieldMeta {
            offset: len_data as isize - Self::BST_OFFSET - self.bst_capt,
        });
        self.drop_heap(heap);
        let len = self.len() + 1;
        //写入总的字段个数
        write_int_ptr(self.data.as_mut_ptr(), len as LenType);
    }

    pub fn len(&self) -> usize {
        let l = read_int::<LenType>(&self.data);
        return l as usize;
    }
    pub fn set_len(&mut self, l: usize) {
        write_int::<LenType>(&mut self.data, l as LenType);
    }
    pub(crate) fn new_field_it(&self) -> FieldIt<'_, T> {
        FieldIt::new(self)
    }

    fn expand(&mut self) {
        let expand_size = Self::BST_EXPAND as isize;
        self.data.reserve(expand_size as usize);
        unsafe {
            self.data.set_len(self.data.len() + expand_size as usize);
        }
        let old_capt = self.bst_capt;
        unsafe {
            let p_data = self.data.as_mut_ptr().offset(Self::BST_OFFSET + old_capt);
            ptr::copy(
                p_data,
                p_data.offset(expand_size as isize),
                self.data.len() - expand_size as usize - Self::BST_OFFSET as usize - old_capt as usize,
            );
            write_int_ptr(
                self.data.as_mut_ptr().offset(mem::size_of::<LenType>() as isize),
                old_capt as LenType + expand_size as LenType,
            );
        };
        self.bst_capt = old_capt + expand_size;
    }

    fn reduce(&mut self) {
        let reduce_size = Self::BST_EXPAND;
        let mut temp_fields = Vec::<u8>::with_capacity(self.data.len() - self.bst_capt as usize - Self::BST_OFFSET as usize);

        let mut head_array = unsafe {
            Vec::from_raw_parts(
                self.data.as_mut_ptr().offset(Self::BST_OFFSET) as *mut FieldMeta,
                self.len(),
                self.bst_capt as usize / mem::size_of::<FieldMeta>(),
            )
        };
        let mut offset = 0;
        let p_data = unsafe { self.data.as_ptr().offset(Self::BST_OFFSET + self.bst_capt) };
        for field_meta in &mut head_array {
            let start = field_meta.offset;
            let field_size = unsafe { read_int_ptr::<SizeField>(p_data.offset(start)) };
            unsafe {
                ptr::copy_nonoverlapping(p_data.offset(start), temp_fields.as_mut_ptr().offset(offset), field_size as usize + Self::SIZE);
            }
            field_meta.offset = offset;
            offset += field_size as isize + Self::SIZE as isize;
            unsafe {
                temp_fields.set_len(offset as usize);
            }
        }
        let _ = ManuallyDrop::new(head_array);
        self.bst_capt -= reduce_size;
        unsafe {
            temp_fields.set_len(offset as usize);
            write_int_ptr(self.data.as_mut_ptr().offset(mem::size_of::<LenType>() as isize), self.bst_capt as LenType);
            ptr::copy_nonoverlapping(
                temp_fields.as_ptr(),
                self.data.as_mut_ptr().offset(Self::BST_OFFSET + self.bst_capt),
                temp_fields.len(),
            );
            self.data.set_len(Self::BST_OFFSET as usize + self.bst_capt as usize + temp_fields.len());
        }
    }
}

pub(crate) struct FieldIt<'a, T: Compare<FieldMeta> + Clone> {
    data: &'a FieldHeap<T>,
    len: isize,
    index: isize,
    offset: isize,
}

impl<'a, T: Compare<FieldMeta> + Clone> FieldIt<'a, T> {
    pub fn new(d: &'a FieldHeap<T>) -> Self {
        FieldIt {
            data: d,
            len: 0,
            index: -1,
            offset: 0,
        }
    }
}

pub(crate) struct FieldItValue<'a> {
    pub field: &'a [u8],
}

impl<'a, T: Compare<FieldMeta> + Clone> Iterator for FieldIt<'a, T> {
    type Item = FieldItValue<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }
        if self.index < 0 {
            self.len = self.data.len() as isize;
            if self.len < 1 {
                return None;
            }
            self.offset = mem::size_of::<LenType>() as isize;
        }

        self.index += 1;
        let field_size = read_int_ptr::<SizeField>(unsafe { self.data.data.as_ptr().offset(self.offset) });
        let it = FieldItValue {
            field: unsafe { slice::from_raw_parts(self.data.data.as_ptr().offset(self.offset + FieldHeap::<T>::SIZE as isize), field_size as usize) },
        };
        return Some(it);
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use crate::write_int;

    #[test]
    fn test_binary_heap() {
        {
            {
                let mut t = binary_heap_plus::BinaryHeap::new();
                t.push(1);
                t.push(2);
                t.push(3);
                assert_eq!(3, t.pop().expect(""));
                assert_eq!(2, t.pop().expect(""));
                assert_eq!(1, t.pop().expect(""));
            }
            {
                const MAX: i32 = 6;
                let mut t = binary_heap_plus::BinaryHeap::<[u8; mem::size_of::<i32>()]>::new();
                for i in 1..MAX {
                    let mut field: [u8; mem::size_of::<i32>()] = [0; mem::size_of::<i32>()];
                    write_int(field.as_mut(), i);
                    t.push(field);
                }
                for i in (1..MAX).rev() {
                    let mut field: [u8; mem::size_of::<i32>()] = [0; mem::size_of::<i32>()];
                    write_int(field.as_mut(), i);
                    let data = t.pop().expect("");
                    assert_eq!(field, data);
                }
            }
            {
                const MAX: i32 = 6;
                let mut t = binary_heap_plus::BinaryHeap::new();
                for i in 1..MAX {
                    let mut field: [u8; mem::size_of::<i32>()] = [0; mem::size_of::<i32>()];
                    write_int(field.as_mut(), i);
                    t.push(field.to_vec());
                }
                for i in (1..MAX).rev() {
                    let mut field: [u8; mem::size_of::<i32>()] = [0; mem::size_of::<i32>()];
                    write_int(field.as_mut(), i);
                    let data = t.pop().expect("");
                    assert_eq!(field.to_vec(), data);
                }
            }
        }
        {
            {
                let mut t = binary_heap_plus::BinaryHeap::new_min();
                t.push(1);
                t.push(2);
                t.push(3);
                assert_eq!(1, t.pop().expect(""));
                assert_eq!(2, t.pop().expect(""));
                assert_eq!(3, t.pop().expect(""));
            }
            {
                const MAX: i32 = 6;
                let mut t = binary_heap_plus::BinaryHeap::new_min();
                for i in 1..MAX {
                    let mut field: [u8; mem::size_of::<i32>()] = [0; mem::size_of::<i32>()];
                    write_int(field.as_mut(), i);
                    t.push(field.to_vec());
                }
                for i in 1..MAX {
                    let mut field: [u8; mem::size_of::<i32>()] = [0; mem::size_of::<i32>()];
                    write_int(field.as_mut(), i);
                    let data = t.pop().expect("");
                    assert_eq!(field.to_vec(), data);
                }
            }
        }
    }
}
