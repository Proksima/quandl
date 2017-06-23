## Rust bindings for Quandl v3 API.

The goal of this crate is to offer a well documented, complete and easy to use interface to
Quandl's RESTful API.

[![Crates.io](http://meritbadge.herokuapp.com/quandl-v3)](https://crates.io/crates/quandl-v3)
[![License: MPL 2.0](https://img.shields.io/badge/License-MPL%202.0-brightgreen.svg)](https://opensource.org/licenses/MPL-2.0)
[![Travis Build Status](https://travis-ci.org/Proksima/quandl.svg?branch=master)](https://travis-ci.org/Proksima/quandl)
[![Documentation](https://img.shields.io/badge/docs-latest-C9893D.svg)](http://proksima.github.io/quandl-v3-doc/quandl-v3/index.html)

This crate uses the `rustc_serialize` crate extensively and thus suffers from some of its
limitation. Namely:

* When querying for the metadata of a dataset, the field `type` will be missing. This is due to
  `type` being a keyword in Rust. Use of this crate assumes knowledge of the layout of the queried
  data, so that field was not very important fortunately.

* Most public enum's variants have non camel case names to match the naming convention of the API.
  The deserializer need the names to match to work properly, thus you will see `Order::asc` instead
  of the more readable `Order::Ascending`.

Some other design choices of this crate includes:

* No runtime checking of the query created. This crate makes it as hard as statically possible to
  create an invalid query. However, the query will be checked by the Quandl API directly. On the
  bright side, we forward Quandl's error messages/codes without pruning any information; and their
  error-reporting is very good.

* The inclusion of a `batch_query` function that allows users to submit a bunch of query at the
  same time. The function returns an iterator which gives the benefit of multithreading downloads
  and asynchronicity which are indispensable when doing data mining.

* We use the JSON Quandl API for everything but data queries as it often returns more information.
  When it comes to the data queries we use the CSV subset of the API as it is faster and allows to
  use the `rust-csv` crates which allow you to define your own structs to receive the data.

### Wish list / TODO

* Adding support for stateful API keys which remember their usage count, have their own individual
  limits and can be persisted/recovered from disk between usage.

* Split the keys from the queries so batch queries can automatically manage the pool of keys in the
  most efficient way possible.

* Refactor BatchQuery::run (I know it works for using it for so long before publishing this long
  overdue update, but it is not my proudest piece of code).

### Simple example

```rust
extern crate quandl_v3;

use quandl_v3::Result;
use quandl_v3::prelude::*;

fn main() {
    let query = {
        let mut query = DataQuery::new("WIKI", "AAPL");

         query.order(Order::asc)
              .end_date(2016, 2, 29)
              .start_date(2016, 2, 1)
              .column_index(4);

         query
    };

    let response: Vec<(String, f64)> = query.send().unwrap();

    // Print the date and closing price for Apple's stock for the month of February 2016.
    for data in &response {
        println!("{} - {}", data.0, data.1);
    }
}
```

This crate is written in the hope it will be useful. I am in no way affiliated to Quandl and
Quandl is not endorsing this crate in any way.

