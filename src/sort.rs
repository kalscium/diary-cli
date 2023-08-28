use soulog::*;
use lazy_db::*;

pub fn younger(this: &[u16; 3], other: &[u16; 3]) -> bool {
    let this_date = this[0] * 10000 + this[1] * 100 + this[2];
    let other_date = other[0] * 10000 + other[1] * 100 + other[2];
    this_date < other_date
}

pub fn sort(mut logger: impl Logger) {
    
}