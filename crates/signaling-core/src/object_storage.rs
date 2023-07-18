// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{ensure, Context, Result};
use aws_sdk_s3::{
    config::{Builder, Credentials as AwsCred, Region},
    endpoint::Params,
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
    Client,
};
use aws_smithy_http::endpoint::ResolveEndpoint;
use aws_smithy_types::endpoint::Endpoint;
use bytes::Bytes;
use controller_settings::MinIO;
use futures::{Stream, StreamExt};
use url::Url;

const CHUNK_SIZE: usize = 5_242_880; // 5 MebiByte (minimum for aws s3)

pub struct ObjectStorage {
    /// The s3 client
    client: Client,
    /// The configured bucket
    bucket: String,
}

impl ObjectStorage {
    pub async fn new(minio: &MinIO) -> Result<Self> {
        let credentials = AwsCred::new(
            minio.access_key.clone(),
            minio.secret_key.clone(),
            None,
            None,
            "opentalk",
        );

        struct Resolver {
            minio_url: Url,
        }

        impl ResolveEndpoint<Params> for Resolver {
            fn resolve_endpoint(&self, params: &Params) -> aws_smithy_http::endpoint::Result {
                let url = if let Some(bucket) = params.bucket() {
                    self.minio_url.join(bucket).unwrap().to_string()
                } else {
                    self.minio_url.to_string()
                };

                Ok(Endpoint::builder().url(url).build())
            }
        }

        let conf = Builder::new()
            .endpoint_resolver(Resolver {
                minio_url: minio.uri.parse()?,
            })
            .credentials_provider(credentials)
            .region(Region::new("unknown"))
            .build();

        let client = Client::from_conf(conf);

        // check if the bucket exists
        ensure!(
            client
                .list_buckets()
                .send()
                .await
                .context("Cannot list buckets for configured MinIO storage")?
                .buckets()
                .map(|b| b.iter().any(|b| b.name() == Some(minio.bucket.as_str())))
                .unwrap_or_default(),
            "Cannot find configured MinIO bucket"
        );

        log::info!("Using MinIO S3 bucket: {} ", minio.bucket,);

        Ok(Self {
            client,
            bucket: minio.bucket.clone(),
        })
    }

    /// Create a broken placeholder S3 client for tests
    ///
    /// The resulting [`ObjectStorage`] will error on first access. This is a placeholder until we can mock the client
    /// or have a minio test deployment.
    ///
    // TODO: create mock client or minio test deployment
    pub fn broken() -> Self {
        let credentials = AwsCred::new("broken", "broken", None, None, "broken");

        let conf = Builder::new()
            .endpoint_url("localhost")
            .credentials_provider(credentials)
            .region(Region::new(""))
            .build();

        let client = Client::from_conf(conf);

        Self {
            client,
            bucket: "broken".into(),
        }
    }

    /// Put an object into S3 storage
    ///
    /// Depending on the data size, this function will either use the `put_object` or `multipart_upload` S3 API call.
    ///
    /// Returns the file size of the uploaded object
    pub async fn put(
        &self,
        key: &str,
        data: impl Stream<Item = Result<Bytes>> + Unpin,
    ) -> Result<usize> {
        let mut multipart_context = None;

        let res = self.put_inner(key, data, &mut multipart_context).await;

        // complete or abort the multipart upload if the context exists
        if let Some(ctx) = multipart_context {
            match &res {
                Ok(_) => {
                    // complete the multipart upload
                    self.client
                        .complete_multipart_upload()
                        .bucket(&self.bucket)
                        .key(key)
                        .upload_id(ctx.upload_id)
                        .multipart_upload(
                            CompletedMultipartUpload::builder()
                                .set_parts(Some(ctx.parts))
                                .build(),
                        )
                        .send()
                        .await
                        .context("failed to complete multipart upload")?;
                }
                Err(_) => {
                    // abort the multi part upload in case of error
                    self.client
                        .abort_multipart_upload()
                        .bucket(&self.bucket)
                        .key(key)
                        .upload_id(ctx.upload_id)
                        .send()
                        .await
                        .context("failed to abort multipart upload")?;
                }
            }
        }

        res
    }

    async fn put_inner(
        &self,
        key: &str,
        mut data: impl Stream<Item = Result<Bytes>> + Unpin,
        multipart_context: &mut Option<MultipartUploadContext>,
    ) -> Result<usize> {
        let mut count = 0;
        let mut file_size = 0;
        let mut buf = Vec::with_capacity(CHUNK_SIZE * 2);

        loop {
            let mut last_part = false;

            // Read chunk to upload
            loop {
                match data.next().await {
                    Some(bytes) => {
                        buf.extend_from_slice(&bytes?);

                        if buf.len() >= CHUNK_SIZE {
                            break;
                        }
                    }
                    None => {
                        // EOS
                        last_part = true;
                        break;
                    }
                }
            }

            count += 1;
            file_size += buf.len();

            // Check if there is only one chunk to send
            // Skip multipart API and put object directly
            let put_object = last_part && count == 1;

            if put_object {
                self.client
                    .put_object()
                    .bucket(&self.bucket)
                    .key(key)
                    .content_length(buf.len() as i64)
                    .body(buf.into())
                    .send()
                    .await
                    .context("failed to put object")?;
            } else {
                let ctx = if let Some(ctx) = multipart_context {
                    ctx
                } else {
                    let output = self
                        .client
                        .create_multipart_upload()
                        .bucket(&self.bucket)
                        .key(key)
                        .send()
                        .await
                        .context("failed to create multipart upload")?;

                    // initialize multipart upload lazily once there is data to upload
                    multipart_context.insert(MultipartUploadContext {
                        upload_id: output
                            .upload_id
                            .context("no upload_id in create_multipart_upload response")?,
                        parts: Vec::new(),
                    })
                };

                // upload a part of the multipart
                let part = self
                    .client
                    .upload_part()
                    .bucket(&self.bucket)
                    .key(key)
                    .upload_id(&ctx.upload_id)
                    .part_number(count)
                    .content_length(buf.len() as i64)
                    .body(buf.into())
                    .send()
                    .await
                    .context("failed to upload part")?;

                ctx.parts.push(
                    CompletedPart::builder()
                        .e_tag(
                            part.e_tag()
                                .context("missing etag in upload_part response")?,
                        )
                        .part_number(count)
                        .build(),
                );
            }

            if last_part {
                break;
            }

            buf = Vec::with_capacity(CHUNK_SIZE * 2);
        }

        Ok(file_size)
    }

    pub async fn get(&self, key: String) -> Result<ByteStream> {
        let data = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(data.body)
    }

    pub async fn delete(&self, key: String) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(())
    }
}

struct MultipartUploadContext {
    upload_id: String,
    parts: Vec<CompletedPart>,
}
