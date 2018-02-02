// Copyright 2016 Mozilla Foundation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use cache::{
    Cache,
    CacheRead,
    CacheWrite,
    Storage,
};
use futures::future::ok;
use errors::*;
use std::collections::HashMap;
use std::time::{
    Duration,
    Instant,
};
use std::sync::{Arc, Mutex};
use std::io::Cursor;


pub struct InMemCacheStore {
    cache: HashMap<String, Vec<u8>>,
    current_size_bytes: u64,
}

impl InMemCacheStore {
    fn new() -> InMemCacheStore {
        InMemCacheStore {
            cache: HashMap::new(),
            current_size_bytes: 0,
        }
    }
}

/// A cache that stores entries in memory
pub struct InMemCache {
    max_size_bytes: Option<u64>,
    store: Arc<Mutex<InMemCacheStore>>,
}

impl InMemCache {
    pub fn new(max_size_bytes:Option<u64>) -> InMemCache {
        InMemCache {
            max_size_bytes: max_size_bytes,
            store: Arc::new(Mutex::new(InMemCacheStore::new())),
        }
    }
}

impl Storage for InMemCache {
    fn get(&self, key: &str) -> SFuture<Cache> {
        let store = self.store.lock().unwrap();
        Box::new(ok(match store.cache.get(key) {
            Some(entry) =>
                CacheRead::from(Cursor::new(entry.to_vec())).map(Cache::Hit).unwrap(),
            None => Cache::Miss
        }))
    }

    fn put(&self, key: &str, entry: CacheWrite) -> SFuture<Duration> {
        let start = Instant::now();
        let mut store = self.store.lock().unwrap();
        let blob = entry.finish().unwrap();
        store.current_size_bytes += blob.len() as u64;
        store.cache.insert(key.to_owned(), blob);
        Box::new(ok(start.elapsed()))
    }

    fn location(&self) -> String {
        "memory".to_owned()
    }

    fn current_size(&self) -> Option<u64> {
        let store = self.store.lock().unwrap();
        Some(store.current_size_bytes)
    }

    fn max_size(&self) -> Option<u64> {
        self.max_size_bytes
    }
}
