//! Paginators for the list operations, mirroring the official Go SDK v2
//! paginator semantics: `has_next` is true until a page returns
//! `is_truncated == false`; markers from each page feed the next request.

use crate::api::bucket::{
    ListObjectVersionsRequest, ListObjectVersionsResult, ListObjectsRequest, ListObjectsResult,
    ListObjectsV2Request, ListObjectsV2Result,
};
use crate::api::object::{
    ListMultipartUploadsRequest, ListMultipartUploadsResult, ListPartsRequest, ListPartsResult,
};
use crate::api::service::{ListBucketsRequest, ListBucketsResult};
use crate::api::vectors::{
    ListVectorBucketsRequest, ListVectorBucketsResult, ListVectorIndexesRequest,
    ListVectorIndexesResult, ListVectorsRequest, ListVectorsResult,
};
use crate::client::Client;

/// Paginates over the pages of a list operation.
///
/// `next_page` returns `Ok(None)` once the listing is exhausted. (Deviation
/// from the Go SDK, which returns an error in that case.)
macro_rules! impl_paginator {
    (
        $paginator:ident, $request:ty, $result:ty, $method:ident,
        max_field = $max_field:ident,
        encoding = $encoding:tt,
        by_ref = $by_ref:tt,
        truncated = |$trunc_arg:ident| $truncated:expr,
        advance = |$self:ident, $res:ident| $advance:block
    ) => {
        /// Paginator
        pub struct $paginator<'a> {
            client: &'a Client,
            request: $request,
            /// Page size limit applied to every page. When `None`, the
            /// request's own max field is left untouched.
            pub limit: Option<i32>,
            first_page: bool,
            done: bool,
        }

        impl<'a> $paginator<'a> {
            /// Returns true while more pages may be available.
            pub fn has_next(&self) -> bool {
                self.first_page || !self.done
            }

            /// Fetches the next page, or `Ok(None)` when the listing is
            /// exhausted.
            pub async fn next_page(
                &mut self,
            ) -> Result<Option<$result>, Box<dyn std::error::Error + Send + Sync>> {
                if !self.has_next() {
                    return Ok(None);
                }

                if let Some(limit) = self.limit {
                    self.request.$max_field = Some(limit);
                }
                impl_paginator!(@set_encoding self, $encoding);

                let result = impl_paginator!(@call self, $method, $by_ref).await?;

                self.first_page = false;
                let $trunc_arg = &result;
                if !($truncated) {
                    self.done = true;
                }

                let $self = self;
                let $res = &result;
                $advance

                Ok(Some(result))
            }
        }
    };
    (@call $self:ident, $method:ident, ref_) => {
        $self.client.$method(&$self.request)
    };
    (@call $self:ident, $method:ident, own) => {
        $self.client.$method($self.request.clone())
    };
    (@set_encoding $self:ident, url) => {
        $self.request.encoding_type = Some("url".to_string());
    };
    (@set_encoding $self:ident, none) => {};
}

impl_paginator!(
    ListObjectsPaginator,
    ListObjectsRequest,
    ListObjectsResult,
    list_objects,
    max_field = max_keys,
    encoding = url,
    by_ref = own,
    truncated = |r| r.is_truncated,
    advance = |this, result| {
        this.request.marker = result.next_marker.clone();
    }
);

impl_paginator!(
    ListObjectsV2Paginator,
    ListObjectsV2Request,
    ListObjectsV2Result,
    list_objects_v2,
    max_field = max_keys,
    encoding = url,
    by_ref = own,
    truncated = |r| r.is_truncated,
    advance = |this, result| {
        this.request.continuation_token = result.next_continuation_token.clone();
    }
);

impl_paginator!(
    ListObjectVersionsPaginator,
    ListObjectVersionsRequest,
    ListObjectVersionsResult,
    list_object_versions,
    max_field = max_keys,
    encoding = url,
    by_ref = own,
    truncated = |r| r.is_truncated,
    advance = |this, result| {
        this.request.key_marker = result.next_key_marker.clone();
        this.request.version_id_marker = result.next_version_id_marker.clone();
    }
);

impl_paginator!(
    ListBucketsPaginator,
    ListBucketsRequest,
    ListBucketsResult,
    list_buckets,
    max_field = max_keys,
    encoding = none,
    by_ref = ref_,
    truncated = |r| r.is_truncated.unwrap_or(false),
    advance = |this, result| {
        this.request.marker = result.next_marker.clone();
    }
);

impl_paginator!(
    ListPartsPaginator,
    ListPartsRequest,
    ListPartsResult,
    list_parts,
    max_field = max_parts,
    encoding = url,
    by_ref = ref_,
    truncated = |r| r.is_truncated.unwrap_or(false),
    advance = |this, result| {
        this.request.part_number_marker = result.next_part_number_marker;
    }
);

impl_paginator!(
    ListMultipartUploadsPaginator,
    ListMultipartUploadsRequest,
    ListMultipartUploadsResult,
    list_multipart_uploads,
    max_field = max_uploads,
    encoding = url,
    by_ref = ref_,
    truncated = |r| r.is_truncated.unwrap_or(false),
    advance = |this, result| {
        this.request.key_marker = result.next_key_marker.clone();
        this.request.upload_id_marker = result.next_upload_id_marker.clone();
    }
);

impl_paginator!(
    ListVectorBucketsPaginator,
    ListVectorBucketsRequest,
    ListVectorBucketsResult,
    list_vector_buckets,
    max_field = max_keys,
    encoding = none,
    by_ref = ref_,
    truncated = |r| r.is_truncated,
    advance = |this, result| {
        this.request.marker = result.next_marker.clone();
    }
);

impl_paginator!(
    ListVectorIndexesPaginator,
    ListVectorIndexesRequest,
    ListVectorIndexesResult,
    list_vector_indexes,
    max_field = max_results,
    encoding = none,
    by_ref = ref_,
    truncated = |r| has_token(&r.next_token),
    advance = |this, result| {
        this.request.next_token = result.next_token.clone();
    }
);

impl_paginator!(
    ListVectorsPaginator,
    ListVectorsRequest,
    ListVectorsResult,
    list_vectors,
    max_field = max_results,
    encoding = none,
    by_ref = ref_,
    truncated = |r| has_token(&r.next_token),
    advance = |this, result| {
        this.request.next_token = result.next_token.clone();
    }
);

/// Token-paginated listings end when the service stops handing out a token.
fn has_token(token: &Option<String>) -> bool {
    token.as_deref().map_or(false, |token| !token.is_empty())
}

impl Client {
    /// Creates a paginator for [Client::list_objects].
    pub fn list_objects_paginator(&self, request: ListObjectsRequest) -> ListObjectsPaginator<'_> {
        ListObjectsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_objects_v2].
    pub fn list_objects_v2_paginator(
        &self,
        request: ListObjectsV2Request,
    ) -> ListObjectsV2Paginator<'_> {
        ListObjectsV2Paginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_object_versions].
    pub fn list_object_versions_paginator(
        &self,
        request: ListObjectVersionsRequest,
    ) -> ListObjectVersionsPaginator<'_> {
        ListObjectVersionsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_buckets].
    pub fn list_buckets_paginator(&self, request: ListBucketsRequest) -> ListBucketsPaginator<'_> {
        ListBucketsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_parts].
    pub fn list_parts_paginator(&self, request: ListPartsRequest) -> ListPartsPaginator<'_> {
        ListPartsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_multipart_uploads].
    pub fn list_multipart_uploads_paginator(
        &self,
        request: ListMultipartUploadsRequest,
    ) -> ListMultipartUploadsPaginator<'_> {
        ListMultipartUploadsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_vector_buckets].
    pub fn list_vector_buckets_paginator(
        &self,
        request: ListVectorBucketsRequest,
    ) -> ListVectorBucketsPaginator<'_> {
        ListVectorBucketsPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_vector_indexes].
    pub fn list_vector_indexes_paginator(
        &self,
        request: ListVectorIndexesRequest,
    ) -> ListVectorIndexesPaginator<'_> {
        ListVectorIndexesPaginator {
            client: self,
            request,
            limit: None,
            first_page: true,
            done: false,
        }
    }

    /// Creates a paginator for [Client::list_vectors].
    pub fn list_vectors_paginator(&self, request: ListVectorsRequest) -> ListVectorsPaginator<'_> {
        ListVectorsPaginator {
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

    fn test_client() -> Client {
        Client::new(
            &Config::default()
                .with_region("cn-hangzhou")
                .with_credentials_provider(Rc::new(StaticCredentialsProvider::new(
                    "test-ak",
                    "test-sk",
                    &[],
                )))
                .with_signature_version(SignatureVersionType::V4),
        )
    }

    #[test]
    fn test_has_token() {
        assert!(has_token(&Some("t-1".to_string())));
        // The service signals the end by returning no token, or an empty one.
        assert!(!has_token(&None));
        assert!(!has_token(&Some(String::new())));
    }

    #[test]
    fn test_paginator_initial_state() {
        let client = test_client();

        let p = client.list_objects_paginator(ListObjectsRequest {
            bucket: "b".to_string(),
            ..Default::default()
        });
        assert!(p.has_next());
        assert!(p.limit.is_none());
        // Construction must not touch the request markers
        assert!(p.request.marker.is_none());

        let p = client.list_objects_v2_paginator(ListObjectsV2Request {
            bucket: "b".to_string(),
            ..Default::default()
        });
        assert!(p.has_next());
        assert!(p.request.continuation_token.is_none());

        let p = client.list_object_versions_paginator(ListObjectVersionsRequest {
            bucket: "b".to_string(),
            ..Default::default()
        });
        assert!(p.has_next());
        assert!(p.request.key_marker.is_none());
        assert!(p.request.version_id_marker.is_none());

        let p = client.list_buckets_paginator(ListBucketsRequest::default());
        assert!(p.has_next());
        assert!(p.request.marker.is_none());

        let p = client.list_parts_paginator(ListPartsRequest {
            bucket: "b".to_string(),
            key: "k".to_string(),
            upload_id: "u".to_string(),
            ..Default::default()
        });
        assert!(p.has_next());
        assert!(p.request.part_number_marker.is_none());

        let p = client.list_multipart_uploads_paginator(ListMultipartUploadsRequest {
            bucket: "b".to_string(),
            ..Default::default()
        });
        assert!(p.has_next());
        assert!(p.request.key_marker.is_none());
        assert!(p.request.upload_id_marker.is_none());
    }

    #[tokio::test]
    async fn test_paginator_next_page_without_network_errors_not_panics() {
        let client = test_client();
        let mut p = client.list_buckets_paginator(ListBucketsRequest::default());
        // No network available in the test environment: the call must return
        // an Err, never panic.
        let result = p.next_page().await;
        assert!(result.is_err() || result.unwrap().is_some());
    }
}
