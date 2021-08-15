// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.

//! Simple INI configuration file format
//!
//! - groups are created by naming them within brackets.
//! - an equal sign ('=') is used for variable assignment.
//! - variables can contain a '=' character.
//! - leading and trailing whitespace is ignored.
//! - whitespace surrounding group names, variables, and values are removed.
//! - whitespace within group names, variable names, and values is allowed.
//! - a semicolon (';') at the beginning of a line denotes a comment.
//! - if a variable is set multiple times in a file, the last one read is kept.
//!
//! Multiple INI files can be merged into a single [Config]; variables read in a
//! later file replace any set in prior configuration files.
//!
//! The configuration values can be modified via the [Config::set] method; `set`
//! also provides a convenient API for setting default values prior to reading a
//! configuration file.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    ops::Index,
    fmt,
    io,
};

#[cfg(feature = "config_global")]
use once_cell::sync::OnceCell;

use super::iter::uniq::Uniq;


#[cfg(feature = "config_global")]
static CONFIG: OnceCell<Config> = OnceCell::new();

#[cfg(feature = "config_global")]
/// Make the provided [Config] available globally.
///
/// This function can only be called once; if called again, it will return an
/// error containing the provided `Config` object.
///
/// Use [Config::global] to obtain the global `Config` instance.
///
/// # Example
///
/// ```
/// # use utility_belt::config::{Config, set_global};
/// let conf = Config::default();
/// set_global(conf).expect("Global configuration was already initialized");
/// let conf = Config::global();
/// ```
pub fn set_global(conf: Config) -> Result<(), Config> {
    CONFIG.set(conf)?;
    Ok(())
}

/// The key used to look up a configuration value.
///
/// The key is a group/variable pair. The default group is "DEFAULT".
// This is not efficient for a large number of keys.
type Key = (String, String);

/// The Configuration object.
#[derive(Debug, Default)]
pub struct Config {
    values: HashMap<Key, String>,
}

impl Config {
    #[cfg(feature = "config_global")]
    /// Retrieve the global [Config] instance.
    ///
    /// # Panics
    ///
    /// Panics if no global configuration has been initialized. Use [set_global]
    /// to register a global `Config`.
    ///
    /// # Example
    ///
    /// ```
    /// # use utility_belt::config::{Config, set_global};
    /// let conf = Config::default();
    /// set_global(conf).expect("Global configuration was already initialized");
    /// let conf = Config::global();
    /// ```
    pub fn global() -> &'static Config {
        CONFIG.get().expect("Global Config is not initialized")
    }

    /// Read a [Config] from the INI file at the path specified.
    ///
    /// # Returns
    ///
    /// Returns the configuration file if successfully read; otherwise returns a
    /// list of errors that occurred while reading or parsing the file.
    pub async fn read_from_file(path: &Path) -> Result<Self, Vec<FileError>> {
        use async_std::{
            fs::File,
            io::{prelude::*, BufReader},
        };

        let f = match File::open(path).await {
            Ok(f) => f,
            Err(e) => return Err(vec![e.into()]),
        };

        let mut reader = BufReader::new(f);
        let mut line = String::new();
        let mut cnt = 0;

        let mut errors = vec![];
        let mut map = HashMap::new();
        let mut group = String::from("DEFAULT");

        loop {
            match reader.read_line(&mut line).await {
                Ok(len) => if len == 0 { break; },
                Err(e) => {
                    errors.push(e.into());
                    continue;
                }
            };

            cnt += 1;
            line = line.trim().into();
            if line.is_empty() { continue; }

            if line.starts_with(';') {
                line.clear();
                continue;
            } else if line.starts_with('[') {
                if line.ends_with(']') {
                    group = line[1..line.len()-1].trim().into();
                } else {
                    errors.push(FileError::Parse {
                        file: path.to_owned(),
                        msg: "Missing closing bracket for group name".into(),
                        data: line.to_owned(),
                        line: cnt,
                    });
                }
            } else if let Some((var, val)) = line.split_once('=') {
                // We'll allow empty values, but not variables.
                let var = var.trim_end().to_string();

                if var.is_empty() {
                    errors.push(FileError::Parse {
                        file: path.to_owned(),
                        msg: "Assignment requires a variable name".into(),
                        data: line.to_owned(),
                        line: cnt,
                    });
                } else {
                    map.insert(
                        (group.clone(), var),
                        val.trim_start().to_string()
                    );
                }
            } else {
                errors.push(FileError::Parse {
                    file: path.to_owned(),
                    msg: "Expected a variable assignment".into(),
                    data: line.to_owned(),
                    line: cnt,
                });
            }
            line.clear();
        }

        if errors.is_empty() {
            Ok(Self { values: map })
        } else {
            Err(errors)
        }
    }

    /// Write this configuration to the given file. If the file exists, it is
    /// replaced with the contents of this configuration.
    pub fn write_to_file(&self, path: &Path) -> Result<(), FileError> {
        // TODO: Make this function async. As of async_std 1.9.0, writes are
        // incomplete and the function hangs indefinitely. Debug and file issue.
        use std::{
            fs::File,
            io::Write as _,
        };

        let mut file = File::create(path)?;

        for group in self.groups() {
            writeln!(file, "[{}]", group)?;

            for var in self.variables_in_group(group) {
                writeln!(
                    file,
                    "{} = {}",
                    var,
                    self[(group.as_str(), var.as_str())]
                )?;
            }
        }

        Ok(())
    }

    /// Merge two [Config]s, consuming both of the originals.
    ///
    /// Any duplicate variables will contain the values in `other`.
    pub fn merge_with(mut self, other: Self) -> Self {
        for (k, v) in other.values {
            self.values.insert(k, v);
        }
        self
    }

    /// Add the specified value to the configuration.
    ///
    /// `set` can be used to create default settings by setting values prior to
    /// reading a configuration.
    pub fn set(mut self, group: &str, var: &str, val: &str) -> Self {
        self.values.insert(
            (group.into(), var.into()),
            val.into()
        );
        self
    }

    /// Add the specified value to the configuration within the DEFAULT group.
    ///
    /// See [Config::set] for more information.
    pub fn set_default(self, var: &str, val: &str) -> Self {
        self.set("DEFAULT", var, val)
    }

    /// Get the list of groups in the configuration file.
    pub fn groups(&self) -> impl Iterator<Item = &String> {
        self.values.keys().uniq().map(|k| &k.0)
    }

    /// Get the list of variables set in the specified group.
    pub fn variables_in_group<'a>(&'a self, group: &'a str)
    -> impl Iterator<Item = &'a String> {
        self.values.keys()
            .filter_map(move |k| {
                if k.0 == group { Some(&k.1) } else { None }
            })
    }

    pub fn group_by_key_value<'a>(&'a self, group: &'a str) ->
        impl Iterator + 'a + Iterator<Item = (String, &String)>
    {
        self.variables_in_group(group)
            .map(move |v| self.values.get_key_value(
                &(group.to_owned(), v.to_owned())).unwrap()
            )
            .map(|(k, v)| (k.1.to_owned(), v))
    }

    /// Retrieve the value of the specified variable within the DEFAULT group,
    /// or `None` if it is not set.
    pub fn get_default(&self, variable: &str) -> Option<&String> {
        self.values.get(&("DEFAULT".into(), variable.into()))
    }

    /// Retrieve the value of the specified variable within the specified group,
    /// or `None` if it is not set.
    pub fn get(&self, group: &str, variable: &str) -> Option<&String> {
        self.values.get(&(group.into(), variable.into()))
    }
}

impl Index<&str> for Config {
    type Output = String;

    fn index(&self, variable: &str) -> &Self::Output {
        self.index(("DEFAULT", variable))
    }
}

impl Index<(&str, &str)> for Config
{
    type Output = String;

    fn index(&self, key: (&str, &str)) -> &Self::Output {
        &self.values[&(key.0.into(), key.1.into())]
    }
}

/// Error for file IO and parse errors.
#[derive(Debug, Clone)]
pub enum FileError {
    #[allow(clippy::upper_case_acronyms)]
    IO((PathBuf, io::ErrorKind)),
    Parse { file: PathBuf, msg: String, data: String, line: u32 },
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FileError::IO((ref file, ref e)) =>
                write!(f, "{:?} in file {}", e, file.to_string_lossy()),
            FileError::Parse { ref file, ref msg, ref data, ref line } =>
                write!(f, "{} at line {} in {}:\n\t{}"
                    , msg, line, file.to_string_lossy(), data),
        }
    }
}

impl std::error::Error for FileError {}

impl From<io::Error> for FileError {
    fn from(err: io::Error) -> FileError {
        FileError::IO((PathBuf::default(), err.kind()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[async_std::test]
    async fn parse_variables() {
        let conf = Config::read_from_file(Path::new("tests/test.ini"))
            .await.unwrap();

        assert_eq!(conf[("DEFAULT", "var1")], "val1");
        assert_eq!(conf[("Group A", "var2")], "value two");
        assert_eq!(conf[("Group A", "var 3")], "value = three");
    }

    #[async_std::test]
    async fn merge_configs() {
        let conf = Config::read_from_file(Path::new("tests/test.ini"))
            .await.unwrap()
            .merge_with(
                Config::read_from_file(Path::new("tests/test2.ini"))
                    .await.unwrap()
            );

        assert_eq!(conf[("DEFAULT", "var1")], "val1");
        assert_eq!(conf[("Group A", "var2")], "value two");
        assert_eq!(conf[("Group A", "var 3")], "value = four");
    }

    #[async_std::test]
    async fn test_write_to_file() {
        use async_std::fs::remove_file;

        let mut path = std::env::temp_dir();
        path.push("writing_test_config_file");
        path.set_extension("txt");

        let _ = remove_file(&path).await;


        let conf = Config::default()
            .set_default("var1", "value")
            .set("Some Group", "my variable", "my value");

        conf.write_to_file(&path).unwrap();
        println!("* conf: {:?}", conf);

        let read_conf = Config::read_from_file(&path).await.unwrap();
        assert_eq!(read_conf.get_default("var1"), Some(&"value".to_string()));
        assert_eq!(
            read_conf.get("Some Group", "my variable"),
            Some(&"my value".to_string())
        );

        let _ = remove_file(&path).await;
    }

    #[async_std::test]
    async fn nonexistent_file_is_err() {
        let conf = Config::read_from_file(Path::new("nopath/notexist.conf"))
            .await;
        assert!(conf.is_err());
    }

    #[async_std::test]
    async fn get_default_group() {
        let conf = Config::read_from_file(Path::new("tests/test.ini"))
            .await.unwrap();

        assert_eq!(conf.get_default("var1"), Some(&"val1".to_string()));
        assert_eq!(conf.get_default("nothing"), None);
    }

    #[async_std::test]
    async fn get_group() {
        let conf = Config::read_from_file(Path::new("tests/test.ini"))
            .await.unwrap();

        assert_eq!(conf.get("Group A", "var2"), Some(&"value two".to_string()));
        assert_eq!(conf.get("Group A", "var1"), None);
    }

    #[async_std::test]
    async fn get_nonexistent_group_is_none() {
        let conf = Config::read_from_file(Path::new("tests/test.ini"))
            .await.unwrap();

        assert!(conf.get("Not a group", "var1").is_none());
    }

    #[async_std::test]
    async fn set_default_values() {
        let conf = Config::default()
            .set_default("var1", "default value")
            .set_default("some-var", "my-value")
            .merge_with(
                Config::read_from_file(Path::new("tests/test.ini"))
                    .await.unwrap()
            );

        assert_eq!(conf["var1"], "val1");
        assert_eq!(conf["some-var"], "my-value");
    }

    #[async_std::test]
    async fn collect_all_parse_errors() {
        let conf = Config::read_from_file(Path::new("tests/invalid.ini")).await;
        let errs = conf.unwrap_err();
        let mut errs = errs.iter();

        match errs.next() {
            Some(FileError::Parse { file, msg, data, line }) => {
                assert!(*file == *PathBuf::from("tests/invalid.ini"));
                assert!(msg.contains("variable assignment"));
                assert_eq!(data, "some variable");
                assert_eq!(*line, 3);
            },
            _ => panic!("Expected a FileError::Parse"),
        }

        match errs.next() {
            Some(FileError::Parse { file, msg, data, line }) => {
                assert!(*file == *PathBuf::from("tests/invalid.ini"));
                assert!(msg.contains("variable name"));
                assert_eq!(data, "= some value");
                assert_eq!(*line, 5);
            },
            _ => panic!("Expected a FileError::Parse"),
        }

        match errs.next() {
            Some(FileError::Parse { file, msg, data, line }) => {
                assert!(*file == *PathBuf::from("tests/invalid.ini"));
                assert!(msg.contains("closing bracket"));
                assert_eq!(data, "[Bad Group");
                assert_eq!(*line, 7);
            },
            _ => panic!("Expected a FileError::Parse"),
        }

        match errs.next() {
            Some(FileError::Parse { file, msg, data, line }) => {
                assert!(*file == *PathBuf::from("tests/invalid.ini"));
                assert!(msg.contains("variable assignment"));
                assert_eq!(data, "# Bad comment");
                assert_eq!(*line, 9);
            },
            _ => panic!("Expected a FileError::Parse"),
        }

        assert!(errs.next().is_none());
    }
}
