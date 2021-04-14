#![cfg(feature = "mp4")]

use crate::{
	impl_tag, Album, AnyTag, AudioTag, AudioTagEdit, AudioTagWrite, Error, MimeType, Picture,
	Result, TagType, ToAny, ToAnyTag,
};
use std::{fs::File, path::Path};

use crate::traits::ReadPath;
pub use mp4ameta::{FourCC, Tag as Mp4InnerTag};

impl ReadPath for Mp4InnerTag {
	fn from_path<P>(path: P, _tag_type: Option<TagType>) -> Result<Self>
	where
		P: AsRef<std::path::Path>,
		Self: Sized,
	{
		Ok(Self::read_from_path(path)?)
	}
}

impl_tag!(Mp4Tag, Mp4InnerTag, TagType::Mp4);

impl<'a> From<&'a Mp4Tag> for AnyTag<'a> {
	fn from(inp: &'a Mp4Tag) -> Self {
		let title = inp.title();
		let artists = inp.artists().map(|i| i.into_iter().collect::<Vec<_>>());
		let year = inp.year().map(|y| y as i32);
		let album = Album::new(inp.album_title(), inp.album_artists(), inp.album_cover());
		let (track_number, total_tracks) = inp.track();
		let (disc_number, total_discs) = inp.disc();

		Self {
			title,
			artists,
			year,
			album,
			track_number,
			total_tracks,
			disc_number,
			total_discs,
			comments: None,
			date: None,
			duration_ms: None, // TODO?
		}
	}
}

impl<'a> From<AnyTag<'a>> for Mp4Tag {
	fn from(inp: AnyTag<'a>) -> Self {
		let mut tag = Mp4Tag::new();

		if let Some(v) = inp.title() {
			tag.set_title(v)
		}
		if let Some(i) = inp.artists() {
			i.iter().for_each(|&a| tag.add_artist(a))
		}
		if let Some(v) = inp.year {
			tag.set_year(v)
		}
		if let Some(v) = inp.album().title {
			tag.set_album_title(v)
		}
		if let Some(i) = inp.album().artists {
			i.iter().for_each(|&a| tag.add_album_artist(a))
		}
		if let Some(v) = inp.track_number() {
			tag.set_track_number(v)
		}
		if let Some(v) = inp.total_tracks() {
			tag.set_total_tracks(v)
		}
		if let Some(v) = inp.disc_number() {
			tag.set_disc_number(v)
		}
		if let Some(v) = inp.total_discs() {
			tag.set_total_discs(v)
		}
		tag
	}
}

impl<'a> std::convert::TryFrom<&'a mp4ameta::Data> for Picture<'a> {
	type Error = Error;
	fn try_from(inp: &'a mp4ameta::Data) -> Result<Self> {
		Ok(match *inp {
			mp4ameta::Data::Png(ref data) => Self {
				data,
				mime_type: MimeType::Png,
			},
			mp4ameta::Data::Jpeg(ref data) => Self {
				data,
				mime_type: MimeType::Jpeg,
			},
			_ => return Err(Error::NotAPicture),
		})
	}
}

impl AudioTagEdit for Mp4Tag {
	fn title(&self) -> Option<&str> {
		self.0.title()
	}
	fn set_title(&mut self, title: &str) {
		self.0.set_title(title)
	}

	fn remove_title(&mut self) {
		self.0.remove_title();
	}
	fn artist(&self) -> Option<&str> {
		self.0.artist()
	}
	fn set_artist(&mut self, artist: &str) {
		self.0.set_artist(artist)
	}

	fn add_artist(&mut self, artist: &str) {
		self.0.add_artist(artist);
	}

	fn artists(&self) -> Option<Vec<&str>> {
		let v = self.0.artists().fold(Vec::new(), |mut v, a| {
			v.push(a);
			v
		});
		if v.is_empty() {
			None
		} else {
			Some(v)
		}
	}
	fn remove_artist(&mut self) {
		self.0.remove_artists();
	}

	fn year(&self) -> Option<i32> {
		self.0.year().and_then(|x| str::parse(x).ok())
	}
	fn set_year(&mut self, year: i32) {
		self.0.set_year(year.to_string())
	}

	fn remove_year(&mut self) {
		self.0.remove_year();
	}
	fn album_title(&self) -> Option<&str> {
		self.0.album()
	}

	fn set_album_title(&mut self, v: &str) {
		self.0.set_album(v)
	}
	fn remove_album_title(&mut self) {
		self.0.remove_album();
	}

	fn album_artists(&self) -> Option<Vec<&str>> {
		let mut album_artists = self.0.album_artists().peekable();

		if album_artists.peek().is_some() {
			Some(album_artists.collect())
		} else {
			None
		}
	}

	fn set_album_artists(&mut self, artists: String) {
		self.0.set_album_artist(artists)
	}

	fn add_album_artist(&mut self, artist: &str) {
		self.0.add_album_artist(artist)
	}

	fn remove_album_artists(&mut self) {
		self.0.remove_data(&FourCC(*b"aART"));
		self.0.remove_album_artists();
	}
	fn album_cover(&self) -> Option<Picture> {
		use mp4ameta::Data::{Jpeg, Png};

		self.0.artwork().and_then(|data| match data {
			Jpeg(d) => Some(Picture {
				data: d,
				mime_type: MimeType::Jpeg,
			}),
			Png(d) => Some(Picture {
				data: d,
				mime_type: MimeType::Png,
			}),
			_ => None,
		})
	}

	fn set_album_cover(&mut self, cover: Picture) {
		self.remove_album_cover();
		self.0.add_artwork(match cover.mime_type {
			MimeType::Png => mp4ameta::Data::Png(cover.data.to_owned()),
			MimeType::Jpeg => mp4ameta::Data::Jpeg(cover.data.to_owned()),
			_ => panic!("Only png and jpeg are supported in m4a"),
		});
	}
	fn remove_album_cover(&mut self) {
		self.0.remove_artwork();
	}
	fn remove_track(&mut self) {
		self.0.remove_track(); // faster than removing separately
	}
	fn track_number(&self) -> Option<u32> {
		self.0.track_number().map(u32::from)
	}

	fn set_track_number(&mut self, track: u32) {
		self.0.set_track_number(track as u16);
	}
	fn remove_track_number(&mut self) {
		self.0.remove_track_number();
	}
	fn total_tracks(&self) -> Option<u32> {
		self.0.total_tracks().map(u32::from)
	}
	fn set_total_tracks(&mut self, total_track: u32) {
		self.0.set_total_tracks(total_track as u16);
	}
	fn remove_total_tracks(&mut self) {
		self.0.remove_total_tracks();
	}
	fn remove_disc(&mut self) {
		self.0.remove_disc();
	}
	fn disc_number(&self) -> Option<u32> {
		self.0.disc_number().map(u32::from)
	}
	fn set_disc_number(&mut self, disc_number: u32) {
		self.0.set_disc_number(disc_number as u16)
	}
	fn remove_disc_number(&mut self) {
		self.0.remove_disc_number();
	}
	fn total_discs(&self) -> Option<u32> {
		self.0.total_discs().map(u32::from)
	}
	fn set_total_discs(&mut self, total_discs: u32) {
		self.0.set_total_discs(total_discs as u16)
	}
	fn remove_total_discs(&mut self) {
		self.0.remove_total_discs();
	}
}

impl AudioTagWrite for Mp4Tag {
	fn write_to(&self, file: &mut File) -> Result<()> {
		self.0.write_to(&file)?;
		Ok(())
	}
	fn write_to_path(&self, path: &str) -> Result<()> {
		self.0.write_to_path(path)?;
		Ok(())
	}
}