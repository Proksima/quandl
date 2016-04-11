use has::*;

use types::{Order, Frequency, Transform};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ApiArguments {
    api_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SearchArguments {
    keywords: Vec<String>,
    per_page: Option<usize>,
    page: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataArguments {
    rows: Option<usize>,
    limit: Option<usize>,
    order: Option<Order>,
    collapse: Option<Frequency>,
    transform: Option<Transform>,
    end_date: Option<(u16, u8, u8)>,
    start_date: Option<(u16, u8, u8)>,
    column_index: Option<usize>,
}

/// Api parameters implemented by all queries.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#api-keys)
///
pub trait ApiParameters: HasMut<ApiArguments> {
    /// Include your personal Quandl API key with your query.
    ///
    fn api_key<S: AsRef<str>>(&mut self, api_key: S) -> &mut Self {
        HasMut::<ApiArguments>::get_mut(self).api_key = Some(api_key.as_ref().to_string());
        self
    }

    /// Return a string which will be appended to the query's URL given that an api key has been
    /// provided.
    ///
    fn fmt(&self) -> Option<String> {
        if let Some(ref key) = Has::<ApiArguments>::get_ref(self).api_key {
            Some(format!("api_key={}", key))
        } else {
            None
        }
    }
}

/// Search parameters implemented by search queries.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#database-search)
///
pub trait SearchParameters: HasMut<SearchArguments> {
    /// Specify a vector/list of search keywords to retrieve only database/dataset related to those
    /// search terms.
    ///
    fn query<V: AsRef<[S]>, S: AsRef<str>>(&mut self, keywords: V) -> &mut Self {
        HasMut::<SearchArguments>::get_mut(self).keywords = {
            keywords.as_ref().iter().map(|x| x.as_ref().trim().to_string()).collect()
        };

        self
    }

    /// Specify how many entries should be returned by search query.
    ///
    fn per_page(&mut self, n: usize) -> &mut Self {
        HasMut::<SearchArguments>::get_mut(self).per_page = Some(n);
        self
    }

    /// Given there is more than one page of entries to be returned, specify which page we want to
    /// query.
    ///
    fn page(&mut self, n: usize) -> &mut Self {
        HasMut::<SearchArguments>::get_mut(self).page = Some(n);
        self
    }

    /// Return a string which will be appended to the query's URL given that at least one of the
    /// search parameters has been specified.
    ///
    fn fmt(&self) -> Option<String> {
        let mut fmt = String::new();

        let arguments = Has::<SearchArguments>::get_ref(self);

        if !arguments.keywords.is_empty() {
            fmt.push_str(&format!("query={}", arguments.keywords[0]));

            for keyword in arguments.keywords.iter().skip(1) {
                fmt.push('+');
                fmt.push_str(&keyword[..]);
            }

            fmt.push('&');
        }

        if let Some(n) = arguments.per_page {
            fmt.push_str(&format!("per_page={}&", n));
        }

        if let Some(n) = arguments.page {
            fmt.push_str(&format!("page={}&", n));
        }

        if fmt.pop().is_some() {
            Some(fmt)
        } else {
            None
        }
    }
}

/// Data parameters implemented by data fetching queries.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#data)
///
pub trait DataParameters: HasMut<DataArguments> {
    /// Specify the number of rows of data to be returned by this query.
    ///
    /// Note that this is identical to the `limit` parameter.
    ///
    fn rows(&mut self, n: usize) -> &mut Self {
        HasMut::<DataArguments>::get_mut(self).rows = Some(n);
        self
    }

    /// Specify the number of rows of data to be returned by this query.
    ///
    /// Note that this is identical to the `rows` parameter.
    ///
    fn limit(&mut self, n: usize) -> &mut Self {
        HasMut::<DataArguments>::get_mut(self).limit = Some(n);
        self
    }

    /// Specify the ordering of the data.
    ///
    /// More specifically, it can be precised whether the data should be returned with dates in an
    /// ascending (`Order::asc`) or descending (`Order::desc`) order.
    ///
    fn order(&mut self, order: Order) -> &mut Self {
        HasMut::<DataArguments>::get_mut(self).order = Some(order);
        self
    }

    /// Specify whether the data should be returned at a smaller frequency than avaiable.
    ///
    fn collapse(&mut self, collapse: Frequency) -> &mut Self {
        HasMut::<DataArguments>::get_mut(self).collapse = Some(collapse);
        self
    }

    /// Specify how the data should be transformed by Quandl's server before being returned.
    ///
    fn transform(&mut self, transform: Transform) -> &mut Self {
        HasMut::<DataArguments>::get_mut(self).transform = Some(transform);
        self
    }

    /// Specify the oldest data point to be returned.
    ///
    /// Note that if the date makes no sense, the error will be reported by the Quandl server
    /// (wasting one api call in the process).
    ///
    fn end_date(&mut self, year: u16, month: u8, day: u8) -> &mut Self {
        HasMut::<DataArguments>::get_mut(self).end_date = Some((year, month, day));
        self
    }

    /// Specify the earliest data point to be returned.
    ///
    /// Note that if the date makes no sense, the error will be reported by the Quandl server
    /// (wasting one api call in the process).
    ///
    fn start_date(&mut self, year: u16, month: u8, day: u8) -> &mut Self {
        HasMut::<DataArguments>::get_mut(self).start_date = Some((year, month, day));
        self
    }

    /// Specify which column to be returned.
    ///
    /// Note that the column 0, i.e. the 'date' column, is always returned.
    ///
    fn column_index(&mut self, index: usize) -> &mut Self {
        HasMut::<DataArguments>::get_mut(self).column_index = Some(index);
        self
    }

    /// Return a string which will be appended to the query's URL given that at least one of the
    /// data parameters has been specified.
    ///
    fn fmt(&self) -> Option<String> {
        let mut fmt = String::new();

        let arguments = Has::<DataArguments>::get_ref(self);

        if let Some(n) = arguments.rows {
            fmt.push_str(&format!("rows={}&", n)[..]);
        }

        if let Some(n) = arguments.limit {
            fmt.push_str(&format!("limit={}&", n)[..]);
        }

        if let Some(order) = arguments.order {
            fmt.push_str(&format!("order={:?}&", order)[..]);
        }

        if let Some(collapse) = arguments.collapse {
            fmt.push_str(&format!("collapse={:?}&", collapse)[..]);
        }

        if let Some(transform) = arguments.transform {
            fmt.push_str(&format!("transform={:?}&", transform)[..]);
        }

        if let Some((year, month, day)) = arguments.end_date {
            fmt.push_str(&format!("end_date={:#04}-{:#02}-{:#02}&", year, month, day));
        }

        if let Some((year, month, day)) = arguments.start_date {
            fmt.push_str(&format!("start_date={:#04}-{:#02}-{:#02}&", year, month, day));
        }

        if let Some(index) = arguments.column_index {
            fmt.push_str(&format!("column_index={}&", index)[..]);
        }

        if fmt.pop().is_some() {
            Some(fmt)
        } else {
            None
        }
    }
}
