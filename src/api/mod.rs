// utoipa's OpenApi derive macro triggers this lint in generated code
#![allow(clippy::needless_for_each)]

pub(crate) mod admin;
pub(crate) mod auth;
pub mod errors;
pub(crate) mod extractors;
pub(crate) mod healthcheck;
pub(crate) mod pagination;
pub(crate) mod schema;
pub(crate) mod search;
pub(crate) mod setup;
pub(crate) mod sites;
pub(crate) mod teams;
pub(crate) mod text;

pub use schema::ApiDoc;
