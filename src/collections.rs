//! Models for the `collection.db` database file, which contains information on beatmap collections.

use nom::{multi::length_count, number::complete::le_u32, IResult};

use crate::common::{osu_string, OsuString};

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
