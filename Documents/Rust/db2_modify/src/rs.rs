use std::str::FromStr;

use clap::{Args, Parser};

#[derive(Debug, Clone)]
enum Dsnum {
    Numeric(u16),
    Literal(String),
}

impl FromStr for Dsnum {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try u16 first
        if let Ok(dsnum) = s.parse::<u16>() {
            return Ok(Dsnum::Numeric(dsnum));
        }
        // Then accept literal values
        match s.to_ascii_uppercase().as_str() {
            "ALL" => Ok(Dsnum::Literal(s.to_string().to_ascii_uppercase())),
            _ => Err(format!("Invalid value '{s}': expected 1-4096 or ALL")),
        }
    }
}

#[derive(Args, Debug, Clone)]
#[group(
    id = "delete_target",
    required = false,
    multiple = false,
    requires = "delete"
)]
struct Delete {
    #[arg(long)]
    age: Option<Age>,
    #[arg(long)]
    date: Option<Date>,
}

#[derive(Debug, Clone)]
enum Age {
    Numeric(u16),
    Literal(String),
}

impl FromStr for Age {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(age) = s.parse::<u16>() {
            return Ok(Age::Numeric(age));
        }
        match s {
            "*" => Ok(Age::Literal(s.to_string())),
            _ => Err(format!("Invalid value '{s}': expected a number or *")),
        }
    }
}

#[derive(Debug, Clone)]
enum Date {
    Numeric(u32),
    Literal(String),
}

impl FromStr for Date {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(date) = s.parse::<u32>() {
            return Ok(Date::Numeric(date));
        }
        match s {
            "*" => Ok(Date::Literal(s.to_string())),
            _ => Err(format!("Invalid valid '{s}'. Expected a number or *")),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Modify {
    #[arg(short, long, required = true)]
    database: String,
    #[arg(short, long, required = true)]
    tablespace: String,
    #[arg(long)]
    dsnum: Option<Dsnum>,
    #[arg(long)]
    clone: bool,
    #[arg(long, requires = "delete_target", conflicts_with = "retain")]
    delete: bool,
    #[command(flatten)]
    delete_target: Delete,
    #[arg(long, conflicts_with = "delete")]
    retain: bool,
}

impl Modify {
    fn compose_statement(&self) -> String {
        let mut modify_statement = String::from("MODIFY RECOVERY");
        modify_statement = format!(
            "{} TABLESPACE {}.{}",
            modify_statement,
            self.database.to_uppercase(),
            self.tablespace.to_uppercase(),
        );

        if let Some(ref dsnum) = self.dsnum {
            match dsnum {
                Dsnum::Literal(value) => {
                    modify_statement = format!("{} DSNUM {}", modify_statement, value)
                }
                Dsnum::Numeric(n) => {
                    modify_statement = format!("{} DSNUM {}", modify_statement, n.to_string())
                }
            }
        } else {
            modify_statement = format!("{} DSNUM ALL", modify_statement)
        }

        match self.clone {
            true => modify_statement = format!("{} CLONE", modify_statement),
            false => {}
        };

        match (&self.delete_target.age, &self.delete_target.date) {
            (Some(Age::Numeric(n)), _) => {
                modify_statement = format!("{} DELETE AGE {} ", modify_statement, n)
            }
            (Some(Age::Literal(s)), _) => {
                modify_statement = format!("{} DELETE AGE({})", modify_statement, s)
            }
            (_, Some(Date::Numeric(n))) => {
                modify_statement = format!("{} DELETE DATE {}", modify_statement, n)
            }
            (_, Some(Date::Literal(s))) => {
                modify_statement = format!("{} DELETE DATE({})", modify_statement, s)
            }
            (_, _) => {}
        }

        modify_statement
    }
}

fn main() {
    let modify = Modify::parse();
    println!("{}", modify.compose_statement())
}
