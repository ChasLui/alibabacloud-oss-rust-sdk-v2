//! Paginators for the data process list operations.
//!
//! Mirrors the Go SDK v2 paginator semantics (`client_paginators.go`):
//! `has_next` stays true until a page reports that no further token is
//! available, and each page's `next_token` feeds the following request.

use crate::api::dataprocess::{
    ListDataPipelineConfigurationsRequest, ListDataPipelineConfigurationsResult,
    ListDatasetsRequest, ListDatasetsResult, ListSmartClustersRequest, ListSmartClustersResult,
};
use crate::client::Client;

/// Paginates over [Client::list_datasets].
pub struct ListDatasetsPaginator<'a> {
    client: &'a Client,
    request: ListDatasetsRequest,
    /// Page size applied to every page. When `None`, the request's own
    /// `max_results` is left untouched.
    pub limit: Option<i64>,
    first_page: bool,
    done: bool,
}

impl<'a> ListDatasetsPaginator<'a> {
    /// Returns true while more pages may be available.
    pub fn has_next(&self) -> bool {
        self.first_page || !self.done
    }

    /// Fetches the next page, or `Ok(None)` when the listing is exhausted.
    pub async fn next_page(
        &mut self,
    ) -> Result<Option<ListDatasetsResult>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.has_next() {
            return Ok(None);
        }

        if let Some(limit) = self.limit {
            if limit > 0 {
                self.request.max_results = Some(limit);
            }
        }

        let result = self.client.list_datasets(&self.request).await?;

        self.first_page = false;
        if result.next_token.is_none() {
            self.done = true;
        }
        self.request.next_token = result.next_token.clone();

        Ok(Some(result))
    }
}

/// Paginates over [Client::list_smart_clusters].
pub struct ListSmartClustersPaginator<'a> {
    client: &'a Client,
    request: ListSmartClustersRequest,
    /// Page size applied to every page. When `None`, the request's own
    /// `max_results` is left untouched.
    pub limit: Option<i64>,
    first_page: bool,
    done: bool,
}

impl<'a> ListSmartClustersPaginator<'a> {
    /// Returns true while more pages may be available.
    pub fn has_next(&self) -> bool {
        self.first_page || !self.done
    }

    /// Fetches the next page, or `Ok(None)` when the listing is exhausted.
    pub async fn next_page(
        &mut self,
    ) -> Result<Option<ListSmartClustersResult>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.has_next() {
            return Ok(None);
        }

        if let Some(limit) = self.limit {
            if limit > 0 {
                self.request.max_results = Some(limit);
            }
        }

        let result = self.client.list_smart_clusters(&self.request).await?;

        self.first_page = false;
        if result.next_token.is_none() {
            self.done = true;
        }
        self.request.next_token = result.next_token.clone();

        Ok(Some(result))
    }
}

/// Paginates over [Client::list_data_pipeline_configurations].
pub struct ListDataPipelineConfigurationsPaginator<'a> {
    client: &'a Client,
    request: ListDataPipelineConfigurationsRequest,
    /// Page size applied to every page. When `None`, the request's own
    /// `max_results` is left untouched.
    pub limit: Option<i64>,
    first_page: bool,
    done: bool,
}

impl<'a> ListDataPipelineConfigurationsPaginator<'a> {
    /// Returns true while more pages may be available.
    pub fn has_next(&self) -> bool {
        self.first_page || !self.done
    }

    /// Fetches the next page, or `Ok(None)` when the listing is exhausted.
    pub async fn next_page(
        &mut self,
    ) -> Result<
        Option<ListDataPipelineConfigurationsResult>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        if !self.has_next() {
            return Ok(None);
        }

        if let Some(limit) = self.limit {
            if limit > 0 {
                self.request.max_results = Some(limit);
            }
        }

        let result = self
            .client
            .list_data_pipeline_configurations(&self.request)
            .await?;

        self.first_page = false;
        // The data pipeline listing reports an empty token instead of a nil
        // one when the listing is exhausted.
        if result.next_token.as_deref().unwrap_or_default().is_empty() {
            self.done = true;
        }
        self.request.next_token = result.next_token.clone();

        Ok(Some(result))
    }
}

impl Client {
    /// Creates a paginator for [Client::list_datasets].
    pub fn list_datasets_paginator(
        &self,
        request: ListDatasetsRequest,
    ) -> ListDatasetsPaginator<'_> {
        ListDatasetsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_smart_clusters].
    pub fn list_smart_clusters_paginator(
        &self,
        request: ListSmartClustersRequest,
    ) -> ListSmartClustersPaginator<'_> {
        ListSmartClustersPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_data_pipeline_configurations].
    pub fn list_data_pipeline_configurations_paginator(
        &self,
        request: ListDataPipelineConfigurationsRequest,
    ) -> ListDataPipelineConfigurationsPaginator<'_> {
        ListDataPipelineConfigurationsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_paginators_start_with_a_page_available() {
        let client = Client::new(&Config::default());

        let datasets = client.list_datasets_paginator(ListDatasetsRequest::new("bucket"));
        assert!(datasets.has_next());

        let smart_clusters = client
            .list_smart_clusters_paginator(ListSmartClustersRequest::new("bucket", "dataset"));
        assert!(smart_clusters.has_next());

        let data_pipelines = client.list_data_pipeline_configurations_paginator(
            ListDataPipelineConfigurationsRequest::new(),
        );
        assert!(data_pipelines.has_next());
    }
}
