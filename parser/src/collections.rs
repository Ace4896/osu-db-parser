//! Models for the `collection.db` database file, which contains information on beatmap collections.

use std::path::Path;

use nom::{multi::length_count, number::complete::le_u32, IResult};

use crate::{
    common::{osu_string, OsuString},
    error::Error,
};

#[derive(Clone, Debug)]
pub struct CollectionListing {
    /// Version (e.g. 20150203)
    pub version: u32,

    /// Beatmap collections
    pub collections: Vec<Collection>,
}

#[derive(Clone, Debug)]
pub struct Collection {
    /// Name of the collection
    pub name: OsuString,

    /// MD5 hashes of beatmaps in the collection
    pub beatmap_md5s: Vec<OsuString>,
}

impl CollectionListing {
    /// Parses the contents of a `collection.db` file.
    pub fn from_bytes(data: &[u8]) -> Result<CollectionListing, Error> {
        let (_, listing) = collection_listing(data).map_err(|e| e.to_owned())?;
        Ok(listing)
    }

    /// Convenience method for reading the contents of an `collection.db` file and parsing it as a `CollectionListing`.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<CollectionListing, Error> {
        let data = std::fs::read(path)?;
        Self::from_bytes(&data)
    }
}

/// Parses a `collection.db` file.
fn collection_listing(input: &[u8]) -> IResult<&[u8], CollectionListing> {
    let (i, version) = le_u32(input)?;
    let (i, collections) = length_count(le_u32, collection)(i)?;

    Ok((
        i,
        CollectionListing {
            version,
            collections,
        },
    ))
}

/// Parses a collection entry in the `collection.db` file.
fn collection(input: &[u8]) -> IResult<&[u8], Collection> {
    let (i, name) = osu_string(input)?;
    let (i, beatmap_md5s) = length_count(le_u32, osu_string)(i)?;

    Ok((i, Collection { name, beatmap_md5s }))
}
