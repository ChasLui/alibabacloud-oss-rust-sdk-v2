//! Paginators for the agentic list operations.
//!
//! Mirrors the Go SDK v2 paginator semantics (`client_paginators.go`):
//! `has_next` stays true until a page reports `is_truncated == false`, and each
//! page's `next_continuation_token` feeds the following request.

use crate::api::agentic::{
    ListAgenticBucketsRequest, ListAgenticBucketsResult, ListBucketSpacesRequest,
    ListBucketSpacesResult,
};
use crate::client::Client;

/// Paginates over [Client::list_agentic_buckets].
pub struct ListAgenticBucketsPaginator<'a> {
    client: &'a Client,
    request: ListAgenticBucketsRequest,
    /// Page size applied to every page. When `None`, the request's own
    /// `max_keys` is left untouched.
    pub limit: Option<i32>,
    first_page: bool,
    done: bool,
}

impl<'a> ListAgenticBucketsPaginator<'a> {
    /// Returns true while more pages may be available.
    pub fn has_next(&self) -> bool {
        self.first_page || !self.done
    }

    /// Fetches the next page, or `Ok(None)` when the listing is exhausted.
    pub async fn next_page(
        &mut self,
    ) -> Result<Option<ListAgenticBucketsResult>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.has_next() {
            return Ok(None);
        }

        if let Some(limit) = self.limit {
            self.request.max_keys = Some(limit);
        }

        let result = self.client.list_agentic_buckets(&self.request).await?;

        self.first_page = false;
        if !result.is_truncated.unwrap_or(false) {
            self.done = true;
        }
        self.request.continuation_token = result.next_continuation_token.clone();

        Ok(Some(result))
    }
}

/// Paginates over [Client::list_bucket_spaces].
pub struct ListBucketSpacesPaginator<'a> {
    client: &'a Client,
    request: ListBucketSpacesRequest,
    /// Page size applied to every page. When `None`, the request's own
    /// `max_keys` is left untouched.
    pub limit: Option<i32>,
    first_page: bool,
    done: bool,
}

impl<'a> ListBucketSpacesPaginator<'a> {
    /// Returns true while more pages may be available.
    pub fn has_next(&self) -> bool {
        self.first_page || !self.done
    }

    /// Fetches the next page, or `Ok(None)` when the listing is exhausted.
    pub async fn next_page(
        &mut self,
    ) -> Result<Option<ListBucketSpacesResult>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.has_next() {
            return Ok(None);
        }

        if let Some(limit) = self.limit {
            self.request.max_keys = Some(limit);
        }

        let result = self.client.list_bucket_spaces(&self.request).await?;

        self.first_page = false;
        if !result.is_truncated.unwrap_or(false) {
            self.done = true;
        }
        self.request.continuation_token = result.next_continuation_token.clone();

        Ok(Some(result))
    }
}

impl Client {
    /// Creates a paginator for [Client::list_agentic_buckets].
    pub fn list_agentic_buckets_paginator(
        &self,
        request: ListAgenticBucketsRequest,
    ) -> ListAgenticBucketsPaginator<'_> {
        ListAgenticBucketsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_bucket_spaces].
    pub fn list_bucket_spaces_paginator(
        &self,
        request: ListBucketSpacesRequest,
    ) -> ListBucketSpacesPaginator<'_> {
        ListBucketSpacesPaginator {
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
    use crate::SignatureVersionType;

    const ACCOUNT_ID: &str = "1234567890123456";
    const REGION: &str = "cn-hangzhou";

    /// Path style keeps the physical bucket name in the URL path, so a mock
    /// server on localhost can be addressed directly.
    fn mock_client(server: &mockito::ServerGuard) -> Client {
        Client::new_agentic(
            &Config::default()
                .with_endpoint(server.url().as_str())
                .with_region(REGION)
                .with_account_id(ACCOUNT_ID)
                .with_use_path_style(true)
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V1),
        )
    }

    #[test]
    fn test_paginator_initial_state() {
        let client = Client::new_agentic(
            &Config::default()
                .with_region(REGION)
                .with_account_id(ACCOUNT_ID),
        );

        let p = client.list_agentic_buckets_paginator(ListAgenticBucketsRequest::default());
        assert!(p.has_next());
        assert!(p.limit.is_none());
        // Construction must not touch the request's own markers.
        assert!(p.request.continuation_token.is_none());
        assert!(p.request.max_keys.is_none());

        let p = client.list_bucket_spaces_paginator(ListBucketSpacesRequest {
            bucket: "my-agentic".to_string(),
            ..Default::default()
        });
        assert!(p.has_next());
        assert!(p.limit.is_none());
        assert!(p.request.continuation_token.is_none());
    }

    #[tokio::test]
    async fn test_list_agentic_buckets_paginator_walks_pages() {
        let mut server = mockito::Server::new_async().await;

        let page_one = r#"<ListAgenticBucketsResult><IsTruncated>true</IsTruncated>
            <NextContinuationToken>token2</NextContinuationToken>
            <AgenticBuckets><AgenticBucket><Name>agentic-1</Name></AgenticBucket></AgenticBuckets>
            </ListAgenticBucketsResult>"#;
        let page_two = r#"<ListAgenticBucketsResult><IsTruncated>false</IsTruncated>
            <AgenticBuckets><AgenticBucket><Name>agentic-2</Name></AgenticBucket></AgenticBuckets>
            </ListAgenticBucketsResult>"#;

        let first = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("agenticBucket".to_string(), "".to_string()),
                mockito::Matcher::UrlEncoded("max-keys".to_string(), "1".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/xml")
            .with_body(page_one)
            .expect(1)
            .create_async()
            .await;
        let second = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("agenticBucket".to_string(), "".to_string()),
                mockito::Matcher::UrlEncoded(
                    "continuation-token".to_string(),
                    "token2".to_string(),
                ),
                mockito::Matcher::UrlEncoded("max-keys".to_string(), "1".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/xml")
            .with_body(page_two)
            .expect(1)
            .create_async()
            .await;

        let client = mock_client(&server);
        let mut paginator = client.list_agentic_buckets_paginator(ListAgenticBucketsRequest {
            max_keys: Some(1),
            ..Default::default()
        });

        let page = paginator.next_page().await.unwrap().unwrap();
        assert_eq!(page.agentic_buckets[0].name.as_deref(), Some("agentic-1"));
        assert!(paginator.has_next());

        let page = paginator.next_page().await.unwrap().unwrap();
        assert_eq!(page.agentic_buckets[0].name.as_deref(), Some("agentic-2"));
        // The final page reports no truncation, so the listing is over.
        assert!(!paginator.has_next());
        assert!(paginator.next_page().await.unwrap().is_none());

        first.assert_async().await;
        second.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_bucket_spaces_paginator_stops_after_one_page() {
        let mut server = mockito::Server::new_async().await;

        let body = r#"<ListBucketSpacesResult><IsTruncated>false</IsTruncated>
            <BucketSpaces><BucketSpace><Name>sandbox-001</Name></BucketSpace></BucketSpaces>
            </ListBucketSpacesResult>"#;
        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/xml")
            .with_body(body)
            .expect(1)
            .create_async()
            .await;

        let client = mock_client(&server);
        let mut paginator = client.list_bucket_spaces_paginator(ListBucketSpacesRequest {
            bucket: "my-agentic".to_string(),
            ..Default::default()
        });

        let page = paginator.next_page().await.unwrap().unwrap();
        assert_eq!(page.bucket_spaces[0].name.as_deref(), Some("sandbox-001"));
        assert!(!paginator.has_next());

        mock.assert_async().await;
    }
}
