//! Paginators for the table bucket list operations.
//!
//! Mirrors the Go SDK v2 paginator semantics (`client_paginators.go`):
//! `has_next` stays true until a page reports an empty continuation token, and
//! each page's token feeds the following request.

use crate::api::tables::{
    ListNamespacesRequest, ListNamespacesResult, ListTableBucketsRequest, ListTableBucketsResult,
    ListTablesRequest, ListTablesResult,
};
use crate::client::Client;

/// Paginates over [Client::list_table_buckets].
pub struct ListTableBucketsPaginator<'a> {
    client: &'a Client,
    request: ListTableBucketsRequest,
    /// Page size applied to every page. When `None`, the request's own
    /// `max_buckets` is left untouched.
    pub limit: Option<i32>,
    first_page: bool,
    done: bool,
}

impl<'a> ListTableBucketsPaginator<'a> {
    /// Returns true while more pages may be available.
    pub fn has_next(&self) -> bool {
        self.first_page || !self.done
    }

    /// Fetches the next page, or `Ok(None)` when the listing is exhausted.
    pub async fn next_page(
        &mut self,
    ) -> Result<Option<ListTableBucketsResult>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.has_next() {
            return Ok(None);
        }

        if let Some(limit) = self.limit {
            self.request.max_buckets = Some(limit);
        }

        let result = self.client.list_table_buckets(&self.request).await?;

        self.first_page = false;
        self.done = result
            .continuation_token
            .as_deref()
            .unwrap_or_default()
            .is_empty();
        self.request.continuation_token = result.continuation_token.clone();

        Ok(Some(result))
    }
}

/// Paginates over [Client::list_namespaces].
pub struct ListNamespacesPaginator<'a> {
    client: &'a Client,
    request: ListNamespacesRequest,
    /// Page size applied to every page. When `None`, the request's own
    /// `max_namespaces` is left untouched.
    pub limit: Option<i32>,
    first_page: bool,
    done: bool,
}

impl<'a> ListNamespacesPaginator<'a> {
    /// Returns true while more pages may be available.
    pub fn has_next(&self) -> bool {
        self.first_page || !self.done
    }

    /// Fetches the next page, or `Ok(None)` when the listing is exhausted.
    pub async fn next_page(
        &mut self,
    ) -> Result<Option<ListNamespacesResult>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.has_next() {
            return Ok(None);
        }

        if let Some(limit) = self.limit {
            self.request.max_namespaces = Some(limit);
        }

        let result = self.client.list_namespaces(&self.request).await?;

        self.first_page = false;
        self.done = result
            .continuation_token
            .as_deref()
            .unwrap_or_default()
            .is_empty();
        self.request.continuation_token = result.continuation_token.clone();

        Ok(Some(result))
    }
}

/// Paginates over [Client::list_tables].
pub struct ListTablesPaginator<'a> {
    client: &'a Client,
    request: ListTablesRequest,
    /// Page size applied to every page. When `None`, the request's own
    /// `max_tables` is left untouched.
    pub limit: Option<i32>,
    first_page: bool,
    done: bool,
}

impl<'a> ListTablesPaginator<'a> {
    /// Returns true while more pages may be available.
    pub fn has_next(&self) -> bool {
        self.first_page || !self.done
    }

    /// Fetches the next page, or `Ok(None)` when the listing is exhausted.
    pub async fn next_page(
        &mut self,
    ) -> Result<Option<ListTablesResult>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.has_next() {
            return Ok(None);
        }

        if let Some(limit) = self.limit {
            self.request.max_tables = Some(limit);
        }

        let result = self.client.list_tables(&self.request).await?;

        self.first_page = false;
        self.done = result
            .continuation_token
            .as_deref()
            .unwrap_or_default()
            .is_empty();
        self.request.continuation_token = result.continuation_token.clone();

        Ok(Some(result))
    }
}

impl Client {
    /// Creates a paginator for [Client::list_table_buckets].
    pub fn list_table_buckets_paginator(
        &self,
        request: ListTableBucketsRequest,
    ) -> ListTableBucketsPaginator<'_> {
        ListTableBucketsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_namespaces].
    pub fn list_namespaces_paginator(
        &self,
        request: ListNamespacesRequest,
    ) -> ListNamespacesPaginator<'_> {
        ListNamespacesPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_tables].
    pub fn list_tables_paginator(&self, request: ListTablesRequest) -> ListTablesPaginator<'_> {
        ListTablesPaginator {
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
    use std::rc::Rc;

    use super::*;
    use crate::config::Config;
    use crate::credential::StaticCredentialsProvider;

    const BUCKET_ARN: &str = "acs:osstables:cn-hangzhou:1234567890123456:bucket/demo-bucket";
    const REGION: &str = "cn-hangzhou";

    /// Path style keeps the endpoint host in the URL, so a mock server on
    /// localhost can be addressed directly. Everything else is the shipped
    /// configuration, including the product's own V4 signer.
    fn mock_client(server: &mockito::ServerGuard) -> Client {
        Client::new_tables(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region(REGION)
                .with_use_path_style(true)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                ))),
        )
    }

    #[test]
    fn test_paginators_start_with_a_page_available() {
        let client = Client::new_tables(&Config::default().with_region(REGION));

        let paginator = client.list_table_buckets_paginator(ListTableBucketsRequest::default());
        assert!(paginator.has_next());
        assert!(paginator.limit.is_none());
        // Construction must not touch the request's own markers.
        assert!(paginator.request.continuation_token.is_none());
        assert!(paginator.request.max_buckets.is_none());

        let paginator = client.list_namespaces_paginator(ListNamespacesRequest {
            table_bucket_arn: BUCKET_ARN.to_string(),
            ..Default::default()
        });
        assert!(paginator.has_next());
        assert!(paginator.request.continuation_token.is_none());

        let paginator = client.list_tables_paginator(ListTablesRequest {
            table_bucket_arn: BUCKET_ARN.to_string(),
            ..Default::default()
        });
        assert!(paginator.has_next());
        assert!(paginator.request.continuation_token.is_none());
    }

    #[tokio::test]
    async fn test_list_table_buckets_paginator_walks_pages() {
        let mut server = mockito::Server::new_async().await;

        let page_one = r#"{"continuationToken":"token2","tableBuckets":[{"name":"demo-1"}]}"#;
        let page_two = r#"{"tableBuckets":[{"name":"demo-2"}]}"#;

        let first = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::AllOf(vec![mockito::Matcher::UrlEncoded(
                "maxBuckets".to_string(),
                "1".to_string(),
            )]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(page_one)
            .expect(1)
            .create_async()
            .await;
        let second = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("continuationToken".to_string(), "token2".to_string()),
                mockito::Matcher::UrlEncoded("maxBuckets".to_string(), "1".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(page_two)
            .expect(1)
            .create_async()
            .await;

        let client = mock_client(&server);
        let mut paginator = client.list_table_buckets_paginator(ListTableBucketsRequest {
            max_buckets: Some(1),
            ..Default::default()
        });

        let page = paginator.next_page().await.unwrap().unwrap();
        assert_eq!(page.table_buckets[0].name.as_deref(), Some("demo-1"));
        assert!(paginator.has_next());

        let page = paginator.next_page().await.unwrap().unwrap();
        assert_eq!(page.table_buckets[0].name.as_deref(), Some("demo-2"));
        // The final page reports no token, so the listing is over.
        assert!(!paginator.has_next());
        assert!(paginator.next_page().await.unwrap().is_none());

        first.assert_async().await;
        second.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_namespaces_paginator_stops_after_one_page() {
        let mut server = mockito::Server::new_async().await;

        let body = r#"{"namespaces":[{"namespace":["space"]}]}"#;
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .expect(1)
            .create_async()
            .await;

        let client = mock_client(&server);
        let mut paginator = client.list_namespaces_paginator(ListNamespacesRequest {
            table_bucket_arn: BUCKET_ARN.to_string(),
            ..Default::default()
        });

        let page = paginator.next_page().await.unwrap().unwrap();
        assert_eq!(page.namespaces[0].namespace, vec!["space".to_string()]);
        assert!(!paginator.has_next());

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_tables_paginator_stops_after_one_page() {
        let mut server = mockito::Server::new_async().await;

        let body = r#"{"tables":[{"name":"table","namespace":["space"]}]}"#;
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .expect(1)
            .create_async()
            .await;

        let client = mock_client(&server);
        let mut paginator = client.list_tables_paginator(ListTablesRequest {
            table_bucket_arn: BUCKET_ARN.to_string(),
            namespace: Some("space".to_string()),
            ..Default::default()
        });

        let page = paginator.next_page().await.unwrap().unwrap();
        assert_eq!(page.tables[0].name.as_deref(), Some("table"));
        assert!(!paginator.has_next());

        mock.assert_async().await;
    }
}
