use anyhow::{anyhow, Result};
use std::path::Path;
use id3::TagLike;

#[derive(Debug, Clone)]
pub enum TagField {
    Title,
    Artist,
    Album,
    Year,
    Track,
    Genre,
    Comment,
}

impl TagField {
    pub fn label(&self) -> &str {
        match self {
            TagField::Title => "Title",
            TagField::Artist => "Artist",
            TagField::Album => "Album",
            TagField::Year => "Year",
            TagField::Track => "Track",
            TagField::Genre => "Genre",
            TagField::Comment => "Comment",
        }
    }

    pub fn all() -> Vec<TagField> {
        vec![
            TagField::Title,
            TagField::Artist,
            TagField::Album,
            TagField::Year,
            TagField::Track,
            TagField::Genre,
            TagField::Comment,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub track: String,
    pub genre: String,
    pub comment: String,
    pub format: AudioFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    Mp3,
    Flac,
    M4a,
}

impl AudioMetadata {
    pub fn read_from_file(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("No file extension"))?
            .to_lowercase();

        match ext.as_str() {
            "mp3" => Self::read_mp3(path),
            "flac" => Self::read_flac(path),
            "m4a" | "mp4" | "aac" => Self::read_m4a(path),
            _ => Err(anyhow!("Unsupported format: {}", ext)),
        }
    }

    fn read_mp3(path: &Path) -> Result<Self> {
        let tag = id3::Tag::read_from_path(path).unwrap_or_else(|_| id3::Tag::new());

        Ok(Self {
            title: tag.title().unwrap_or("").to_string(),
            artist: tag.artist().unwrap_or("").to_string(),
            album: tag.album().unwrap_or("").to_string(),
            year: tag.year().map(|y| y.to_string()).unwrap_or_default(),
            track: tag.track().map(|t| t.to_string()).unwrap_or_default(),
            genre: tag.genre().unwrap_or("").to_string(),
            comment: tag
                .comments()
                .next()
                .map(|c| c.text.clone())
                .unwrap_or_default(),
            format: AudioFormat::Mp3,
        })
    }

    fn read_flac(path: &Path) -> Result<Self> {
        let tag = metaflac::Tag::read_from_path(path)?;
        let vorbis = tag.vorbis_comments().ok_or_else(|| anyhow!("No vorbis comments"))?;

        let get_field = |name: &str| {
            vorbis
                .get(name)
                .and_then(|v| v.first())
                .map(|s| s.to_string())
                .unwrap_or_default()
        };

        Ok(Self {
            title: get_field("TITLE"),
            artist: get_field("ARTIST"),
            album: get_field("ALBUM"),
            year: get_field("DATE"),
            track: get_field("TRACKNUMBER"),
            genre: get_field("GENRE"),
            comment: get_field("COMMENT"),
            format: AudioFormat::Flac,
        })
    }

    fn read_m4a(path: &Path) -> Result<Self> {
        let tag = mp4ameta::Tag::read_from_path(path)?;

        Ok(Self {
            title: tag.title().unwrap_or("").to_string(),
            artist: tag.artist().unwrap_or("").to_string(),
            album: tag.album().unwrap_or("").to_string(),
            year: tag.year().unwrap_or("").to_string(),
            track: tag
                .track_number()
                .map(|t| t.to_string())
                .unwrap_or_default(),
            genre: tag.genre().unwrap_or("").to_string(),
            comment: tag.comment().unwrap_or("").to_string(),
            format: AudioFormat::M4a,
        })
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        match self.format {
            AudioFormat::Mp3 => self.write_mp3(path),
            AudioFormat::Flac => self.write_flac(path),
            AudioFormat::M4a => self.write_m4a(path),
        }
    }

    fn write_mp3(&self, path: &Path) -> Result<()> {
        let mut tag = id3::Tag::read_from_path(path).unwrap_or_else(|_| id3::Tag::new());

        if !self.title.is_empty() {
            tag.set_title(&self.title);
        } else {
            tag.remove_title();
        }

        if !self.artist.is_empty() {
            tag.set_artist(&self.artist);
        } else {
            tag.remove_artist();
        }

        if !self.album.is_empty() {
            tag.set_album(&self.album);
        } else {
            tag.remove_album();
        }

        if let Ok(year) = self.year.parse::<i32>() {
            tag.set_year(year);
        } else if !self.year.is_empty() {
            tag.set_text("TYER", &self.year);
        } else {
            tag.remove_year();
        }

        if let Ok(track) = self.track.parse::<u32>() {
            tag.set_track(track);
        } else {
            tag.remove_track();
        }

        if !self.genre.is_empty() {
            tag.set_genre(&self.genre);
        } else {
            tag.remove_genre();
        }

        if !self.comment.is_empty() {
            tag.add_frame(id3::frame::Comment {
                lang: "eng".to_string(),
                description: String::new(),
                text: self.comment.clone(),
            });
        }

        tag.write_to_path(path, id3::Version::Id3v24)?;
        Ok(())
    }

    fn write_flac(&self, path: &Path) -> Result<()> {
        let mut tag = metaflac::Tag::read_from_path(path)?;

        {
            let vorbis = tag.vorbis_comments_mut();

            vorbis.remove("TITLE");
            vorbis.remove("ARTIST");
            vorbis.remove("ALBUM");
            vorbis.remove("DATE");
            vorbis.remove("TRACKNUMBER");
            vorbis.remove("GENRE");
            vorbis.remove("COMMENT");

            if !self.title.is_empty() {
                vorbis.set_title(vec![&self.title]);
            }
            if !self.artist.is_empty() {
                vorbis.set_artist(vec![&self.artist]);
            }
            if !self.album.is_empty() {
                vorbis.set_album(vec![&self.album]);
            }
            if !self.year.is_empty() {
                vorbis.set("DATE", vec![&self.year]);
            }
            if !self.track.is_empty() {
                vorbis.set("TRACKNUMBER", vec![&self.track]);
            }
            if !self.genre.is_empty() {
                vorbis.set_genre(vec![&self.genre]);
            }
            if !self.comment.is_empty() {
                vorbis.set("COMMENT", vec![&self.comment]);
            }
        }

        tag.write_to_path(path)?;
        Ok(())
    }

    fn write_m4a(&self, path: &Path) -> Result<()> {
        let mut tag = mp4ameta::Tag::read_from_path(path)?;

        tag.set_title(&self.title);
        tag.set_artist(&self.artist);
        tag.set_album(&self.album);
        tag.set_year(&self.year);

        if let Ok(track) = self.track.parse::<u16>() {
            tag.set_track_number(track);
        }

        tag.set_genre(&self.genre);
        tag.set_comment(&self.comment);

        tag.write_to_path(path)?;
        Ok(())
    }

    pub fn get_field_value(&self, index: usize) -> String {
        match index {
            0 => self.title.clone(),
            1 => self.artist.clone(),
            2 => self.album.clone(),
            3 => self.year.clone(),
            4 => self.track.clone(),
            5 => self.genre.clone(),
            6 => self.comment.clone(),
            _ => String::new(),
        }
    }

    pub fn set_field_value(&mut self, index: usize, value: String) {
        match index {
            0 => self.title = value,
            1 => self.artist = value,
            2 => self.album = value,
            3 => self.year = value,
            4 => self.track = value,
            5 => self.genre = value,
            6 => self.comment = value,
            _ => {}
        }
    }

    pub fn field_count(&self) -> usize {
        7
    }
}
