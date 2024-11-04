pub mod array;
pub mod de;
pub mod map;
mod reference;
mod value;
mod value_ref;

pub use array::Array;
pub use map::Map;
pub use reference::Reference;
pub use value::Value;
pub use value_ref::ValueRef;

#[cfg(test)]
fn gen_string<R: rand::Rng>(rng: &mut R) -> String {
    let len = rng.gen_range(1_usize..=50);

    let mut dst = String::with_capacity(len);

    for _ in 0..len {
        let mut b = rng.gen_range(b'0'..=b'z');
        while !b.is_ascii_alphanumeric() {
            b = rng.gen_range(b'0'..=b'z');
        }

        dst.push(b as char);
    }

    dst
}
