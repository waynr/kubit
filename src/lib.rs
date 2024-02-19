#![deny(rustdoc::broken_intra_doc_links, rustdoc::bare_urls, rust_2018_idioms)]

use local::DryRun;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Kube Error: {0}")]
    KubeError(#[source] kube::Error),

    #[error("{0}")]
    OCI(#[from] oci::Error),

    #[error("OCI error: {0}")]
    OCIParseError(#[from] oci_distribution::ParseError),

    #[error("Unsupported manifest type: Index")]
    UnsupportedManifestIndex,

    #[error("Unsupported dry run option: {0}")]
    UnsupportedDryRunOption(DryRun),

    #[error("Error decoding package config JSON: {0}")]
    DecodePackageConfig(serde_json::Error),

    #[error("Error decoding kubecfg pack metadata JSON: {0}")]
    DecodeKubecfgPackageMetadata(serde_json::Error),

    #[error("Error rendering spec back as JSON: {0}")]
    RenderOverlay(serde_json::Error),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    TempfilePersistError(#[from] tempfile::PersistError),

    #[error("Namespace is required")]
    NamespaceRequired,

    #[error("ConfigMap is required")]
    ConfigMapRequired,

    #[error(".spec.imagePullSecret currently requires to have exactly one pull secret")]
    UnsupportedMultipleImagePullSecrets,

    #[error("Image pull secret doesn't contain .dockerconfigjson")]
    NoDockerConfigJsonInImagePullSecret,

    #[error("Error decoding docker config JSON: {0}")]
    DecodeDockerConfig(#[from] docker_config::Error),

    #[error("Unsupported image pull secret type: {0:?}, should be kubernetes.io/dockerconfigjson")]
    BadImagePullSecretType(Option<String>),

    #[error("Finalizer Error: {0}")]
    // NB: awkward type because finalizer::Error embeds the reconciler error (which is this)
    // so boxing this error to break cycles
    FinalizerError(#[source] Box<kube::runtime::finalizer::Error<Error>>),

    #[error("Timeout elapsed before object could be deleted, retrying")]
    ResourceDeletionTimeout,

    #[error("The ConfigMap had an invalid status: {0}")]
    InvalidConfigMapStatus(#[from] serde_json::Error),

    #[error("The ConfigMap could not be converted to an AppInstance: {0}")]
    InvalidConfigMap(String),
}

impl From<kube::Error> for Error {
    fn from(e: kube::Error) -> Self {
        match e {
            kube::Error::Discovery(ref e) => {
                tracing::error!("discovery api failure (are the CRDs installed?): {e}");
            }
            _ => {
                tracing::error!("kubernetes api or connection error: {e}");
            }
        }
        Self::KubeError(e)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Expose all controller components used by main.
pub mod controller;

/// Resource type definitions.
pub mod resources;

pub mod apply;
pub mod delete;
pub mod helpers;
pub mod local;
pub mod metadata;
pub mod render;
mod scripting;

mod docker_config;
mod oci;
