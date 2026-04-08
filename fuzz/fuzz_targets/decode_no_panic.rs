#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = zerompk::from_msgpack::<bool>(data);
    let _ = zerompk::from_msgpack::<i64>(data);
    let _ = zerompk::from_msgpack::<u64>(data);
    let _ = zerompk::from_msgpack::<f32>(data);
    let _ = zerompk::from_msgpack::<f64>(data);
    let _ = zerompk::from_msgpack::<String>(data);
    let _ = zerompk::from_msgpack::<Vec<u8>>(data);
    let _ = zerompk::from_msgpack::<Option<i32>>(data);
    let _ = zerompk::from_msgpack::<(i32, bool, String)>(data);
});
