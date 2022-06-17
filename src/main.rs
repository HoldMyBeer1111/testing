use std::env;

use anyhow::anyhow;

mod task4;
mod task_1_and_2;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        eprintln!("USAGE: testing <dir> <ext>");
        return Err(anyhow!("invalid usage"));
    }
    task4::search_files(&args[1], &args[2])
}
