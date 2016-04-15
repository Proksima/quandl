/// Parameters to indicate the desired frequency. When you change the frequency of a dataset,
/// Quandl returns the last observation for the given period.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#data)
///
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Frequency {
    /// Unspecified frequency. In a data query, will default to the frequency of the dataset.
    ///
    none,

    /// Frequency of one data point every day.
    ///
    daily,

    /// Frequency of one data point every week.
    ///
    weekly,

    /// Frequency of one data point every month.
    ///
    monthly,

    /// Frequency of one data point every 4 months (or 4 times a year).
    ///
    quarterly,

    /// Frequency of one data point every year.
    ///
    annual
}

/// Select the sort order with this enum. The default sort order is descending.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#data)
///
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Order {
    /// Ascending ordering, for time series this means the first entry is the earliest date.
    ///
    asc,

    /// Descending ordering, for time series this means the first entry if the latest date.
    ///
    desc,
}

/// Perform calculations on your data prior to downloading.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#data)
///
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum Transform {
    /// No transformation, also the default.
    ///
    none,

    /// Row-on-row change; a parameter that will transform the data to show the difference between
    /// days. Equivalent to `y'[t] = y[t] - y[t - 1]`.
    ///
    diff,

    /// Row-on-row percentage change; a parameter that will transform the data to show the
    /// difference between days divided by the previous day. Equivalent to `y'[t] = (y[t] - y[t -
    /// 1]) / y[t - 1]`.
    ///
    rdiff,

    /// Row-on-row percentage change from latest value; a parameter that will transfrom the data to
    /// show the percentage difference between the latest value and all subsequent values (where
    /// `y[n]` is the latest observation). Equivalent to `y'[t] = (y[n] - y[t]) / y[t]`.
    ///
    rdiff_from,

    /// Cumulative sum; a parameter that will calculate the sum of all preceding data returned.
    /// Equivalent to `y'[t] = y[t] + y[t - 1] + ... + y[0]`.
    ///
    cumul,

    /// Start at 100; a parameter that will normalize the data to the oldest datapoint returned.
    /// Equivalent to `y'[t] = (y[t] / y[0]) * 100`.
    ///
    normalize,
}

/// Hold the metadata associated to a specific database.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#database-metadata)
///
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct DatabaseMetadata {
    /// Quandl's numerical identifier for this database.
    ///
    pub id: usize,

    /// Name of the database.
    ///
    pub name: String,

    /// Database code; it is the code needed to construct any query on a specific database.
    ///
    pub database_code: String,

    /// Description of the database.
    ///
    pub description: String,

    /// Number of datasets in the database.
    ///
    pub datasets_count: usize,

    /// Number of time the database's content was downloaded.
    ///
    pub downloads: usize,

    /// Whether or not this is a premium database.
    ///
    pub premium: bool,

    /// URL pointing to the logo of the database.
    ///
    pub image: String,
}

/// Hold the metadata associated to a specific dataset.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#metadata)
///
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct DatasetMetadata {
    /// Quandl's numerical identifier for this dataset.
    ///
    pub id: usize,

    /// The dataset code for the returned dataset.
    ///
    /// [Quandl API Reference](https://www.quandl.com/docs/api#quandl-codes)
    ///
    pub dataset_code: String,

    /// The code for the database this dataset belongs to.
    ///
    /// [Quandl API Reference](https://www.quandl.com/docs/api#quandl-codes)
    ///
    pub database_code: String,

    /// The title of this dataset.
    ///
    pub name: String,

    /// An explanation of the contents of the data in this dataset.
    ///
    pub description: String,

    /// The last time the data in this dataset and metadata of this dataset was refreshed.
    ///
    pub refreshed_at: String,

    /// The most recent date of all available data points in this dataset.
    ///
    pub newest_available_date: String,

    /// The earliest date of all available data points in this dataset.
    ///
    pub oldest_available_date: String,

    /// The titles for each column of data in this datset.
    ///
    pub column_names: Vec<String>,

    /// How often each data point in the resulting dataset is returned.
    ///
    pub frequency: Frequency,

    /// Whether or not this is a dataset from a premium database.
    ///
    pub premium: bool,

    /// Quandl's numerical identifier for the database containing this dataset.
    ///
    pub database_id: usize,
}

/// Some queries, namely those which list datasets or databases metadata, often return some
/// metadata about the search itself. This is a structure to hold that metadata.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#dataset-search)
///
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct SearchMetadata {
    /// A string of the search keywords submitted formatted as `format!("{}+{}+...+{}", keyword_1,
    /// keyword_2, ..., keyword_n)`.
    ///
    pub query: String,

    /// The number of search result per page.
    ///
    pub per_page: usize,

    /// The current page of result that was returned by this query.
    ///
    pub current_page: usize,

    /// The number of the previous page, unless there is no previous page.
    ///
    pub prev_page: Option<usize>,

    /// The total number of pages that can be queried.
    ///
    pub total_pages: usize,

    /// The total number of search result returned.
    ///
    pub total_count: usize,

    /// The number of the next page, unless there is no next page.
    ///
    pub next_page: Option<usize>,

    /// Index of the first result on the current page, with respect to the total number of results.
    ///
    pub current_first_item: Option<usize>,

    /// Index of the last result on the current page, with respect to the total number of results.
    ///
    pub current_last_item: Option<usize>,
}

/// Data structure to hold the result of doing a search database query.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#database-list)
///
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct DatabaseList {
    /// A vector containing the first page of databases' metadata.
    ///
    pub databases: Vec<DatabaseMetadata>,

    /// The search metadata associated with this listing.
    ///
    pub meta: SearchMetadata,
}

/// Data structure to hold the result of a search dataset query.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#dataset-search)
///
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct DatasetList {
    /// A vector containing the first page of datasets' metadata.
    ///
    pub datasets: Vec<DatasetMetadata>,

    /// The search metadata associated with this listing.
    ///
    pub meta: SearchMetadata,
}

/// Data structure to hold the result of a code list query.
///
/// [Quandl API Reference](https://www.quandl.com/docs/api#dataset-list)
///
/// It should be noted that I slightly changed the meaning of a "dataset list" in this crate for
/// consistency with the `DatabaseList`. In this crate `DatasetList` and `DatabaseList` correspond
/// to Quandl's "dataset search" and "database list" respectively while `Vec<Code>` is Quandl's
/// equivalent of a "dataset list".
///
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct Code {
    /// The dataset code for the returned dataset.
    ///
    pub dataset_code: String,

    /// The code for the database this dataset belongs to.
    ///
    pub database_code: String,

    /// The title of this dataset.
    ///
    pub name: String,
}
