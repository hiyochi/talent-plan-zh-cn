```rust
use std::num::Wrapping;

#[derive(Clone)]
pub struct Bitset(Vec<u64>);

impl Bitset {
    // 创建一个新的位集合，指定位数
    pub fn new(bits: usize) -> Self {
        let extra = if bits % 64 != 0 { 1 } else { 0 };
        Bitset(vec![0; bits / 64 + extra])
    }

    // 设置指定位置的位为1
    pub fn set(&mut self, pos: usize) {
        let (major, minor) = bitset_index(pos);
        self.0[major] |= 1 << minor;
    }

    // 清除指定位置的位（设为0）
    pub fn clear(&mut self, pos: usize) {
        let (major, minor) = bitset_index(pos);
        self.0[major] &= !(1 << minor);
    }

    // 计算位集合中1的个数（使用位操作优化）
    fn popcnt(&self) -> usize {
        let mut total = 0;
        for b in &self.0 {
            let mut v = *b;
            v = (v & 0x5555_5555_5555_5555) + ((v & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
            v = (v & 0x3333_3333_3333_3333) + ((v & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
            v = (v & 0x0F0F_0F0F_0F0F_0F0F) + ((v & 0xF0F0_F0F0_F0F0_F0F0) >> 4);
            v = (Wrapping(v) * Wrapping(0x0101_0101_0101_0101)).0;
            total += ((v >> 56) & 0xFF) as usize;
        }
        total
    }

    // 计算位集合的哈希值
    pub fn hash(&self) -> u64 {
        let mut hash = self.popcnt() as u64;
        for v in &self.0 {
            hash ^= v;
        }
        hash
    }

    // 比较两个位集合是否相等
    pub fn equals(&self, b2: &Bitset) -> bool {
        let b = &self.0;
        let b2 = &b2.0;
        if b.len() != b2.len() {
            return false;
        }
        for i in 0..b.len() {
            if b[i] != b2[i] {
                return false;
            }
        }
        true
    }
}

// 将位位置转换为数组索引和位偏移
fn bitset_index(pos: usize) -> (usize, usize) {
    (pos / 64, pos % 64)
}
```