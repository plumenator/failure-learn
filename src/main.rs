extern crate failure;
extern crate serde;
extern crate toml;

#[macro_use] extern crate failure_derive;
#[macro_use] extern crate matches;


use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use failure::Error;
use failure::ResultExt;

// This is a new error type that you've created. It represents the ways a
// toolchain could be invalid.
//
// The custom derive for Fail derives an impl of both Fail and Display.
// We don't do any other magic like creating new types.
#[derive(Debug, Fail)]
pub enum ToolchainError {
    #[fail(display = "invalid toolchain name: {}", name)]
    InvalidToolchainName {
        name: String,
    },
    #[fail(display = "unknown toolchain version: {}", version)]
    UnknownToolchainVersion {
        version: String,
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ToolchainId (String);

impl FromStr for ToolchainId {
    type Err = ToolchainError;

    fn from_str(_s: &str) -> Result<ToolchainId, ToolchainError> {
        Err(ToolchainError::InvalidToolchainName{name: String::from("An invalid toolchain")})
    }
}

pub type Toolchains = HashMap<ToolchainId, PathBuf>;

// This opens a toml file containing associations between ToolchainIds and
// Paths (the roots of those toolchains).
//
// This could encounter an io Error, a toml parsing error, or a ToolchainError,
// all of them will be thrown into the special Error type
pub fn read_toolchains(path: PathBuf) -> Result<Toolchains, Error>
{
    use std::fs::File;
    use std::io::Read;

    let mut string = String::new();
    File::open(path)?.read_to_string(&mut string)?;

    let toml: HashMap<String, PathBuf> = toml::from_str(&string)?;

    let toolchains = toml.iter().map(|(key, path)| {
        let toolchain_id = key.parse()?;
        Ok((toolchain_id, path.clone()))
    }).collect::<Result<Toolchains, ToolchainError>>()?;

    Ok(toolchains)
}

fn main() {
    println!("Backtrace {:?}", read_toolchains(PathBuf::from("Cargo.toml")).err().unwrap().backtrace());
    println!("Result Context {:?}", read_toolchains(PathBuf::from("Cargo.toml")).context("Reading toolchains").err().unwrap());
    println!("Falure context {:?}", read_toolchains(PathBuf::from("Cargo.toml")).err().unwrap().context("Reading toolchains"));
}

#[test]
fn invalid_path()
{
    let result = read_toolchains(PathBuf::from("/this/path/is/invalid"));
    assert!(matches!(result, Err(_)));
    use std::io;
    assert!(result.err().unwrap().cause().downcast_ref::<io::Error>().is_some());
}

#[test]
fn invalid_toml()
{
    let result = read_toolchains(PathBuf::from("Cargo.toml"));
    assert!(matches!(result, Err(_)));
    assert!(result.err().unwrap().cause().downcast_ref::<toml::de::Error>().is_some());
}
