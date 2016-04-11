use std::thread::spawn;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use rustc_serialize::Decodable;

use Result;
use api_call::ApiCall;

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
pub fn batch_query<T, B, C>(queries: B, threads: usize) -> Iterator<Result<T>>
    where T: Decodable + Clone + Send + 'static,
          C: ApiCall<T> + Clone + Send + 'static,
          B: AsRef<[C]>,
{
    let threads = {
        if threads == 0 {
            1
        } else {
            threads
        }
    };

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
            let (tx, rx) = channel();
            iterator.channels.push(rx);

            spawn(move || {
                for api_call in api_queries {
                    if let Err(_) = tx.send(api_call.send()) {
                        panic!("Inter-threads communication channel closed prematurely.");
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
