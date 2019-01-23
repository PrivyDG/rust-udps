use std::mem::*;

pub fn conv_slice_to_u32(slice: &[u8]) -> u32 {
    let buf = [
        slice[0],
        slice[1],
        slice[2],
        slice[3]
    ];
    u32::from_le_bytes(buf)
}

pub fn conv_u32_to_bytes(input: &u32) -> [u8; 4] {
    let bytes = unsafe {
        transmute(input.to_le())
    };
    return bytes;
}

pub fn generate_random_bytes(n: usize) -> std::vec::Vec<u8> {
    use rand::prelude::*;
    use std::vec::*;
    let mut vec: Vec<u8> = Vec::with_capacity(n);
    let mut rand = rand::thread_rng();
    for i in 0..n {
        vec[i] = rand.gen_range(0, 255);
    }
    vec
}


#[macro_export]
macro_rules! loop_at {
    ($n:expr, $e:expr) => {
        {
            use std::time::*;
            let mut timestamp_last;
            let mut now = Instant::now();
            let loop_time = 1000 / $n;
            loop {
                timestamp_last = Instant::now();
                $e
                now = Instant::now();
                let elapsed_ms = now.duration_since(timestamp_last).as_millis() as u64;
                if elapsed_ms <= loop_time {

                    std::thread::sleep(
                        Duration::from_millis(
                            loop_time - elapsed_ms
                        )
                    );
                }
            }
        }
    };
    ($n:expr, $i:ident, $e:expr) => {
        {
            use std::time::*;
            let mut timestamp_last;
            let mut now = Instant::now();
            let loop_time = 1000 / $n;
            loop {
                timestamp_last = Instant::now();
                $i = timestamp_last.duration_since(now).as_millis() as u64;
                $e
                now = Instant::now();
                let elapsed_ms = now.duration_since(timestamp_last).as_millis() as u64;
                if elapsed_ms <= loop_time {
                    std::thread::sleep(
                        Duration::from_millis(
                            loop_time - elapsed_ms
                        )
                    );
                }
            }
        }
    };
    ($n:expr, $ms:ident, $i:ident, $e:expr) => {

    };
}
