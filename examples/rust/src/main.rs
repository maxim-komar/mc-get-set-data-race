use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use std::str;
use std::sync::Arc;
use std::thread;

#[derive(Clone, Debug, ValueEnum)]
enum GetApplySet {
    Atomic,
    Nonatomic,
}

impl std::fmt::Display for GetApplySet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Atomic => "atomic",
            Self::Nonatomic => "nonatomic",
        };
        s.fmt(f)
    }
}

impl std::str::FromStr for GetApplySet {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "atomic"    => Ok(Self::Atomic),
            "nonatomic" => Ok(Self::Nonatomic),
            _ => Err(format!("Unsupported function type: {s}")),
        }
    }
}

#[derive(Parser)]
struct Args {
    /// memcached host
    #[arg(long)]
    host: String,

    /// memcached port
    #[arg(long)]
    port: u32,

    /// memcached key
    #[arg(long)]
    key: String,

    /// number of iterations
    #[arg(long)]
    iter: usize,

    /// method to call
    #[arg(
        long,
    )]
    method: GetApplySet,
}

fn main() {
    let args = Args::parse();

    let dsn = format!("memcache://{host}:{port}", host=args.host, port=args.port);
    let conn = memcache::connect(dsn.clone()).unwrap();

    conn.delete(&args.key).unwrap();

    let method = match args.method {
        GetApplySet::Atomic    => atomic_get_apply_set,
        GetApplySet::Nonatomic => nonatomic_get_apply_set,
    };

    let concurrency = 2;
    run_concurrent(concurrency, method, dsn, &args.key, args.iter, next_str);

    let mut expected = None;
    for _ in 0..(concurrency * args.iter) {
        expected = Some(next_str(expected));
    }

    let actual: Option<String> = conn.get(&args.key).unwrap();
    println!("expected: {expected}, actual: {actual}",
             expected=expected.unwrap(),
             actual=actual.unwrap()
    );
}

fn run_concurrent(concurrency: usize, method: fn(&memcache::Client, &String, NextStrSig), dsn: String, key: &String, iter: usize, nxt: NextStrSig) {
    let dsn_shared = Arc::new(dsn);
    let key_shared = Arc::new(key);

    let threads: Vec<_> = (0..concurrency)
        .map(|_| {
            let dsn_cloned = String::clone(&dsn_shared);
            let key_cloned = String::clone(&key_shared);

            thread::spawn(move || {
                let conn = memcache::connect(dsn_cloned).unwrap();

                for _ in 0..iter {
                    method(&conn, &key_cloned, nxt);
                }
            })
        })
        .collect();

    for handle in threads {
        handle.join().unwrap();
    }
}

fn nonatomic_get_apply_set(conn: &memcache::Client, key: &String, nxt: NextStrSig) {
    let exptime = 86400;

    let curr: Option<String> = conn.get(key).unwrap();
    let next = nxt(curr);
    conn.set(key, next, exptime).unwrap();
}

fn atomic_get_apply_set(conn: &memcache::Client, key: &String, nxt: NextStrSig) {
    let exptime = 86400;

    loop {
        let get_res: HashMap<String, (Vec<u8>, u32, Option<u64>)> = conn.gets(&[&key]).unwrap();
        match get_res.get(key) {
            None => {
                match conn.add(key, nxt(None), exptime) {
                    Ok(_) => return (),
                    _ => (),
                }
            },
            Some((curr, _, Some(cas))) => {
                let curr = str::from_utf8(curr).unwrap().to_string();

                match conn.cas(key, nxt(Some(curr)), exptime, cas.clone()) {
                    Ok(true) => return (),
                    Ok(false) => (),
                    _ => (),
                }
            },
            _ => (),
        }
    }
}

fn next(v: Option<i32>) -> i32 {
    match v {
        None => 0,
        Some(i) => {
            if i % 2 == 0 {
                i + 3
            } else {
                i + 1
            }
        }
    }
}

type NextStrSig = fn(Option<String>) -> String;

fn next_str(v: Option<String>) -> String {
    let n = match v {
        None => next(None),
        Some(s) => {
            let s_int = s.parse::<i32>().unwrap();
            next(Some(s_int))
        }
    };
    n.to_string()
}
