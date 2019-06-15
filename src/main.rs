fn main() {
    println!("{}", decrypt("Hello"));
}

fn decrypt(input: &str) -> String {
    b(input)
}

fn a(a: Option<i32>, b: Option<i32>, c: Option<i32>, d: &DType) -> bool {
    match d.length {
        1 => a.is_some() && a.unwrap() == d.ch0,
        2 => a.is_some() && a.unwrap() == d.ch0 && b == d.ch1,
        3 => a.is_some() && a.unwrap() == d.ch0 && b == d.ch1 && c == d.ch2,
        _ => false,
    }
}

fn b(input: &str) -> String {
    let input_utf16: Vec<u16> = input.encode_utf16().collect();
    let input_utf16: Vec<i32> = input_utf16.iter().map(|x| *x as i32).collect();

    let i = *input_utf16.get(12).expect("There to be enough characters");
    let j = *input_utf16.get(13).expect("There to be enough characters");
    let k = *input_utf16.get(14).expect("There to be enough characters");
    let l = *input_utf16.get(15).expect("There to be enough characters");
    let mut m = 0;
    let mut n = 0;
    let mut o = 0;
    let mut p = 0;
    let mut q = String::new();

    // Just some default value to please the compiler
    let mut d = DType {
        length: 0,
        ch0: 0,
        ch1: None,
        ch2: None,
    };
    let mut e = DType {
        length: 0,
        ch0: 0,
        ch1: None,
        ch2: None,
    };
    let mut f = DType {
        length: 0,
        ch0: 0,
        ch1: None,
        ch2: None,
    };

    if i == 58 && j == 32 {
        d = DType {
            length: 1,
            ch0: 33,
            ch1: None,
            ch2: None,
        };
        e = DType {
            length: 1,
            ch0: 36,
            ch1: None,
            ch2: None,
        };
        f = DType {
            length: 1,
            ch0: 37,
            ch1: None,
            ch2: None,
        };
        o = 40;
        p = 126;
        let substrend = std::cmp::min(input_utf16.len(), 14 + 19);
        m = c(&input_utf16[14..substrend]);
        n = 33;
    } else if i == 58 && j == 58 && k == 32 {
        d = DType {
            length: 1,
            ch0: 171,
            ch1: None,
            ch2: None,
        };
        e = DType {
            length: 1,
            ch0: 169,
            ch1: None,
            ch2: None,
        };
        f = DType {
            length: 1,
            ch0: 187,
            ch1: None,
            ch2: None,
        };
        o = 40;
        p = 126;
        let substrend = std::cmp::min(input_utf16.len(), 15 + 19);
        m = c(&input_utf16[15..substrend]);
        n = 34;
    } else if i == 58 && j == 58 && k == 58 && l == 32 {
        d = DType {
            length: 3,
            ch0: 33,
            ch1: Some(36),
            ch2: Some(34),
        };
        e = DType {
            length: 3,
            ch0: 35,
            ch1: Some(37),
            ch2: Some(38),
        };
        f = DType {
            length: 3,
            ch0: 37,
            ch1: Some(35),
            ch2: Some(36),
        };
        o = 40;
        p = 126;
        let substrend = std::cmp::min(input_utf16.len(), 16 + 19);
        m = c(&input_utf16[16..substrend]);
        n = 35;
    }
    let g: i32 = input_utf16.len() as i32 - n;
    let mut h = g;
    while h > 0 {
        let v = n + h - 1;
        let r = *input_utf16.get(v as usize).expect("Can this fail?");
        let u = (g - h + 1) % 10;
        if r >= o && p >= r {
            q.push(if o > r - m - u {
                u16_to_char(r - m - u + (p - o + 1))
            } else {
                u16_to_char(r - m - u)
            });
        } else {
            let mut s = None;
            let mut t = None;
            if v >= 1 {
                s = Some(
                    *input_utf16
                        .get(v as usize - 1)
                        .expect("This to find a character"),
                );
            }
            if v >= 2 {
                t = Some(
                    *input_utf16
                        .get(v as usize - 2)
                        .expect("This to find a character"),
                );
            }
            if a(t, s, Some(r), &d) {
                q.push(u16_to_char(10));
                h = h - (d.length - 1);
            } else if a(t, s, Some(r), &e) {
                q.push(u16_to_char(13));
                h = h - (e.length - 1);
            } else if a(t, s, Some(r), &f) {
                q.push(u16_to_char(32));
                h = h - (f.length - 1);
            } else {
                q.push(u16_to_char(
                    *input_utf16
                        .get(v as usize)
                        .expect("There to be a character"),
                ));
            }
        }
        // Finish JS for loop
        h = h - 1;
    }
    q
}

fn u16_to_char(input: i32) -> char {
    std::char::decode_utf16([input as u16].iter().cloned())
        .next()
        .expect("Decoding should work") // Opens next() option
        .expect("Decoding should work") // Opens decoding result
}

fn c(a: &[i32]) -> i32 {
    let b = d(a.get(17).unwrap());
    let c = d(a.get(18).unwrap());
    let e = d(a.get(11).unwrap());
    let mut f = my_sum(&[
        d(a.get(0).unwrap()),
        d(a.get(1).unwrap()),
        d(a.get(2).unwrap()),
        d(a.get(3).unwrap()),
        d(a.get(5).unwrap()),
        d(a.get(6).unwrap()),
        d(a.get(8).unwrap()),
        d(a.get(9).unwrap()),
        e,
        d(a.get(12).unwrap()),
        d(a.get(14).unwrap()),
        d(a.get(15).unwrap()),
        b,
        c,
    ]);
    if c >= b {
        f = f.and_then(|f| c.and_then(|c| b.and_then(|b| Some(f + c - b))));
    } else {
        f = f.and_then(|f| e.and_then(|e| Some(f + (3 * (e + 1)))));
    }
    f.unwrap_or(27)
}
fn d(char_code: &i32) -> Option<i32> {
    if 48 <= *char_code && *char_code <= 57 {
        Some(char_code - 48)
    } else {
        None
    }
}
fn my_sum(args: &[Option<i32>]) -> Option<i32> {
    let mut sum = Some(0);
    for arg in args {
        if let Some(number) = arg {
            sum = sum.and_then(|s| Some(s + number));
        } else {
            sum = None;
        }
    }
    sum
}

struct DType {
    length: i32,
    ch0: i32,
    ch1: Option<i32>,
    ch2: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_to_string(path: &str) -> String {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open(path).unwrap();
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();
        buffer
    }
    #[test]
    fn test_decryption() {
        // Need to trim() what is read from the files as they contain an extra \n at the end
        // otherwise. This then screws up the decryption.
        let encrypted = read_to_string("src/encrypted.txt");
        let decrypted = read_to_string("src/decrypted.txt");
        let mydecrypted = decrypt(encrypted.trim());
        assert_eq!(decrypted.trim(), mydecrypted);
    }

    #[test]
    fn test_c() {
        let encoded: Vec<u16> = "2019-06-10 15:25:46".encode_utf16().collect();
        let encoded: Vec<i32> = encoded.iter().map(|x| *x as i32).collect();
        assert_eq!(c(&encoded), 44);
        let encoded: Vec<u16> = "2019-06-10 15:29:12".encode_utf16().collect();
        let encoded: Vec<i32> = encoded.iter().map(|x| *x as i32).collect();
        assert_eq!(c(&encoded), 40);
    }
}
