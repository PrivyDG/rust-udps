use udps::prelude::*;

fn loop_fun(i: i32) {
    //std::thread::sleep_ms(1000);
    println!("Loop iteration #{}", i + 1);
}

fn main() {
    let mut i = 0;
    let mut ms: u64 = 0;
    loop_at!(60, ms, {
        println!("Time elapsed since last call: {}ms", &ms);
        loop_fun(i);
        i += 1;
        if i == (100-1) {
            println!("Exiting loop on iteration #{}", i + 1);
            break;
        }
    });
}