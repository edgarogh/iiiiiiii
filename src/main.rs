mod loader;
mod util;

use loader::Loader;
use rodio::{DeviceTrait, Source};
use std::io::{BufReader, Read, stdin, Cursor};
use std::time::Duration;
use crate::loader::{DirectoryLoader, Sound, ZipLoader};
use rand::seq::SliceRandom;
use std::ops::Range;
use rand::Rng;
use std::sync::{Mutex, Arc};

fn sleep(seconds: f32) {
    std::thread::sleep(Duration::from_secs_f32(seconds))
}

fn ram_usage() -> f32 {
    sys_info::mem_info().map(|sys_info::MemInfo { avail, total, .. }| {
        let used = total - avail;
        (used as f32) / (total as f32)
    }).unwrap_or(0.5)
}

const THRESHOLD: f32 = 0.8;
const FIRST_INTERVAL: Range<f32> = 2f32..10f32;
const SECOND_INTERVAL: Range<f32> = 5f32..30f32;

fn load_packed() -> ZipLoader<Cursor<&'static [u8]>> {
    println!("Loading from packed archive");
    let archive = include_bytes!("../otters.zip").as_ref();
    ZipLoader::new(Cursor::new(archive)).unwrap()
}

fn main() -> ! {
    #[cfg(not(feature = "pack"))]
    let loader = DirectoryLoader::new("src/samples");

    #[cfg(feature = "pack")]
    let mut loader = load_packed();

    let files: Vec<Sound<_>> = loader.all().unwrap();
    let loader = Arc::new(Mutex::new(loader));

    assert!(!files.is_empty(), "No files to play");

    let output = rodio::default_output_device().expect("Cannot find default input device ;(");

    let mut rng = rand::thread_rng();

    loop {
        let ram_usage = ram_usage();
        let ram_usage = if ram_usage < 0f32 {
            0f32
        } else if ram_usage > 1f32 {
            1f32
        } else {
            ram_usage
        };

        println!("ram usage: {:.2}%", ram_usage * 100f32);

        if ram_usage > THRESHOLD {
            let index = files.choose(&mut rng).unwrap().index.clone();

            println!("Start loading");
            let loader = Arc::clone(&loader);
            let source_handle = std::thread::spawn(move || {
                let ret = loader.lock().unwrap().load(&index).unwrap();
                println!("End loading");
                ret
            });

            sleep(rng.gen_range(FIRST_INTERVAL.start, FIRST_INTERVAL.end));
            println!("End sleeping");

            let sink = rodio::play_once(&output, source_handle.join().unwrap()).unwrap();
            sink.sleep_until_end();
            println!("End playing");

            let until_full = 1f32 - ((ram_usage - THRESHOLD) / (1f32 - THRESHOLD));
            let proportion = SECOND_INTERVAL.start + (until_full * (SECOND_INTERVAL.end - SECOND_INTERVAL.start));

            println!("Start 2nd interval");
            sleep(proportion);
            println!("End 2nd interval");
        }

        sleep(1f32);
    }
}
