// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use actix_http::error::PayloadError;
use aws_sdk_s3::{
    config::{
        endpoint::{Endpoint, EndpointFuture, Params, ResolveEndpoint},
        Builder, Credentials as AwsCred, Region,
    },
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
    Client,
};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use opentalk_controller_settings::MinIO;
use opentalk_types::api::error::ApiError;
use snafu::{ensure, OptionExt, ResultExt, Snafu};
use url::Url;

const CHUNK_SIZE: usize = 5_242_880; // 5 MebiByte (minimum for aws s3)

#[derive(Debug, Snafu)]
pub enum ObjectStorageError {
    #[snafu(display("{message}: {source}"))]
    InvalidSettings {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    InvalidResponse {
        message: String,
    },

    Upload {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    Put {
        message: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    Get {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    Delete {
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[snafu(display("the following bucket is missing: {name}"))]
    MissingBucket {
        name: String,
    },

    #[snafu(whatever)]
    Other {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<reqwest::Error> for ObjectStorageError {
    fn from(value: reqwest::Error) -> Self {
        Self::Other {
            message: "Reqwest error".into(),
            source: Some(value.into()),
        }
    }
}

impl From<PayloadError> for ObjectStorageError {
    fn from(value: PayloadError) -> Self {
        Self::Other {
            message: "Actix error".into(),
            source: Some(value.into()),
        }
    }
}

impl From<ObjectStorageError> for ApiError {
    fn from(value: ObjectStorageError) -> Self {
        log::error!("Internal Error: {value}");
        ApiError::internal()
    }
}

type Result<T, E = ObjectStorageError> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
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

        #[derive(Debug)]
        struct Resolver {
            minio_url: Url,
        }

        impl ResolveEndpoint for Resolver {
            fn resolve_endpoint(&self, params: &Params) -> EndpointFuture {
                let url = params.bucket().map(|bucket| self.minio_url.join(bucket));
                let url = match url {
                    Some(Ok(url)) => url.to_string(),
                    Some(Err(e)) => return EndpointFuture::ready(Err(e.into())),
                    None => self.minio_url.to_string(),
                };
                let endpoint = Endpoint::builder().url(url).build();

                EndpointFuture::ready(Ok(endpoint))
            }
        }

        let conf =
            Builder::new()
                .endpoint_resolver(Resolver {
                    minio_url: minio.uri.parse().map_err(Into::into).context(
                        InvalidSettingsSnafu {
                            message: "Invalid minio URI",
                        },
                    )?,
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
                .map_err(Into::into)
                .context(InvalidSettingsSnafu {
                    message: "Cannot list buckets for configured MinIO storage",
                })?
                .buckets()
                .iter()
                .any(|b| b.name() == Some(minio.bucket.as_str())),
            MissingBucketSnafu {
                name: minio.bucket.as_str(),
            },
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
    pub async fn put<E>(
        &self,
        key: &str,
        data: impl Stream<Item = Result<Bytes, E>> + Unpin,
    ) -> Result<usize>
    where
        ObjectStorageError: From<E>,
    {
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
                        .map_err(Into::into)
                        .context(UploadSnafu {
                            message: "failed to complete multipart upload",
                        })?;
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
                        .map_err(Into::into)
                        .context(UploadSnafu {
                            message: "failed to abort multipart upload",
                        })?;
                }
            }
        }

        res
    }

    async fn put_inner<E>(
        &self,
        key: &str,
        mut data: impl Stream<Item = Result<Bytes, E>> + Unpin,
        multipart_context: &mut Option<MultipartUploadContext>,
    ) -> Result<usize>
    where
        ObjectStorageError: From<E>,
    {
        let mut count = 0;
        let mut file_size = 0;

        loop {
            // This buffer must be reallocated with each iteration since the aws
            // crate takes ownership and drops the buffer internally.
            let mut buf = Vec::with_capacity(CHUNK_SIZE * 2);
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
                    .map_err(Into::into)
                    .context(PutSnafu {
                        message: "failed to put object",
                    })?;
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
                        .map_err(Into::into)
                        .context(PutSnafu {
                            message: "failed to create multipart upload",
                        })?;

                    // initialize multipart upload lazily once there is data to upload
                    multipart_context.insert(MultipartUploadContext {
                        upload_id: output.upload_id.context(InvalidResponseSnafu {
                            message: "no upload_id in create_multipart_upload response",
                        })?,
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
                    .map_err(Into::into)
                    .context(PutSnafu {
                        message: "failed to upload part",
                    })?;

                ctx.parts.push(
                    CompletedPart::builder()
                        .e_tag(part.e_tag().context(InvalidResponseSnafu {
                            message: "missing etag in upload_part response",
                        })?)
                        .part_number(count)
                        .build(),
                );
            }

            if last_part {
                break;
            }
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
            .await
            .map_err(Into::into)
            .context(GetSnafu)?;

        Ok(data.body)
    }

    pub async fn delete(&self, key: String) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(Into::into)
            .context(DeleteSnafu)?;

        Ok(())
    }
}

struct MultipartUploadContext {
    upload_id: String,
    parts: Vec<CompletedPart>,
}
