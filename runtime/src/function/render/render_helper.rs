#[inline]
pub fn round_up(value: u32, alignment: u32) -> u32 {
    let temp = value + alignment -1;
    return temp - temp % alignment;
}