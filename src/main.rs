extern crate chrono;
extern crate chrono_humanize;
extern crate colored;
extern crate failure;
extern crate git2;
#[macro_use]
extern crate structopt;

use chrono::{NaiveDateTime, TimeZone, Utc};
use colored::*;
use failure::Error;
use git2::{BranchType, Repository};
use std::env;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "branches", about = "Slightly more useful than `git branches`")]
struct Args {
    /// Sort in alphabetical order rather than by age
    #[structopt(short = "a", long = "alphabetical")]
    alphabetical: bool,
    /// Reverse sort order
    #[structopt(short = "r", long = "reverse")]
    reverse: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();
    let cur_dir = env::current_dir().expect("Unable to get current directory");
    let repo = Repository::discover(cur_dir).expect("Unable to open repo");
    let branches = repo.branches(Some(BranchType::Local))?;
    let now = Utc::now();
    let mut rv = vec![];
    for branch in branches {
        let (branch, _) = branch?;

        let last_commit = branch.get().peel_to_commit()?;

        let commit_time = last_commit.time();
        let message = last_commit
            .message()
            .and_then(|m| m.split('\n').next())
            .unwrap_or_default()
            .to_string();
        let commit_time = NaiveDateTime::from_timestamp(commit_time.seconds() as i64, 0);
        let commit_time = Utc
            .from_local_datetime(&commit_time)
            .single()
            .expect("Ambiguous timestamp");

        rv.push((branch, commit_time, message));
    }
    if !args.alphabetical {
        rv.sort_unstable_by_key(|v| v.1);
    }
    if args.reverse {
        rv.reverse();
    }

    let rv: Vec<_> = rv.into_iter().map(|(b, commit_time, m)| (b, commit_time, format!("{}", chrono_humanize::HumanTime::from(commit_time)), m)).collect();

    let branch_name_width = rv
        .iter()
        .filter_map(|row| row.0.name().ok().and_then(|x| x).map(str::len))
        .max()
        .unwrap_or(0);

    let date_width = rv
        .iter()
        .map(|row| row.2.len())
        .max()
        .unwrap_or(0);

    for (branch, commit_time, commit_str, message) in rv {
        let line = format!(
            "{:>width$} [{:>date_width$}] {}",
            branch.name()?.unwrap_or(""),
            commit_str,
            message,
            width = branch_name_width,
            date_width = date_width
        );
        let diff = now - commit_time;
        if diff.num_weeks() > 4 {
            println!("{}", line.red());
        } else if diff.num_weeks() >= 1 {
            println!("{}", line.yellow());
        } else if diff.num_days() >= 3 {
            println!("{}", line.bright_blue());
        } else if diff.num_days() >= 1 {
            println!("{}", line.green());
        } else {
            println!("{}", line.bright_green());
        }
    }
    Ok(())
}
