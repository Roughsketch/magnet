use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum Error {
    InvalidScheme,
    InvalidField(String, String),
    InvalidTopic(String),
    UrlEncode(serde_urlencoded::de::Error),
}

#[derive(Debug)]
pub struct MagnetUri {
    fields: Vec<Field>,
}

impl MagnetUri {
    pub fn from_fields(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    pub fn topic(&self) -> Option<Topic> {
        None
    }
}

impl FromStr for MagnetUri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //  Check if the scheme is valid
        if !s.starts_with("magnet:?") {
            return Err(Error::InvalidScheme);
        }

        //  Remove the scheme portion from the start of the string
        let args = s.trim_start_matches("magnet:?");

        //  Deserialize the key-value pairs from the urlencoded arguments
        let result = serde_urlencoded::from_str::<Vec<(String, String)>>(args)
            .map_err(Error::UrlEncode)?;

        //  Map each pair to a field type, failing early on bad data
        let fields = result
            .iter()
            .map(Field::from_pair)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(MagnetUri::from_fields(fields))
    }
}

#[derive(Debug)]
pub enum Field {
    AcceptableSource(String),
    DisplayName(String),
    Extension(String, String),
    ExactTopic(Topic),
    KeywordTopic(String),
    Length(u64),
    ManifestTopic(String),
    Source(String),
    Tracker(String),
    Unknown(String, String),
}

impl Field {
    pub fn new(key: &str, value: &str) -> Result<Self, Error> {
        match key {
            "as" => Ok(Field::AcceptableSource(value.into())),
            "dn" => Ok(Field::DisplayName(value.into())),
            "kt" => Ok(Field::KeywordTopic(value.into())),
            "mt" => Ok(Field::ManifestTopic(value.into())),
            "tr" => Ok(Field::Tracker(value.into())),
            "xl" => {
                if let Ok(len) = value.parse::<u64>() {
                    Ok(Field::Length(len))
                } else {
                    Err(Error::InvalidField(key.into(), value.into()))
                }
            },
            "xs" => Ok(Field::Source(value.into())),
            "xt" => Ok(Field::ExactTopic(Topic::from_str(value)?)),
            _ => {
                if key.starts_with("x.") {
                    Ok(Field::Extension(key.into(), value.into()))
                } else {
                    Ok(Field::Unknown(key.into(), value.into()))
                }
            }
        }
    }

    pub fn from_pair((key, value): &(String, String)) -> Result<Self, Error> {
        Field::new(key, value)
    }
}

#[derive(Debug)]
pub enum Topic {
    AICH(String),
    BitPrint(String),
    BitTorrent(String),
    ED2K(String),
    Kazaa(String),
    MD5(String),
    SHA1(String),
    TTHash(String),
    Unknown(String),
}

impl FromStr for Topic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //  If the topic does not start with urn, it isn't valid
        if !s.starts_with("urn:") {
            return Err(Error::InvalidTopic(s.into()));
        }

        let args = s
            .split(':') //  Split by :
            .skip(1)    //  Ignore the "urn:"" part
            .collect::<Vec<_>>();

        //  If there are no additional parts, the topic is invalid
        if args.is_empty() {
            return Err(Error::InvalidTopic(s.into()));
        }

        //  From the currently recognized URNs, there can only be
        //  either 2 or 3 args with TigerTree being the only 3 at
        //  the moment.
        match args.len() {
            2 => {
                //  Guaranteed to have len 2, so we can unwrap these
                let key = args.get(0).unwrap();
                let value = args.get(1).unwrap();

                //  If key matches, create a topic with the given value.
                //  If not, return an InvalidTopic error with the given topic.
                match *key {
                    "aich" => Ok(Topic::AICH(value.to_string())),
                    "bitprint" => Ok(Topic::BitPrint(value.to_string())),
                    "btih" => Ok(Topic::BitTorrent(value.to_string())),
                    "ed2k" => Ok(Topic::ED2K(value.to_string())),
                    "kzhash" => Ok(Topic::Kazaa(value.to_string())),
                    "md5" => Ok(Topic::MD5(value.to_string())),
                    "sha1" => Ok(Topic::SHA1(value.to_string())),
                    topic => Err(Error::InvalidTopic(topic.to_string())),
                }
            },
            3 => {
                //  Guaanteed to have len 3, so we can unwrap these
                let first = args.get(0).unwrap();
                let second = args.get(1).unwrap();
                let value = args.get(2).unwrap();

                //  Check if topic is tree:tiger
                if *first == "tree" && *second == "tiger" {
                    Ok(Topic::TTHash(value.to_string()))
                } else {
                    //  If invalid, return whole topic to debug
                    Err(Error::InvalidTopic(s.into()))
                }
            },
            //  If invalid, return whole topic to debug
            _ => Err(Error::InvalidTopic(s.into())),
        }
    }
}