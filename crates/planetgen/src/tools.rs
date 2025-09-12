use rand::Rng;
use rand::distr::Uniform;

pub fn generate_seed8() -> u32 {
    let mut rng = rand::rng();
    rng.sample(Uniform::new(0u32, 100_000_000u32).unwrap())
}

pub fn expand_seed64(code: u32) -> u64 {
    splitmix64(code as u64)
}

pub fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}
