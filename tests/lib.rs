extern crate quandl_v3;
extern crate rustc_serialize;

use quandl_v3::Result;
use quandl_v3::prelude::*;

static API_KEY: Option<&'static str> = None;

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

    let data: Result<Data<(String, f64)>> = query.send();

    println!("{}", ApiCall::<Data<(String, f64)>>::url(&query));
    println!("{:?}", data);

    assert!(data.is_ok());
}

#[test]
fn data_and_metadata_query() {
    let query = {
        let mut query = DataAndMetadataQuery::new("WIKI", "AAPL");

        query.rows(20)
             .transform(Transform::none)
             .end_date(2016, 2, 10)
             .start_date(2016, 2, 1)
             .column_index(5);

        if let Some(key) = API_KEY {
            query.api_key(key);
        }

        query
    };

    let data_and_metadata: Result<DataAndMetadata<(String, f64)>> = query.send();

    println!("{}", ApiCall::<DataAndMetadata<(String, f64)>>::url(&query));
    println!("{:?}", data_and_metadata);

    assert!(data_and_metadata.is_ok());
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
        let mut query = DatabaseMetadataQuery::new("ICE");

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
        batch_query(&[query_1.clone(), query_2.clone(), query_3.clone(), query_4.clone()],
                    1).collect()
    };

    println!("{:?}", vector);

    assert_eq!(vector.len(), 4);

    for result in &vector {
        assert!(result.is_ok());
    }

    for i in 2..4 {
        assert_eq!(&vector,
                   &batch_query(&[query_1.clone(),
                                  query_2.clone(),
                                  query_3.clone(),
                                  query_4.clone()], i).collect::<Vec<_>>());
    }
}
