use crate::common::Image;
use crate::{enum_values, BulkResult, Crunchyroll, Locale, Request};
use crate::{options, Result};
use serde::Deserialize;

enum_values! {
    #[doc = "Video categories / genres"]
    pub enum Category {
        Action = "action"
        Adventure = "adventure"
        Comedy = "comedy"
        Drama = "drama"
        Fantasy = "fantasy"
        Music = "music"
        Romance = "romance"
        SciFi = "sci-fi"
        Seinen = "seinen"
        Shojo = "shojo"
        Shonen = "shonen"
        SliceOfLife = "slice-of-life"
        Sports = "sports"
        Supernatural = "supernatural"
        Thriller = "thriller"
    }
}

impl From<TenantCategory> for Category {
    fn from(tenant_category: TenantCategory) -> Self {
        Self::from(tenant_category.category)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct TenantCategoryImages {
    pub background: Vec<Image>,
    pub low: Vec<Image>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct TenantCategoryLocalization {
    pub title: String,
    pub description: String,
    pub locale: Locale,
}

#[derive(Clone, Debug, Deserialize, Default, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SubTenantCategory {
    #[serde(rename = "tenant_category")]
    pub category: String,
    pub parent_category: String,
    pub slug: String,

    /// A human readable title & description about the category.
    pub localization: TenantCategoryLocalization,
}

#[derive(Clone, Debug, Deserialize, Default, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct TenantCategory {
    #[serde(rename = "tenant_category")]
    pub category: String,
    pub slug: String,

    /// Sub categories of this (parent) category. This field is missing if requested with
    /// `include_subcategories=false`. Why it isn't null... who knows.
    #[serde(default)]
    pub sub_categories: Vec<SubTenantCategory>,

    pub images: TenantCategoryImages,

    /// A human readable title & description about the category.
    pub localization: TenantCategoryLocalization,
}

options! {
    TenantCategoryOptions;
    #[doc = "If tenant categories should contains subcategories."]
    include_subcategories(bool, "include_subcategories") = Some(false)
}

impl Crunchyroll {
    /// Returns all video categories.
    pub async fn tenant_categories(
        &self,
        options: TenantCategoryOptions,
    ) -> Result<BulkResult<TenantCategory>> {
        let endpoint = "https://beta.crunchyroll.com/content/v1/tenant_categories";
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&options.to_query(&[(
                "locale".to_string(),
                self.executor.details.locale.to_string(),
            )]));
        self.executor.request(builder).await
    }
}