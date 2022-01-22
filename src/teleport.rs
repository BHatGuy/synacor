use std::collections::HashMap;
use std::io::{stdout, Write};


fn f(a: u16, b: u16, c: u16, cache: &mut HashMap<(u16, u16), u16>) -> u16  {
    if let Some(res) = cache.get(&(a, b)) {
        return *res;
    }
    let res;
    if a == 0 {
        res = b + 1;
    } else {
        if b == 0 {
            res = f(a-1, c, c, cache);
        } else {
            res = f(a-1, f(a, b-1, c, cache), c, cache);
        }
    }
    cache.insert((a, b), res);
    return res;
}

fn main() {
    for i in 0..0x8000 {
        let mut cache = HashMap::new();
        let res = f(4, 1, i, &mut cache);
        if i % 16 == 0{
            print!("0x{:04x}\r", i);
            stdout().flush().unwrap();
        }
        if res == 6 {
            println!("{:#04x} {}", i, res);
            break;
        }
    }
}