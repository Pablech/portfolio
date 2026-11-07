use std::sync::LazyLock;

pub static COMBS: LazyLock<Vec<[usize; 4]>> = LazyLock::new(|| {
    let mut c = Vec::with_capacity(69);
    for v in 0..6_usize { for h in 0..4_usize {
        let p = v * 7 + h; c.push([p, p+1, p+2, p+3]); }}
    for h in 0..7_usize { for v in 0..3_usize {
        let p = v * 7 + h; c.push([p, p+7, p+14, p+21]); }}
    for v in 0..3_usize { for h in 0..4_usize {
        let p = v * 7 + h; c.push([p, p+8, p+16, p+24]); }}
    for v in 3..6_usize { for h in 0..4_usize {
        let p = v * 7 + h; c.push([p, p-6, p-12, p-18]); }}
    assert_eq!(c.len(), 69);
    c
});
