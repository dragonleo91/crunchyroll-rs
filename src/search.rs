pub mod query {
    use std::collections::HashMap;
    use std::sync::Arc;
    use serde::Deserialize;
    use crate::{Collection, Crunchyroll, Executor};
    use crate::common::{BulkResult, Request};
    use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};

    #[derive(Deserialize, Debug)]
    #[serde(try_from = "QueryResultsBulkResult")]
    pub struct QueryResults {
        #[serde(skip)]
        executor: Arc<Executor>,

        pub top_results: BulkResult<Collection>,
        pub series: BulkResult<Collection>,
        pub movie_listing: BulkResult<Collection>,
        pub episode: BulkResult<Collection>
    }

    impl Request for QueryResults {
        fn set_executor(&mut self, executor: Arc<Executor>) {
            self.executor = executor.clone();

            for collection in self.top_results.items.iter_mut() {
                collection.set_executor(executor.clone());
            }
            for collection in self.series.items.iter_mut() {
                collection.set_executor(executor.clone());
            }
            for collection in self.movie_listing.items.iter_mut() {
                collection.set_executor(executor.clone());
            }
            for collection in self.episode.items.iter_mut() {
                collection.set_executor(executor.clone());
            }
        }
    }

    impl TryFrom<QueryResultsBulkResult> for QueryResults {
        type Error = CrunchyrollError;

        fn try_from(value: QueryResultsBulkResult) -> std::result::Result<Self, Self::Error> {
            let mut top_results: Option<BulkResult<Collection>> = None;
            let mut series: Option<BulkResult<Collection>> = None;
            let mut movie_listing: Option<BulkResult<Collection>> = None;
            let mut episode: Option<BulkResult<Collection>> = None;

            for item in value.items {
                let result = BulkResult{ items: item.items, total: item.total };
                match item.result_type.as_str() {
                    "top_results" => top_results = Some(result),
                    "series" => series = Some(result),
                    "movie_listing" => movie_listing = Some(result),
                    "episode" => episode = Some(result),
                    _ => return Err(CrunchyrollError::Decode(
                        CrunchyrollErrorContext{ message: format!("invalid result type found: '{}'", item.result_type) }
                    ))
                };
            }

            Ok(Self {
                executor: Default::default(),
                top_results: top_results.ok_or_else(|| CrunchyrollError::Decode(
                    CrunchyrollErrorContext{ message: "could not find 'top_result' type".into() }
                ))?,
                series: series.ok_or_else(|| CrunchyrollError::Decode(
                    CrunchyrollErrorContext{ message: "could not find 'series' type".into() }
                ))?,
                movie_listing: movie_listing.ok_or_else(|| CrunchyrollError::Decode(
                    CrunchyrollErrorContext{ message: "could not find 'movie_listing' type".into() }
                ))?,
                episode: episode.ok_or_else(|| CrunchyrollError::Decode(
                    CrunchyrollErrorContext{ message: "could not find 'episode' type".into() }
                ))?
            })
        }
    }

    #[derive(Deserialize)]
    struct QueryResultsBulkResult {
        items: [QueryBulkResult; 4]
    }

    #[derive(Deserialize)]
    struct QueryBulkResult {
        #[serde(rename = "type")]
        result_type: String,
        items: Vec<Collection>,
        total: u32
    }

    #[derive(Clone, Debug)]
    pub enum QueryType {
        Series,
        MovieListing,
        Episode
    }

    #[derive(derive_setters::Setters, smart_default::SmartDefault)]
    pub struct QueryOptions {
        #[default = 20]
        pub limit: u32,
        pub result_type: Option<QueryType>
    }

    impl Crunchyroll {
        pub async fn query(&self, query: String, options: QueryOptions) -> Result<QueryResults> {
            let executor = self.executor.clone();

            let endpoint = "https://beta.crunchyroll.com/content/v1/search";
            let builder = executor.client
                .get(endpoint)
                .query(&HashMap::from([
                    ("q", query),
                    ("n", options.limit.to_string()),
                    ("type", options.result_type.map_or_else(|| "".to_string(), |f| format!("{:?}", f).to_string()).to_lowercase()),
                    ("locale", self.executor.locale.to_string())
                ]));

            executor.request(builder).await
        }
    }
}