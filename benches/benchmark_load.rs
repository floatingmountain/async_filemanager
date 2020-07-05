use criterion::{black_box, criterion_group, criterion_main, Criterion};

use futures::stream::futures_unordered::FuturesUnordered;
use futures::stream::StreamExt;
use futures::FutureExt;
use std::sync::Arc;
use futures::executor::{  ThreadPoolBuilder};

use async_filemanager::AsyncFileManager;
use async_filemanager::LoadStatus;
use std::{convert::TryFrom, path::PathBuf};
#[derive(Debug, Eq, PartialEq)]
struct LoadedFile {
    string: String,
}

impl TryFrom<(PathBuf, Vec<u8>)> for LoadedFile {
    type Error = std::string::FromUtf8Error;
    fn try_from((_path, bytes): (PathBuf, Vec<u8>)) -> Result<Self, Self::Error> {
        Ok(LoadedFile {
            string: String::from_utf8(bytes)?,
        })
    }
}
async fn load_custom(f: &[&str], manager: &mut AsyncFileManager<LoadedFile>) {
    let mut fut = FuturesUnordered::new();
    let mut l = Vec::new();
    for file in f.iter() {
        let mut path = String::from("benches/benchfiles/");
        path.push_str(file);
        manager.load(&path).await;
        match manager.get(&path).await{
            LoadStatus::Loading(f) => fut.push(f),
            LoadStatus::Loaded(f) => l.push(f),
            _ => panic!(),
        }
    }

    let mut vec = Vec::new();
    while let Some(val) = fut.next().await{
        vec.push(val.unwrap());
    }
    black_box(vec);
    black_box(l);
}
async fn load_async(f: &[&str]) {
    let mut u = FuturesUnordered::new();
    for file in f.iter() {
        let mut path = PathBuf::from("benches/benchfiles/");
        path.push(file);
        let l = async_std::fs::read(path.clone()).map(|f| (path,f.unwrap())).shared();
        u.push(l);
    }
    let mut vec = Vec::new();
    while let Some((p,v)) = u.next().await {
        vec.push(LoadedFile::try_from((p,v)).unwrap());
    }
    black_box(vec);
}
fn load_sync(f: &[&str]){
    let mut vec = Vec::new();
    for file in f.iter() {
        let mut path = PathBuf::from("benches/benchfiles/");
        path.push(file);
        let l = std::fs::read(path.clone()).unwrap();
        vec.push(LoadedFile::try_from((path,l)).unwrap());
    }
    black_box(vec);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("own", |b| {
        let pool = Arc::new(ThreadPoolBuilder::new().pool_size(8).create().unwrap());

        b.iter(|| {
            let pool = pool.clone();
            let mut manager = AsyncFileManager::new(pool);

            async_std::task::block_on(async {
                load_custom(
                    black_box(&[
                        "l01", "l02", "l03", "l04", "l05", "l06", "l07", "l08", "s01", "s02",
                        "s03", "s04", "s05", "s06", "s07", "s08", "s09", "s10", "s11", "s12",
                        "s13", "s14", "s15", "s16",
                    ]),
                    &mut manager,
                )
                .await
            })
        })
    });
    c.bench_function("async-std", |b| {
        b.iter(|| {
            async_std::task::block_on(async {
                load_async(black_box(&[
                    "l01", "l02", "l03", "l04", "l05", "l06", "l07", "l08", "s01", "s02", "s03",
                    "s04", "s05", "s06", "s07", "s08", "s09", "s10", "s11", "s12", "s13", "s14",
                    "s15", "s16",
                ]))
                .await
            })
        })
    });
    c.bench_function("sync", |b| {
        b.iter(|| {
            load_sync(black_box(&[
                "l01", "l02", "l03", "l04", "l05", "l06", "l07", "l08", "s01", "s02", "s03", "s04",
                "s05", "s06", "s07", "s08", "s09", "s10", "s11", "s12", "s13", "s14", "s15", "s16",
            ]))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
