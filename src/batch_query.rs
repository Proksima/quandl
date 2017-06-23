use std::collections::HashMap;

use std::thread::spawn;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex, RwLock};

use has::Has;
use serde::de::DeserializeOwned;

use Result;
use api_call::ApiCall;
use parameters::ApiArguments;

/// Builder pattern run multiple queries in batch.
///
/// The data is downloaded from the Quandl servers asynchronously. It does so by returning an
/// Iterator which return the `Result` from each individual query in the order they are fed to the
/// `query` and `queries` methods.
///
/// When batch downloading, it is important to keep Quandl's API limits in mind. Please read the
/// documentation for methods `limit` and `concurrent_calls` for more information.
///
pub struct BatchQuery<A, T>
    where T: DeserializeOwned + Clone + Sync + Send + 'static,
          A: ApiCall<T> + Clone + Sync + Send + 'static,
{
    offset: usize,
    limits: Vec<(usize, ::std::time::Duration)>,
    queries: Vec<A>,
    threads: usize,
    concurrent_calls: bool,
    marker: ::std::marker::PhantomData<T>,
}

impl<A, T> BatchQuery<A, T>
    where T: DeserializeOwned + Clone + Sync + Send + 'static,
          A: ApiCall<T> + Clone + Sync + Send + 'static,
{
    /// Construct a new (empty) BatchQuery with default state.
    ///
    pub fn new() -> Self {
        BatchQuery {
            offset: 0,
            limits: vec![],
            queries: vec![],
            threads: ::num_cpus::get(),
            concurrent_calls: false,
            marker: ::std::marker::PhantomData,
        }
    }

    /// Assume that every API keys has already been used the specified number of times.
    ///
    /// This is an hackish way to send a big batch query underway immediately even if some API keys
    /// have been used a small number of times (e.g. for testing purposes). This is obviously
    /// suboptimal and a future version of this library might provide stateful API keys which hold
    /// information about their usage and can be saved/retrieved from disk to always be used as
    /// efficiently as possible.
    ///
    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }

    /// Specify some download rate limits for this batch query.
    ///
    /// From the beginning of 2017, some heavy restrictions applies to non-premium API keys,
    /// namely:
    ///
    /// * Up to 300 API calls can be made with a single key within 10 seconds.
    /// * Up to 2,000 API calls can be made with a single key within 10 minutes (600 seconds).
    /// * Up to 50,000 API calls can be made with a single key within a day (86,400 seconds).
    ///
    /// For premium keys the limits are as follow:
    ///
    /// * Up to 5,000 API calls can be made with a single key within 10 minutes (600 seconds).
    /// * Up to 720,000 API calls can be made with a single key within a day (86,400 seconds).
    ///
    /// Not using an API key at all yield the most restrictive access:
    ///
    /// * Up to 20 API calls by 10 minutes (600 seconds).
    /// * Up to 50 API calls within a day (86,400 seconds).
    ///
    /// This method allow to specify those limits in a future-proof fashion (the limits could
    /// change at any time and this library does not handle it for this reason).
    ///
    /// For example, if not using any key, you would use this method as follow:
    ///
    /// ```rust
    /// extern crate quandl_v3;
    ///
    /// use quandl_v3::Result;
    /// use quandl_v3::prelude::*;
    ///
    /// fn main() {
    ///     let mut batch_query = BatchQuery::new();
    ///
    ///     batch_query
    ///         .query(DatabaseMetadataQuery::new("ICE"))
    ///         .query(DatabaseMetadataQuery::new("WIKI"))
    ///         .limit(20, 600)
    ///         .limit(50, 86_400);
    ///
    ///     let result: Vec<_> = batch_query.run().collect();
    /// }
    /// ```
    ///
    pub fn limit(&mut self, limit: usize, timeout: u64) -> &mut Self {
        self.limits.push((limit, ::std::time::Duration::new(timeout, 0)));
        self
    }

    /// Add a single query to this batch.
    ///
    pub fn query(&mut self, query: A) -> &mut Self {
        self.queries.push(query);
        self
    }

    /// Add a slice of queries to this batch.
    ///
    pub fn queries(&mut self, queries: &[A]) -> &mut Self {
        self.queries.extend_from_slice(queries);
        self
    }

    /// Specify the maximum number of threads to use.
    ///
    /// By default the number of logical cores is used. The number of threads specified must be
    /// bigger than 0.
    ///
    pub fn threads(&mut self, threads: usize) -> &mut Self {
        assert!(threads > 0, "threads: {}", threads);
        self.threads = threads;
        self
    }

    /// Whether to allow concurrent calls to the API with a single key.
    ///
    /// This usage of the Quandl API is forbidden for non-premium keys but allowed for premium
    /// users. Thus this method should only be called on your batch if you are strictly using
    /// premium keys.
    ///
    pub fn concurrent_calls(&mut self) -> &mut Self {
        self.concurrent_calls = true;
        self
    }

    /// Execute the batch query and return an iterator which asynchronously fetch the data.
    ///
    pub fn run(self) -> Iterator<Result<T>> {
        let keys = Arc::new(RwLock::new(HashMap::<String, Mutex<usize>>::new()));

        for query in self.queries.iter() {
            if let Some(ref key) = Has::<ApiArguments>::get_ref(query).api_key {
                if !keys.read().unwrap().contains_key(&key[..]) {
                    keys.write().unwrap().insert(key.clone(), Mutex::new(self.offset));
                }
            }
        }

        let mut jobs: Vec<Vec<A>> = vec![];

        for _ in 0..self.threads {
            jobs.push(vec![]);
        }

        for (index, api_call) in self.queries.iter().enumerate() {
            jobs[index % self.threads].push(api_call.clone());
        }

        let mut iterator = {
            Iterator {
                index: 0,
                channels: vec![],
            }
        };

        let batch_query = Arc::new(self);

        for api_queries in jobs {
            if !api_queries.is_empty() {
                let keys = keys.clone();
                let (tx, rx) = channel();

                iterator.channels.push(rx);

                let batch_query = batch_query.clone();

                spawn(move || {
                    for api_call in api_queries {
                        if let Some(ref key) = Has::<ApiArguments>::get_ref(&api_call).api_key {
                            if batch_query.concurrent_calls {
                                {
                                    let keys = keys.read().unwrap();

                                    let mut calls = {
                                        keys.get(&key[..]).expect("Key not found")
                                            .lock().expect("Poisoned Mutex")
                                    };

                                    for &(limit, ref duration) in batch_query.limits.iter() {
                                        if *calls != 0 && *calls % limit == 0 {
                                            ::std::thread::sleep(duration.clone());
                                        }
                                    }

                                    *calls += 1;
                                }

                                if let Err(_) = tx.send(api_call.send()) {
                                    panic!("Thread's communication channel closed prematurely.");
                                }
                            } else {
                                let keys = keys.read().unwrap();

                                let mut calls = {
                                    keys.get(&key[..]).expect("Key not found")
                                        .lock().expect("Poisoned Mutex")
                                };

                                for &(limit, ref duration) in batch_query.limits.iter() {
                                    if *calls != 0 && *calls % limit == 0 {
                                        ::std::thread::sleep(duration.clone());
                                    }
                                }

                                *calls += 1;

                                if let Err(_) = tx.send(api_call.send()) {
                                    panic!("Thread's communication channel closed prematurely.");
                                }
                            }
                        }
                    }
                });
            }
        }

        iterator
    }
}

/// Iterator returned by the `BatchQuery::run` method.
///
/// See the `BatchQuery` struct documentation for more information.
///
pub struct Iterator<T> {
    index: usize,
    channels: Vec<Receiver<T>>,
}

impl<T: Sync + Send + 'static> Iterator<T> {
    /// Check if the next `Result` value is ready in a non blocking way.
    ///
    /// If the value is not yet avaiable, `Some(None)` is returned. If the iterator is over, `None`
    /// is returned. Otherwise, `Some(Result)` is to be expected.
    ///
    /// Note that the implementation of the `Iterator` trait is done by calling this function in
    /// the `next` implementation and yielding whether this function returns `Some(None)`.
    ///
    pub fn try_next(&mut self) -> Option<Option<T>> {
        loop {
            if self.channels.is_empty() {
                return None;
            } else {
                match self.channels[self.index].try_recv() {
                    Ok(item) => {
                        self.index = (self.index + 1) % self.channels.len();
                        return Some(Some(item));
                    },

                    Err(TryRecvError::Disconnected) => {
                        self.channels.truncate(self.index);

                        if self.channels.is_empty() {
                            return None;
                        } else {
                            self.index = 0;
                        }
                    },

                    Err(TryRecvError::Empty) => return Some(None),
                }
            }
        }
    }
}

impl<T: Sync + Send + 'static> ::std::iter::Iterator for Iterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.try_next() {
                Some(Some(item)) => return Some(item),
                Some(None) => ::std::thread::yield_now(),
                None => return None,
            }
        }
    }
}
