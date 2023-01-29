use crate::categories::Category;
use crate::common::{BulkResult, Image, V2BulkResult};
use crate::media::{PlaybackStream, VideoStream};
use crate::{options, Crunchyroll, Executor, Locale, Request, Result};
use chrono::{DateTime, Duration, Utc};
use serde::de::{DeserializeOwned, Error, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};
use std::sync::Arc;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(try_from = "Map<String, Value>")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct ThumbnailImages {
    pub thumbnail: Vec<Image>,
}

impl TryFrom<Map<String, Value>> for ThumbnailImages {
    type Error = serde_json::Error;

    fn try_from(value: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        if let Some(thumbnail) = value.get("thumbnail") {
            let thumbnail = serde_json::from_value::<Vec<Vec<Image>>>(thumbnail.clone())?
                .into_iter()
                .flatten()
                .collect::<Vec<Image>>();
            Ok(ThumbnailImages { thumbnail })
        } else {
            Ok(ThumbnailImages { thumbnail: vec![] })
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(try_from = "Map<String, Value>")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PosterImages {
    pub poster_tall: Vec<Image>,
    pub poster_wide: Vec<Image>,
}

impl TryFrom<Map<String, Value>> for PosterImages {
    type Error = serde_json::Error;

    fn try_from(value: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        let tall = if let Some(tall) = value.get("poster_tall") {
            serde_json::from_value::<Vec<Vec<Image>>>(tall.clone())?
                .into_iter()
                .flatten()
                .collect::<Vec<Image>>()
        } else {
            vec![]
        };
        let wide = if let Some(wide) = value.get("poster_wide") {
            serde_json::from_value::<Vec<Vec<Image>>>(wide.clone())?
                .into_iter()
                .flatten()
                .collect::<Vec<Image>>()
        } else {
            vec![]
        };

        Ok(Self {
            poster_tall: tall,
            poster_wide: wide,
        })
    }
}

#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SearchMetadata {
    /// [`None`] if queried by [`crate::Crunchyroll::query`].
    pub last_public: Option<DateTime<Utc>>,
    /// [`None`] if queried by [`crate::Crunchyroll::query`].
    pub rank: Option<u32>,

    pub score: f64,
    /// [`None`] if not queried by [`crate::Media<Series>::similar`] or
    /// [`crate::Media<MovieListing>::similar`].
    pub popularity_score: Option<f64>,
}

/// Metadata for a [`Media`] series.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Series {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    pub channel_id: String,

    #[serde(default)]
    pub content_provider: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub description: String,
    pub extended_description: String,

    pub series_launch_year: Option<u32>,

    pub episode_count: u32,
    pub season_count: u32,
    #[serde(default)]
    pub media_count: u32,

    #[serde(default)]
    pub season_tags: Vec<String>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    /// Might be empty. Some series have this field populated with locales, others not.
    pub audio_locales: Vec<Locale>,
    /// Might be empty. Some series have this field populated with locales, others not.
    pub subtitle_locales: Vec<Locale>,

    pub images: PosterImages,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    #[serde(default)]
    pub keywords: Vec<String>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    external_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    last_public: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    new: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    new_content: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    search_metadata: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    #[serde(rename = "type")]
    _type: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_description: Option<crate::StrictValue>,
}

#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SeasonVersion {
    #[serde(rename = "guid")]
    pub id: String,

    pub audio_locale: Locale,

    pub original: bool,

    pub variant: String,
}

/// Metadata for a [`Media`] season.
/// The deserializing requires a proxy struct because the season json response has two similar
/// fields, `audio_locale` and `audio_locales`. Sometimes the first is populated, sometimes the
/// second and sometimes both. They're representing the season audio but why it needs two fields
/// for this, who knows. `audio_locale` is also a [`Vec`] of locales but, if populated, contains
/// always only one locale.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Season {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    pub series_id: String,
    pub channel_id: String,

    pub title: String,
    pub slug_title: String,
    pub description: String,

    pub season_number: u32,
    pub season_sequence_number: u32,

    pub number_of_episodes: u32,

    pub is_complete: bool,

    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub season_tags: Vec<String>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    pub audio_locale: Locale,
    /// Most of the time, like 99%, this contains only one locale. But sometimes Crunchyroll does
    /// weird stuff and marks a season which clearly has only one locale with two locales. See
    /// [this](https://github.com/crunchy-labs/crunchy-cli/issues/81#issuecomment-1351813787) issue
    /// comment for an example.
    pub audio_locales: Vec<Locale>,
    pub subtitle_locales: Vec<Locale>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    /// If the season is not available this might contain some information why.
    pub availability_notes: String,

    pub versions: Vec<SeasonVersion>,

    #[cfg(feature = "__test_strict")]
    // currently empty (on all of my tests) but its might be filled in the future
    images: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    season_display_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    identifier: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    seo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_description: Option<crate::StrictValue>,
}

#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct EpisodeVersion {
    pub guid: String,
    pub media_guid: String,
    pub season_guid: String,

    pub audio_locale: Locale,

    pub is_premium_only: bool,
    pub original: bool,

    pub variant: String,
}

/// Metadata for a [`Media`] episode.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Episode {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(alias = "streams_link")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_streams_link")]
    pub stream_id: String,
    pub channel_id: String,
    #[serde(alias = "playback")]
    pub playback_url: String,
    pub identifier: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub description: String,

    // both missing if the episode is the last one in its season unpopulated
    #[serde(default)]
    pub next_episode_id: String,
    #[serde(default)]
    pub next_episode_title: String,

    pub season_id: String,
    pub season_title: String,
    pub season_slug_title: String,
    #[serde(default)]
    pub season_tags: Vec<String>,

    pub series_id: String,
    pub series_title: String,
    pub series_slug_title: String,

    // probably empty
    #[serde(default)]
    pub production_episode_id: String,

    /// Usually the same as [`Episode::episode_number`], just as string.
    pub episode: String,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_null_to_default")]
    pub episode_number: u32,
    /// Usually also the same as [`Episode::episode_number`]. If the episode number is null (which
    /// occurs for the first AOT episode, which is a preview, for example) this might be a floating
    /// number like 0.5.
    pub sequence_number: f32,

    pub season_number: u32,

    pub audio_locale: Locale,
    pub subtitle_locales: Vec<Locale>,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub episode_air_date: DateTime<Utc>,
    /// The same as episode_air_date as far as I can see.
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub upload_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub availability_starts: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub availability_ends: DateTime<Utc>,

    #[serde(deserialize_with = "crate::internal::serde::deserialize_thumbnail_image")]
    pub images: Vec<Image>,

    pub is_dubbed: bool,
    pub is_subbed: bool,

    pub is_premium_only: bool,
    pub is_clip: bool,

    pub is_mature: bool,
    pub maturity_ratings: Vec<String>,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    pub closed_captions_available: bool,

    pub eligible_region: String,

    pub versions: Vec<EpisodeVersion>,

    #[cfg(feature = "__test_strict")]
    media_type: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    external_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    new: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    search_metadata: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    #[serde(rename = "type")]
    _type: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    tenant_categories: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    seo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    listing_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    hd_flag: Option<crate::StrictValue>,
}

#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListingVersion {
    #[serde(rename = "guid")]
    pub id: String,

    pub audio_locale: Locale,

    pub original: bool,

    pub variant: String,
}

/// Metadata for a [`Media`] movie listing.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListing {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    pub channel_id: String,
    pub identifier: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub description: String,
    pub extended_description: String,

    pub content_provider: String,

    pub movie_release_year: u32,

    /// Sometimes empty, sometimes not. Not recommended to rely on this.
    pub audio_locale: Locale,
    /// Sometimes empty, sometimes not. Not recommended to rely on this.
    pub subtitle_locales: Vec<Locale>,

    pub is_subbed: bool,
    pub is_dubbed: bool,

    pub images: PosterImages,

    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub season_tags: Vec<String>,

    pub is_premium_only: bool,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    pub versions: Vec<MovieListingVersion>,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    seo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    hd_flag: Option<crate::StrictValue>,
}

/// Metadata for a [`Media`] movie.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Movie {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(alias = "streams_link")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_streams_link")]
    pub stream_id: String,
    #[serde(alias = "playback")]
    pub playback_url: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub description: String,

    #[serde(alias = "listing_id")]
    pub movie_listing_id: String,

    pub movie_listing_title: String,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    pub images: ThumbnailImages,

    #[default(DateTime::< Utc >::from(std::time::SystemTime::UNIX_EPOCH))]
    free_available_date: DateTime<Utc>,
    #[default(DateTime::< Utc >::from(std::time::SystemTime::UNIX_EPOCH))]
    premium_available_date: DateTime<Utc>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,

    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    media_type: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    #[serde(default)]
    movie_listing_slug_title: crate::StrictValue,
}

macro_rules! impl_manual_media_deserialize {
    ($($media:ident = $metadata:literal)*) => {
        $(
            impl<'de> Deserialize<'de> for $media {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    let mut as_map = serde_json::Map::deserialize(deserializer)?;

                    if let Some(mut metadata) = as_map.remove($metadata) {
                        if let Some(object) = metadata.as_object_mut() {
                            as_map.append(object);
                        } else {
                            as_map.insert($metadata.to_string(), metadata);
                        }
                    }

                    #[cfg(feature = "__test_strict")]
                    {

                    }

                    $media::deserialize(
                        serde_json::to_value(as_map)
                            .map_err(|e| Error::custom(e.to_string()))?
                            .into_deserializer(),
                    )
                    .map_err(|e| Error::custom(e.to_string()))
                }
            }
        )*
    }
}

impl_manual_media_deserialize! {
    Series = "series_metadata"
    Season = "season_metadata"
    Episode = "episode_metadata"
    MovieListing = "movie_listing_metadata"
    Movie = "movie_metadata"
}

macro_rules! media_eq {
    ($($media:ident)*) => {
        $(
            impl PartialEq<Self> for $media {
                fn eq(&self, other: &Self) -> bool {
                    self.id == other.id
                }
            }
        )*
    }
}

media_eq! {
    Series Season Episode
    MovieListing Movie
}

macro_rules! impl_playback {
    ($($media:ident)*) => {
        $(
            impl $media {
                pub async fn playback(&self) -> Result<PlaybackStream> {
                    self.executor.get(&self.playback_url).request().await
                }
            }
        )*
    }
}

impl_playback! {
    Episode Movie
}

impl Series {
    /// Returns all series seasons.
    pub async fn seasons(&self, preferred_audio: Option<Locale>) -> Result<Vec<Season>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/series/{}/seasons",
            self.id
        );
        request_media(self.executor.clone(), endpoint, preferred_audio).await
    }
}

impl Season {
    /// Returns the series the season belongs to.
    pub async fn series(&self, preferred_audio: Option<Locale>) -> Result<Season> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/series/{}",
            self.series_id
        );
        Ok(
            request_media(self.executor.clone(), endpoint, preferred_audio)
                .await?
                .remove(0),
        )
    }

    /// Returns all episodes of this season.
    pub async fn episodes(&self, preferred_audio: Option<Locale>) -> Result<Vec<Episode>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/seasons/{}/episodes",
            self.id
        );
        request_media(self.executor.clone(), endpoint, preferred_audio).await
    }
}

impl Episode {
    /// Returns the series the episode belongs to.
    pub async fn series(&self, preferred_audio: Option<Locale>) -> Result<Series> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/series/{}",
            self.series_id
        );
        Ok(
            request_media(self.executor.clone(), endpoint, preferred_audio)
                .await?
                .remove(0),
        )
    }

    /// Returns the season the episode belongs to.
    pub async fn season(&self, preferred_audio: Option<Locale>) -> Result<Season> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/seasons/{}",
            self.season_id
        );
        Ok(
            request_media(self.executor.clone(), endpoint, preferred_audio)
                .await?
                .remove(0),
        )
    }
}

impl MovieListing {
    /// Returns all movies for this movie listing.
    pub async fn movies(&self, preferred_audio: Option<Locale>) -> Result<Vec<Movie>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/movie_listings/{}/movies",
            self.id
        );
        request_media(self.executor.clone(), endpoint, preferred_audio).await
    }
}

impl Movie {
    /// Returns the parent movie listing of this movie.
    pub async fn movie_listing(&self, preferred_audio: Option<Locale>) -> Result<MovieListing> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/movie_listings/{}",
            self.movie_listing_id
        );
        Ok(
            request_media(self.executor.clone(), endpoint, preferred_audio)
                .await?
                .remove(0),
        )
    }
}

async fn request_media<T: Default + DeserializeOwned + Request>(
    executor: Arc<Executor>,
    endpoint: String,
    preferred_audio: Option<Locale>,
) -> Result<Vec<T>> {
    let result: V2BulkResult<T> = executor
        .get(endpoint)
        .query(&preferred_audio.map_or(vec![], |l| {
            vec!["preferred_audio_language".to_string(), l.to_string()]
        }))
        .apply_locale_query()
        .request()
        .await?;
    Ok(result.data)
}

macro_rules! impl_from_id {
    ($($media:ident = $endpoint:literal),*) => {
        $(
            impl $media {
                pub async fn from_id<S: AsRef<str>>(crunchyroll: &Crunchyroll, id: S, preferred_audio: Option<Locale>) -> Result<$media> {
                    let endpoint = format!($endpoint, id.as_ref());
                    Ok(request_media(crunchyroll.executor.clone(), endpoint, preferred_audio).await?.remove(0))
                }
            }
        )*
    }
}

impl_from_id! {
    Series = "https://www.crunchyroll.com/content/v2/cms/series/{}",
    Season = "https://www.crunchyroll.com/content/v2/cms/seasons/{}",
    Episode = "https://www.crunchyroll.com/content/v2/cms/episodes/{}",
    MovieListing = "https://www.crunchyroll.com/content/v2/cms/movie_listings/{}",
    Movie = "https://www.crunchyroll.com/content/v2/cms/movies/{}"
}

/// Collection of all media types. Useful in situations where [`Media`] can contain more than one
/// specific media.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum MediaCollection {
    Series(Series),
    Season(Season),
    Episode(Episode),
    MovieListing(MovieListing),
    Movie(Movie),
}

impl<'de> Deserialize<'de> for MediaCollection {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_map = serde_json::Map::deserialize(deserializer)?;

        let err_conv = |e: serde_json::Error| Error::custom(e.to_string());

        if as_map.contains_key("series_metadata") || as_map.contains_key("series_launch_year") {
            Ok(MediaCollection::Series(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("season_metadata")
            || as_map.contains_key("number_of_episodes")
        {
            Ok(MediaCollection::Season(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("episode_metadata") || as_map.contains_key("sequence_number")
        {
            Ok(MediaCollection::Episode(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("movie_listing_metadata")
            || as_map.contains_key("movie_release_year")
        {
            Ok(MediaCollection::MovieListing(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("movie_metadata")
            || as_map.contains_key("movie_listing_title")
        {
            Ok(MediaCollection::Movie(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else {
            Err(Error::custom(
                "could not deserialize into media collection".to_string(),
            ))
        }
    }
}

impl Default for MediaCollection {
    fn default() -> Self {
        Self::Series(Series::default())
    }
}

#[async_trait::async_trait]
impl Request for MediaCollection {
    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        match self {
            MediaCollection::Series(series) => series.__set_executor(executor).await,
            MediaCollection::Season(season) => season.__set_executor(executor).await,
            MediaCollection::Episode(episode) => episode.__set_executor(executor).await,
            MediaCollection::MovieListing(movie_listing) => {
                movie_listing.__set_executor(executor).await
            }
            MediaCollection::Movie(movie) => movie.__set_executor(executor).await,
        }
    }
}

macro_rules! impl_media_collection {
    ($($media:ident)*) => {
        $(
            impl From<$media> for MediaCollection {
                fn from(value: $media) -> Self {
                    MediaCollection::$media(value)
                }
            }
        )*
    }
}

impl_media_collection! {
    Series Season Episode MovieListing Movie
}

options! {
    SimilarOptions;
    /// Limit of results to return.
    limit(u32, "n") = Some(20),
    /// Specifies the index from which the entries should be returned.
    start(u32, "start") = None
}

macro_rules! impl_media_video_collection {
    ($($media_video:ident)*) => {
        $(
            impl $media_video {
                /// Similar series or movie listing to the current item.
                pub async fn similar(&self, options: SimilarOptions) -> Result<BulkResult<MediaCollection>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v1/{}/similar_to", self.executor.details.account_id.clone()?);
                    self.executor.get(endpoint)
                        .query(&[("guid", &self.id)])
                        .query(&options.into_query())
                        .apply_locale_query()
                        .request()
                        .await
                }
            }
        )*
    }
}

impl_media_video_collection! {
    Series MovieListing
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct VideoIntroResult {
    media_id: String,

    #[serde(rename = "startTime")]
    start_time: f64,
    #[serde(rename = "endTime")]
    end_time: f64,
    duration: f64,

    /// Id of the next episode.
    #[serde(rename = "comparedWith")]
    compared_with: String,

    /// It seems that this represents the episode number relative to the season the episode is part
    /// of. But in a weird way. It is, for example, '0003.00' instead of simply 3 if it's the third
    /// episode in a season.
    ordering: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    last_updated: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct RelatedMedia<T: Request + DeserializeOwned> {
    pub fully_watched: bool,

    pub playhead: u32,

    #[serde(alias = "panel")]
    #[serde(deserialize_with = "deserialize_panel")]
    pub media: T,

    #[cfg(feature = "__test_strict")]
    shortcut: Option<crate::StrictValue>,
}

pub(crate) fn deserialize_panel<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let mut as_map = Map::deserialize(deserializer)?;

    if let Some(mut episode_metadata) = as_map.remove("episode_metadata") {
        as_map.append(episode_metadata.as_object_mut().unwrap())
    }

    serde_json::from_value(serde_json::to_value(as_map).map_err(|e| Error::custom(e.to_string()))?)
        .map_err(|e| Error::custom(e.to_string()))
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PlayheadInformation {
    playhead: u32,

    content_id: String,

    fully_watched: bool,

    /// Date when the last playhead update was
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    last_modified: DateTime<Utc>,
}

macro_rules! impl_media_video {
    ($($media_video:ident)*) => {
        $(
            impl $media_video {
                /// Streams for this episode / movie.
                pub async fn streams(&self) -> Result<VideoStream> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/cms/v2/{}/videos/{}/streams",
                        self.executor.details.bucket, self.stream_id
                    );
                    self.executor.get(endpoint)
                        .apply_media_query()
                        .apply_locale_query()
                        .request()
                        .await
                }

                /// Check if the episode / movie can be watched.
                pub async fn available(&self) -> bool {
                    self.executor.details.premium || !self.is_premium_only
                }

                /// Get time _in seconds_ when the episode / movie intro begins and ends.
                pub async fn intro(&self) -> Result<Option<(f64, f64)>> {
                    let endpoint = format!(
                        "https://static.crunchyroll.com/datalab-intro-v2/{}.json",
                        self.id
                    );
                    let raw_result = self.executor.get(endpoint)
                        .request_raw()
                        .await?;
                    let result = String::from_utf8_lossy(raw_result.as_slice());
                    if result.contains("</Error>") {
                        Ok(None)
                    } else {
                        let video_intro_result: VideoIntroResult = serde_json::from_str(&result)?;
                        Ok(Some((video_intro_result.start_time, video_intro_result.end_time)))
                    }
                }

                /// Return the previous episode / movie. Is [`None`] if the current media is the
                /// first in its season / has no previous media.
                pub async fn previous(&self, preferred_audio: Option<Locale>) -> Result<Option<RelatedMedia<$media_video>>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v2/discover/previous_episode/{}", &self.id);
                    let result: serde_json::Value = self.executor.get(endpoint)
                        .query(&preferred_audio.map_or(vec![], |l| {
                            vec!["preferred_audio_language".to_string(), l.to_string()]
                        }))
                        .apply_locale_query()
                        .request()
                        .await?;
                    let as_map: serde_json::Map<String, serde_json::Value> = serde_json::from_value(result.clone())?;
                    if as_map.is_empty() {
                        Ok(None)
                    } else {
                        let mut previous: V2BulkResult<RelatedMedia<$media_video>> = serde_json::from_value(result)?;
                        Ok(Some(previous.data.remove(0)))
                    }
                }

                /// Return the next episode / movie. Is [`None`] if the current media is the last in
                /// its season / has no further media afterwards.
                pub async fn next(&self, preferred_audio: Option<Locale>) -> Result<Option<RelatedMedia<$media_video>>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v2/discover/up_next/{}", self.id);
                    let result: serde_json::Value = self.executor.get(endpoint)
                        .query(&preferred_audio.map_or(vec![], |l| {
                            vec!["preferred_audio_language".to_string(), l.to_string()]
                        }))
                        .apply_locale_query()
                        .request()
                        .await?;
                    let as_map: serde_json::Map<String, serde_json::Value> = serde_json::from_value(result.clone())?;
                    if as_map.is_empty() {
                        Ok(None)
                    } else {
                        let mut next: V2BulkResult<RelatedMedia<$media_video>> = serde_json::from_value(result)?;
                        Ok(Some(next.data.remove(0)))
                    }
                }

                /// Get playhead information.
                pub async fn playhead(&self) -> Result<Option<PlayheadInformation>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v1/playheads/{}/{}", self.executor.details.account_id.clone()?, &self.id);
                    let result: serde_json::Value = self.executor.get(endpoint)
                        .apply_locale_query()
                        .request()
                        .await?;
                    let as_map: serde_json::Map<String, serde_json::Value> = serde_json::from_value(result.clone())?;
                    if as_map.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(serde_json::from_value(as_map.get(&self.id).unwrap().clone())?))
                    }
                }

                /// Set the playhead (current playback position) for this episode / movie. Used unit
                /// is seconds. Setting the playhead also triggers the Crunchyroll Discord
                /// integration so if you update the playhead and have Crunchyroll connected to
                /// Discord, this episode / movie will be shown as your Discord status.
                pub async fn set_playhead(&self, position: u32) -> Result<()> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v1/playheads/{}", self.executor.details.account_id.clone()?);
                    self.executor.post(endpoint)
                        .apply_locale_query()
                        .json(&serde_json::json!({"content_id": &self.id, "playhead": position}))
                        .request::<crate::EmptyJsonProxy>()
                        .await?;
                    Ok(())
                }
            }
        )*
    }
}

impl_media_video! {
    Episode Movie
}
