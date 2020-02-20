#![allow(dead_code)]
#![feature(map_first_last)]

mod journal;

#[macro_use]
extern crate clap;

use chrono::{Duration, Local, NaiveDate};
use clap::ArgMatches;
use journal::Journal;

fn main() {
	#[allow(deprecated)]
	let matches = clap_app! { journal =>
		(version: crate_version!())
		(author: crate_authors!(",\n"))
		(about: crate_description!())
		(@arg USER: -u --user +takes_value +global
			"User whose entries will be managed. \
			 Defaults to the host computer's name.")
		(@arg ENTRY: +takes_value
			"Message to be logged.")
		(@subcommand view =>
			(about:
				"Displays journal entries.\n\
				 Dates must be in the format YYYY-mm-dd.")
			(@arg ENTRIES: -n --limit +takes_value
				"Limits the number of daily entries to be displayed.")
			(@group period =>
				(@arg DATES: +takes_value ...
					"The date(s) to view logs from. Defaults to today.")
				(@arg RANGE: -r --range +takes_value #{1, 2}
					"Takes up to two dates to form the (inclusive) range from \
					 which to view logs. If only one value is provided, it \
					 defaults to [date, today].")
			)
		)
		(@subcommand remove =>
			(about:
				"Removes entries from today, starting from the latest.")
			(@group amount =>
				(@arg ALL: --all
					"Removes all entries.")
				(@arg ENTRIES: -n +takes_value
					"Amount of entries to be removed. Defaults to the latest.")
			)
		)
	}
	.get_matches();
	let today = Local::today().naive_local();
	let user = matches.value_of_os("USER");
	let mut journal = match Journal::open(user, today) {
		Ok(journal) => journal,
		Err(_) => {
			eprintln!("Failed to open journal. Aborting...");
			return;
		}
	};
	let (subcmd, submatches) = matches.subcommand();
	let no_submatch = ArgMatches::default();
	let submatches = submatches.unwrap_or(&no_submatch);
	match subcmd {
		"view" => {
			let limit = value_t!(submatches.value_of("ENTRIES"), usize).ok();
			if submatches.is_present("RANGE") {
				let mut dates =
					values_t!(submatches.values_of("RANGE"), NaiveDate).unwrap_or_default();
				match dates.len() {
					1 => dates.push(today),
					2 => (),
					_ => return,
				}
				for days in 0..=(dates[1] - dates[0]).num_days() {
					journal.view(dates[0] + Duration::days(days), limit);
				}
			} else if let Some(dates) = submatches.values_of("DATES") {
				for date in dates {
					if let Ok(date) = date.parse() {
						journal.view(date, limit);
					} else {
						return;
					}
				}
			} else {
				journal.view(today, limit);
			}
		}
		"remove" => {
			let limit = if submatches.is_present("ALL") {
				usize::max_value()
			} else {
				value_t!(submatches.value_of("ENTRIES"), usize).unwrap_or(1)
			};
			journal.remove(limit);
		}
		_ => {
			if let Some(entry) = matches.value_of("ENTRY") {
				journal.add_entry(entry);
			}
		}
	}
}
