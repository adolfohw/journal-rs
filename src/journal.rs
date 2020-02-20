use chrono::{Datelike, Local, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::{
	collections::BTreeMap,
	ffi::{OsStr, OsString},
	fs::{self, File, OpenOptions},
	io,
	path::PathBuf,
};

type Entries = BTreeMap<NaiveDateTime, String>;

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Journal {
	#[serde(skip)]
	user: OsString,

	#[serde(skip)]
	#[serde(default = "naive_now")]
	date: NaiveDate,

	entries: Entries,
}

impl Journal {
	pub fn open(user: Option<&OsStr>, date: NaiveDate) -> io::Result<Self> {
		let user = match user.map(ToOwned::to_owned) {
			Some(user) => user,
			None => hostname::get()?,
		};
		let path = make_path(&user, &date, false);
		let maybe_file = OpenOptions::new()
			.read(true)
			.append(true)
			.create(true)
			.open(path);
		let mut journal: Self = maybe_file
			.map(|file| serde_json::from_reader(file).unwrap_or_default())
			.unwrap_or_default();
		journal.user = user;
		journal.date = date;
		Ok(journal)
	}

	pub fn add_entry(&mut self, entry: &str) {
		let now = Local::now().naive_local();
		if self.date != now.date() {
			return;
		}
		make_path(&self.user, &self.date, true);
		self.entries.insert(now, entry.into());
		println!(
			"Entry added to day {} of {}'s journal",
			self.date,
			self.user.to_string_lossy()
		);
	}

	pub fn view(&self, date: NaiveDate, limit: Option<usize>) {
		let path = make_path(&self.user, &date, false);
		if let Ok(file) = File::open(&path) {
			let entries = serde_json::from_reader::<File, Entries>(file).unwrap_or_default();
			let num_entries = entries.len();
			let limit = limit.unwrap_or(usize::max_value());
			let mut counter = 0;
			for (datetime, entry) in entries {
				if counter >= limit {
					println!("... {} entries omitted ...", num_entries - limit);
					return;
				}
				println!(
					"[{} @ {}] > {}",
					self.user.to_string_lossy(),
					datetime.format("%Y-%m-%d %H:%M:%S"),
					entry
				);
				counter += 1;
			}
		}
	}

	pub fn remove(&mut self, limit: usize) {
		let max_entries = self.entries.len();
		if limit >= max_entries {
			self.entries.clear();
		} else {
			for _ in 0..limit {
				if let Some(entry) = self.entries.last_entry() {
					entry.remove_entry();
				}
			}
		}
		let removed = limit.min(max_entries);
		println!(
			"Removed {} entr{} from day {} of {}'s journal",
			removed,
			if removed > 1 { "ies" } else { "y" },
			self.date,
			self.user.to_string_lossy()
		);
	}

	pub fn close(self) {}
}

impl Default for Journal {
	fn default() -> Self {
		Self {
			user: OsString::new(),
			date: naive_now(),
			entries: BTreeMap::new(),
		}
	}
}

impl Drop for Journal {
	#[allow(unused_must_use)]
	fn drop(&mut self) {
		let path = make_path(&self.user, &self.date, false);
		if self.entries.is_empty() {
			fs::remove_file(&path);
			for parent in path.ancestors() {
				fs::remove_dir(parent);
			}
		} else {
			File::create(path).map(|file| serde_json::to_writer_pretty(file, self));
		}
	}
}

fn naive_now() -> NaiveDate {
	Local::today().naive_local()
}

#[allow(unused_must_use)]
fn make_path(user: &OsString, date: &NaiveDate, mkdir: bool) -> PathBuf {
	let mut path = PathBuf::from(dirs::home_dir().unwrap());
	path.push(".journals");
	path.push(user);
	path.push(date.year().to_string());
	path.push(date.month().to_string());
	if mkdir {
		fs::create_dir_all(&path);
	}
	path.push(date.day().to_string());
	path.set_extension("json");
	path
}
