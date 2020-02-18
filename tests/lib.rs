extern crate quandl_v3;

use quandl_v3::Result;
use quandl_v3::prelude::*;

static SKIP_CODE_LIST_QUERY: bool = true; // Necessary to pass build on travis-cl
static API_KEY: Option<&'static str> = Some("x3E2BsxsYR1V9iNuAw6m"); // quandl.tester@gmail.com

#[test]
fn database_metadata_query() {
    let query = {
        let mut query = DatabaseMetadataQuery::new("WIKI");

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let metadata = query.send();

    println!("{}", query.url());
    println!("{:?}", metadata);

    assert!(metadata.is_ok());
}

#[test]
fn dataset_metadata_query() {
    let query = {
        let mut query = DatasetMetadataQuery::new("WIKI", "AAPL");

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let metadata = query.send();

    println!("{}", query.url());
    println!("{:?}", metadata);

    assert!(metadata.is_ok());
}

#[test]
fn database_search() {
    let query = {
        let mut query = DatabaseSearch::new();

        query.query(&["Oil", "Recycling"])
             .per_page(1)
             .page(1);

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let list = query.send();

    println!("{}", query.url());
    println!("{:?}", list);

    assert!(list.is_ok());
}

#[test]
fn dataset_search() {
    let query = {
        let mut query = DatasetSearch::new("WIKI");

        query.query(&["Oil", "Recycling"])
             .per_page(1)
             .page(1);

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let list = query.send();

    println!("{}", query.url());
    println!("{:?}", list);

    assert!(list.is_ok());
}

#[test]
fn code_list_query() {
    if !SKIP_CODE_LIST_QUERY {
        let query = {
            let mut query = CodeListQuery::new("WIKI");

            if let Some(key) = API_KEY {
                query.api_key(key);
            }

            query
        };

        let list = query.send();

        println!("{}", query.url());
        println!("{:?}", list);

        assert!(list.is_ok());
    }
}

#[test]
fn data_query() {
    let query = {
        let mut query = DataQuery::new("WIKI", "AAPL");

        query.rows(20)
             .order(Order::asc)
             .collapse(Frequency::daily)
             .transform(Transform::none)
             .end_date(2016, 2, 10)
             .start_date(2016, 2, 1)
             .column_index(2);

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let data: Result<Vec<(String, f64)>> = query.send();

    println!("{}", ApiCall::<Vec<(String, f64)>>::url(&query));
    println!("{:?}", data);

    assert!(data.is_ok());
}

#[test]
fn batch_querying() {
    let query_1 = {
        let mut query = DatabaseMetadataQuery::new("WIKI");

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let query_2 = {
        let mut query = DatabaseMetadataQuery::new("FRED");

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let query_3 = {
        let mut query = DatabaseMetadataQuery::new("JODI");

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let query_4 = {
        let mut query = DatabaseMetadataQuery::new("EIA");

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    println!("{}", query_1.url());
    println!("{}", query_2.url());
    println!("{}", query_3.url());
    println!("{}", query_4.url());

    let vector: Vec<_> = {
        let mut batch_query = BatchQuery::new();

        batch_query
            .queries(&[query_1.clone(), query_2.clone(), query_3.clone(), query_4.clone()])
            .threads(1);

        batch_query.run().collect()
    };

    println!("{:?}", vector);

    assert_eq!(vector.len(), 4);

    for result in &vector {
        assert!(result.is_ok());
    }

    for i in 2..4 {
        let other_vector: Vec<_> = {
            let mut batch_query = BatchQuery::new();

            batch_query
                .queries(&[query_1.clone(), query_2.clone(), query_3.clone(), query_4.clone()])
                .threads(i);

            batch_query.run().collect()
        };

        assert_eq!(vector, other_vector);
    }
}
