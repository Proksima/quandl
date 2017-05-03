use std::collections::HashMap;

use std::thread::spawn;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex, RwLock};

use has::Has;
use serde::Deserialize;

use Result;
use api_call::ApiCall;
use parameters::ApiArguments;

/// Iterator returned by the `batch_query` function.
///
/// See the `batch_query` function's documentation for more information.
///
pub struct Iterator<T> {
    index: usize,
    channels: Vec<Receiver<T>>,
}

/// Submit multiple queries at the same time.
///
/// A slice/vector of queries is given as argument together with the number of threads that should
/// be used. The number of threads will be truncated to the range `[1, queries.as_ref().len()]`.
///
/// This function download the data from the Quandl servers asynchronously. It does so by returning
/// an Iterator which return the `Result` from each individual query in the order they appear in
/// the `queries` argument.
///
/// This function has been updated on the '2017-01-08' to handle the limits imposed by Quandl on
/// their API. More specifically:
///
/// * After 300 API calls with a single API key, the routine will stop using that specific key for
///   10 seconds. After 2,000 API calls, the key is put on hold for an additional 10 minutes.
///   Finally, after 50,000 API calls, the waiting time increase to 24 hours.
///
/// * The routine do not make simultaneous calls using a single API key anymore as it is forbidden.
///   Different keys are still used concurrently however.
///
/// You might want to look into the `batch_query_premium` function which use the premium limits
/// instead and allow concurrency on a single key. Also, the `batch_query_with_offset` and
/// `batch_query_premium_with_offset` takes an additional argument in the event the keys have been
/// used for other tasks before, to not go over the Quandl's limit.
///
pub fn batch_query<T, B, C>(queries: B, threads: usize) -> Iterator<Result<T>>
    where T: Deserialize + Clone + Send + 'static,
          C: ApiCall<T> + Clone + Send + 'static,
          B: AsRef<[C]>,
{
    batch_query_with_offset(queries, threads, 0)
}

/// Submit multiple queries at the same time, using premium API keys.
///
/// A slice/vector of queries is given as argument together with the number of threads that should
/// be used. The number of threads will be truncated to the range `[1, queries.as_ref().len()]`.
///
/// This function download the data from the Quandl servers asynchronously. It does so by returning
/// an Iterator which return the `Result` from each individual query in the order they appear in
/// the `queries` argument.
///
/// This function has been updated on the '2017-01-08' to handle the limits imposed by Quandl on
/// their API. More specifically:
///
/// * After 5,000 API calls with a single API key, the routine will stop using that specific key
///   for 10 minutes. After 720,000 API calls, the key is put on hold for 24 hours instead.
///
pub fn batch_query_premium<T, B, C>(queries: B, threads: usize) -> Iterator<Result<T>>
    where T: Deserialize + Clone + Send + 'static,
          C: ApiCall<T> + Clone + Send + 'static,
          B: AsRef<[C]>,
{
    batch_query_premium_with_offset(queries, threads, 0)
}

/// Submit multiple queries at the same time.
///
/// Identical to the `batch_query` function, but takes an extra argument specifying how many calls
/// has already been made with every key. The purpose is simply to avoid going over the limit when
/// batch processing (e.g. making 301 calls in less than 10 seconds).
///
pub fn batch_query_with_offset<T, B, C>(queries: B, threads: usize, calls_offset: usize)
    -> Iterator<Result<T>>

    where T: Deserialize + Clone + Send + 'static,
          C: ApiCall<T> + Clone + Send + 'static,
          B: AsRef<[C]>,
{
    lazy_static! {
        static ref LIMITS: Vec<(usize, ::std::time::Duration)> = vec![
            (300, ::std::time::Duration::new(10, 0)),
            (2_000, ::std::time::Duration::new(600, 0)),
            (50_000, ::std::time::Duration::new(86_400, 0)),
        ];
    }

    batch_query_implementation(queries, threads, &*LIMITS, calls_offset, false)
}

/// Submit multiple queries at the same time, using premium API keys.
///
/// Identical to the `batch_query_premium` function, but takes an extra argument specifying how
/// many calls has already been made with every key. The purpose is simply to avoid going over the
/// limit when batch processing (e.g. making 5001 calls in less than 10 minutes).
///
pub fn batch_query_premium_with_offset<T, B, C>(queries: B, threads: usize, calls_offset: usize)
    -> Iterator<Result<T>>

    where T: Deserialize + Clone + Send + 'static,
          C: ApiCall<T> + Clone + Send + 'static,
          B: AsRef<[C]>,
{
    lazy_static! {
        static ref LIMITS: Vec<(usize, ::std::time::Duration)> = vec![
            (5_000, ::std::time::Duration::new(600, 0)),
            (720_000, ::std::time::Duration::new(86_400, 0)),
        ];
    }

    batch_query_implementation(queries, threads, &*LIMITS, calls_offset, true)
}

fn batch_query_implementation<T, B, C>(queries: B,
                                       threads: usize,
                                       limits: &'static Vec<(usize, ::std::time::Duration)>,
                                       calls: usize,
                                       concurrent_calls: bool)

    -> Iterator<Result<T>>

    where T: Deserialize + Clone + Send + 'static,
          C: ApiCall<T> + Clone + Send + 'static,
          B: AsRef<[C]>,
{
    let threads = ::std::cmp::max(1, threads);
    let keys = Arc::new(RwLock::new(HashMap::<String, Mutex<usize>>::new()));

    for query in queries.as_ref().iter() {
        if let Some(ref key) = Has::<ApiArguments>::get_ref(query).api_key {
            if !keys.read().unwrap().contains_key(&key[..]) {
                keys.write().unwrap().insert(key.clone(), Mutex::new(calls));
            }
        }
    }

    let mut jobs: Vec<Vec<C>> = vec![];

    for _ in 0..threads {
        jobs.push(vec![]);
    }

    for (index, api_call) in queries.as_ref().iter().enumerate() {
        jobs[index % threads].push(api_call.clone());
    }

    let mut iterator = {
        Iterator {
            index: 0,
            channels: vec![],
        }
    };

    for api_queries in jobs {
        if !api_queries.is_empty() {
            let keys = keys.clone();
            let (tx, rx) = channel();

            iterator.channels.push(rx);

            spawn(move || {
                for api_call in api_queries {
                    if let Some(ref key) = Has::<ApiArguments>::get_ref(&api_call).api_key {
                        if concurrent_calls {
                            {
                                let keys = keys.read().unwrap();

                                let mut calls = {
                                    keys.get(&key[..]).expect("Key not found")
                                        .lock().expect("Poisoned Mutex")
                                };

                                for &(limit, ref duration) in limits.iter() {
                                    if *calls != 0 && *calls % limit == 0 {
                                        ::std::thread::sleep(duration.clone());
                                    }
                                }

                                *calls += 1;
                            }

                            if let Err(_) = tx.send(api_call.send()) {
                                panic!("Inter-threads communication channel closed prematurely.");
                            }
                        } else {
                            let keys = keys.read().unwrap();

                            let mut calls = {
                                keys.get(&key[..]).expect("Key not found")
                                    .lock().expect("Poisoned Mutex")
                            };

                            for &(limit, ref duration) in limits.iter() {
                                if *calls != 0 && *calls % limit == 0 {
                                    ::std::thread::sleep(duration.clone());
                                }
                            }

                            *calls += 1;

                            if let Err(_) = tx.send(api_call.send()) {
                                panic!("Inter-threads communication channel closed prematurely.");
                            }
                        }
                    }
                }
            });
        }
    }

    iterator
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
