use crate::components::logic::riff;
use crate::{
	Album, AnyTag, AudioTag, AudioTagEdit, AudioTagWrite, Result, TagType, ToAny, ToAnyTag,
};

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek};

use lofty_attr::impl_tag;

struct RiffInnerTag {
	data: HashMap<String, String>,
}

impl Default for RiffInnerTag {
	fn default() -> Self {
		let data: HashMap<String, String> = HashMap::new();

		Self { data }
	}
}

#[impl_tag(RiffInnerTag, TagType::RiffInfo)]
pub struct RiffTag;

impl RiffTag {
	#[allow(missing_docs)]
	#[allow(clippy::missing_errors_doc)]
	pub fn read_from<R>(reader: &mut R) -> Result<Self>
	where
		R: Read + Seek,
	{
		Ok(Self {
			inner: RiffInnerTag {
				data: riff::read_from(reader)?,
			},
		})
	}
}

impl RiffTag {
	fn get_value(&self, key: &str) -> Option<&str> {
		self.inner.data.get_key_value(key).map(|(_, v)| v.as_str())
	}

	fn set_value<V>(&mut self, key: &str, val: V)
	where
		V: Into<String>,
	{
		self.inner.data.insert(key.to_string(), val.into());
	}

	fn remove_key(&mut self, key: &str) {
		self.inner.data.remove(key);
	}
}

impl AudioTagEdit for RiffTag {
	fn title(&self) -> Option<&str> {
		self.get_value("INAM")
	}
	fn set_title(&mut self, title: &str) {
		self.set_value("INAM", title)
	}
	fn remove_title(&mut self) {
		self.remove_key("INAM")
	}

	fn artist_str(&self) -> Option<&str> {
		self.get_value("IART")
	}
	fn set_artist(&mut self, artist: &str) {
		self.set_value("IART", artist)
	}
	fn remove_artist(&mut self) {
		self.remove_key("IART")
	}

	fn date(&self) -> Option<String> {
		self.get_value("ICRD").map(std::string::ToString::to_string)
	}
	fn set_date(&mut self, date: &str) {
		self.set_value("ICRD", date)
	}
	fn remove_date(&mut self) {
		self.remove_key("ICRD")
	}

	fn copyright(&self) -> Option<&str> {
		self.get_value("ICOP")
	}
	fn set_copyright(&mut self, copyright: &str) {
		self.set_value("ICOP", copyright)
	}
	fn remove_copyright(&mut self) {
		self.remove_key("ICOP")
	}

	fn genre(&self) -> Option<&str> {
		self.get_value("IGNR")
	}
	fn set_genre(&mut self, genre: &str) {
		self.set_value("IGNR", genre)
	}
	fn remove_genre(&mut self) {
		self.remove_key("IGNR")
	}

	fn album_title(&self) -> Option<&str> {
		self.get_value("IPRD").or_else(|| self.get_value("ALBU"))
	}
	fn set_album_title(&mut self, title: &str) {
		self.set_value("IPRD", title)
	}
	fn remove_album_title(&mut self) {
		self.remove_key("IPRD")
	}

	fn track_number(&self) -> Option<u32> {
		if let Some(Ok(track_num)) = self
			.get_value("ITRK")
			.or_else(|| self.get_value("IPRT"))
			.or_else(|| self.get_value("TRAC"))
			.map(str::parse::<u32>)
		{
			return Some(track_num);
		}

		None
	}

	fn set_track_number(&mut self, track_number: u32) {
		self.set_value("ITRK", track_number.to_string())
	}

	fn remove_track_number(&mut self) {
		self.remove_key("ITRK")
	}

	fn total_tracks(&self) -> Option<u32> {
		if let Some(Ok(total_tracks)) = self.get_value("IFRM").map(str::parse::<u32>) {
			return Some(total_tracks);
		}

		None
	}

	fn set_total_tracks(&mut self, total_track: u32) {
		self.set_value("IFRM", total_track.to_string())
	}

	fn remove_total_tracks(&mut self) {
		self.remove_key("IFRM")
	}

	fn disc_number(&self) -> Option<u32> {
		if let Some(Ok(disc_number)) = self.get_value("DISC").map(str::parse::<u32>) {
			return Some(disc_number);
		}

		None
	}

	fn set_disc_number(&mut self, disc_number: u32) {
		self.set_value("DISC", disc_number.to_string())
	}

	fn remove_disc_number(&mut self) {
		self.remove_key("DISC")
	}

	fn total_discs(&self) -> Option<u32> {
		self.disc_number()
	}

	fn set_total_discs(&mut self, total_discs: u32) {
		self.set_disc_number(total_discs)
	}

	fn remove_total_discs(&mut self) {
		self.remove_disc_number()
	}
}

impl AudioTagWrite for RiffTag {
	fn write_to(&self, file: &mut File) -> Result<()> {
		riff::write_to(file, self.inner.data.clone())
	}
}
