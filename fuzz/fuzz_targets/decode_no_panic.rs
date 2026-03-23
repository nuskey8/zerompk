#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = zerompk::from_msgpack_slice::<bool>(data);
    let _ = zerompk::from_msgpack_slice::<i64>(data);
    let _ = zerompk::from_msgpack_slice::<u64>(data);
    let _ = zerompk::from_msgpack_slice::<f32>(data);
    let _ = zerompk::from_msgpack_slice::<f64>(data);
    let _ = zerompk::from_msgpack_slice::<String>(data);
    let _ = zerompk::from_msgpack_slice::<Vec<u8>>(data);
    let _ = zerompk::from_msgpack_slice::<Option<i32>>(data);
    let _ = zerompk::from_msgpack_slice::<(i32, bool, String)>(data);
});
