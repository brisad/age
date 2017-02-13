extern crate chrono;

use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::env;

use chrono::*;

#[derive(Debug)]
struct Person {
    name: String,
    birth: Date<UTC>,
}

impl Person {
    fn from_str(s: &str) -> Result<Person, &str> {
        let fields: Vec<&str> = s.split_whitespace().collect();

        if fields.len() == 0 {
            return Err("No data");
        }

        let birth = fields[0];
        let name = if fields.len() == 1 { "You" } else { fields[1] };

        let parts: Vec<u32> = birth.split("-")
            .map(|y| y.parse::<u32>().unwrap())
            .collect();
        if parts.len() != 3 {
            return Err("Invalid date");
        }

        let birth =
            try!(UTC.ymd_opt(parts[0] as i32, parts[1], parts[2]).single().ok_or("Invalid date"));

        Ok(Person {
            name: name.to_string(),
            birth: birth,
        })
    }

    fn days(&self) -> i32 {
        let today = UTC::now().date();
        let duration = today - self.birth;
        duration.num_days() as i32
    }

    fn years(&self) -> i32 {
        let today = UTC::now().date();
        let year_diff = today.year() - self.birth.year();

        if (today.month() < self.birth.month()) ||
           (today.month() == self.birth.month() && today.day() < self.birth.day()) {
            year_diff - 1
        } else {
            year_diff
        }
    }

    fn next_birthday_from_year(&self, mut year: i32) -> Date<UTC> {
        let mut next = UTC.ymd_opt(year, self.birth.month(), self.birth.day());
        while next == LocalResult::None {
            year += 1;
            next = UTC.ymd_opt(year, self.birth.month(), self.birth.day());
        }
        next.single().unwrap()
    }

    fn days_until_next_birthday(&self) -> i32 {
        let today = UTC::now().date();
        let mut next = self.next_birthday_from_year(today.year());
        if next < today {
            next = self.next_birthday_from_year(next.year() + 1);
        }
        (next - today).num_days() as i32
    }
}

fn read_birthdays(path: &Path) -> io::Result<Vec<Person>> {
    let mut file = try!(File::open(path));

    let mut s = String::new();
    file.read_to_string(&mut s).expect("Couldn't read file");

    Ok(s.lines().filter_map(|line| Person::from_str(line).ok()).collect())
}

fn print(persons: &[Person], as_days: bool) {
    for person in persons {
        let counter = if as_days { "day" } else { "year" };
        let amount = if as_days {
            person.days()
        } else {
            person.years()
        };

        println!("{} {} {} {}{} old",
                 person.name,
                 if person.name == "You" { "are" } else { "is" },
                 amount,
                 counter,
                 if amount > 1 { "s" } else { "" });
    }
}

fn print_long(persons: &[Person], as_days: bool) {
    println!("{:20}{:>5}{:>15}{:>18}",
             "Name",
             "Age",
             "Birthdate",
             "Days remaining");
    for p in persons {
        println!("{:20}{:>5}{:>15}{:>18}",
                 p.name,
                 if as_days { p.days() } else { p.years() },
                 p.birth.format("%Y-%m-%d").to_string(),
                 p.days_until_next_birthday());
    }
}

macro_rules! usage {
    () => (
        println!("Usage: age [-adhlsw]
Prints your age
	-a	also print ages of other people
	-d	output age in days
	-h	print this help and exit
	-l	long output
	-s	sort in birthday order
	-w	warn if someone has birthday soon");
        return
    )
}

const DATA_FILE: &'static str = ".age";

fn main() {
    let mut as_days = false;
    let mut all = false;
    let mut long_output = false;
    let mut sort = false;
    let mut warn = false;

    for arg in env::args().skip(1) {
        let mut chars = arg.chars().into_iter();

        if chars.next().unwrap() != '-' {
            usage!();
        }
        for c in chars {
            match c {
                'a' => all = true,
                'd' => as_days = true,
                'l' => long_output = true,
                's' => sort = true,
                'w' => warn = true,
                _ => {
                    usage!();
                }
            }
        }
    }

    let path_opt = env::home_dir().map(|p| p.join(DATA_FILE));
    if path_opt.is_none() {
        println!("Unable to determine home directory");
        return;
    }
    let path = path_opt.unwrap();

    let read_result = read_birthdays(&path);
    if read_result.is_err() {
        println!("Unable to read file '{}'", path.display());
        return;
    }
    let mut persons = read_result.unwrap();

    if warn {
        for p in &persons {
            let remaining = p.days_until_next_birthday();
            match remaining {
                0 => println!("Warning: {}'s birthday is today", p.name),
                1 => println!("Warning: {}'s birthday is tomorrow", p.name),
                2...14 => println!("Warning: {}'s birthday is in {} days", p.name, remaining),
                _ => {}
            }
        }
    }

    if !all {
        persons.retain(|p| p.name == "You");
    }

    if sort {
        persons.sort_by_key(|p| (p.birth.month(), p.birth.day()));
    }

    if long_output {
        print_long(&persons, as_days);
    } else {
        print(&persons, as_days);
    }
}

#[cfg(test)]
mod test {
    use super::Person;
    use chrono::*;

    #[test]
    fn parse_valid_date() {
        let person = Person::from_str("1970-02-20").unwrap();
        assert_eq!("You", person.name);
        assert_eq!(UTC.ymd(1970, 2, 20), person.birth);
    }

    #[test]
    fn parse_other_persons_date() {
        let person = Person::from_str("1980-05-10 Anne").unwrap();
        assert_eq!("Anne", person.name);
        assert_eq!(UTC.ymd(1980, 5, 10), person.birth);
    }

    #[test]
    fn parse_empty_string() {
        let error = Person::from_str("").unwrap_err();
        assert_eq!("No data", error);
    }

    #[test]
    fn parse_invalid_date() {
        let error = Person::from_str("1980-0510").unwrap_err();
        assert_eq!("Invalid date", error);
    }

    #[test]
    fn parse_invalid_date2() {
        let error = Person::from_str("1980-20-10").unwrap_err();
        assert_eq!("Invalid date", error);
    }

    #[test]
    fn parse_with_trailing_strings() {
        let person = Person::from_str("1980-05-10 Anne X Y Z").unwrap();
        assert_eq!("Anne", person.name);
        assert_eq!(UTC.ymd(1980, 5, 10), person.birth);
    }

    #[test]
    fn no_panic_for_special_date() {
        let person = Person::from_str("1984-02-29 Ben").unwrap();
        assert!(person.days_until_next_birthday() > 0)
    }
}
