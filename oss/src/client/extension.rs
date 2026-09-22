use crate::OperationInput;

/// Resolves the bucket name a request is signed and sent with.
///
/// Some OSS products address a bucket by a name derived from the client's
/// account and region rather than by the short name carried in the request.
/// Mirrors Go's `BucketNameResolver`.
pub trait BucketNameResolver {
    fn build_bucket_name(
        &self,
        input: &OperationInput,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

/// Builds the request URL when a product's addressing rules differ from OSS's.
///
/// The returned URL carries the scheme, host and path; the SDK still appends
/// the query string. Mirrors Go's `EndpointProvider` / `EndpointProviderE`.
pub trait EndpointProvider {
    fn build_url(
        &self,
        input: &OperationInput,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
