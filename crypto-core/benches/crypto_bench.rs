use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crypto_core::{
    cipher::{decrypt, encrypt},
    kdf::{derive_keys, derive_master_key, Salt},
    password::{generate_password, PasswordOptions},
};
use rand::RngCore;

fn benchmark_key_derivation(c: &mut Criterion) {
    let salt = Salt::generate().unwrap();

    c.bench_function("derive_master_key", |b| {
        b.iter(|| derive_master_key(black_box("test_password"), black_box(&salt)))
    });
}

fn benchmark_hkdf(c: &mut Criterion) {
    let salt = Salt::generate().unwrap();
    let master_key = derive_master_key("test_password", &salt).unwrap();

    c.bench_function("derive_keys", |b| {
        b.iter(|| derive_keys(black_box(&master_key)))
    });
}

fn benchmark_encryption(c: &mut Criterion) {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    let data = vec![0u8; 1024]; // 1KB of data

    c.bench_function("encrypt_1kb", |b| {
        b.iter(|| encrypt(black_box(&data), black_box(&key)))
    });
}

fn benchmark_decryption(c: &mut Criterion) {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);

    let data = vec![0u8; 1024];
    let blob = encrypt(&data, &key).unwrap();

    c.bench_function("decrypt_1kb", |b| {
        b.iter(|| decrypt(black_box(&blob), black_box(&key)))
    });
}

fn benchmark_password_generation(c: &mut Criterion) {
    let options = PasswordOptions::default();

    c.bench_function("generate_password", |b| {
        b.iter(|| generate_password(black_box(&options)))
    });
}

criterion_group!(
    benches,
    benchmark_key_derivation,
    benchmark_hkdf,
    benchmark_encryption,
    benchmark_decryption,
    benchmark_password_generation,
);

criterion_main!(benches);
