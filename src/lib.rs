//! Inspired by Ruby's [`ARGF`](https://ruby-doc.org/core-1.9.3/ARGF.html).
//! Treat files and `stdin` as if they were a big long concatenated stream.
//!
//! [`argf()`](fn.argf.html) will pull input from your command line arguments,
//! no frills, no questions asked, and [`argf_lines()`](fn.argf_lines.html) will
//! give you an iterator over all *lines* of command line input.
//!
//! `argf()` and `argf_lines()` assume that the command line arguments contain **only**
//! file arguments. If you need a little more control (for example, you're using `docopt`
//! to parse command line arguments instead), use [`input()`](fn.input.html) or
//! [`input_lines()`](fn.input_lines.html)

use std::env::args_os;
use std::iter::ExactSizeIterator;
use std::io::{self, Read};
use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::convert::From;

#[derive(Debug)]
pub struct FailReadFileError {
  pub inner: io::Error,
  pub filename: String
}

impl Display for FailReadFileError {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    writeln!(f, "could not read file {}", self.filename)?;
    writeln!(f, "caused by: {}", self.inner)?;
    Ok(())
  }
}

impl Error for FailReadFileError {
  fn description(&self) -> &str {
    "failed to read file"
  }

  fn cause(&self) -> Option<&Error> {
    Some(&self.inner)
  }
}

#[derive(Debug)]
pub struct InputError {
  pub badfiles: Vec<FailReadFileError>
}

impl Display for InputError {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    for e in &self.badfiles {
      writeln!(f, "{}", e)?;
    }
    Ok(())
  }
}

impl Error for InputError {
  fn description(&self) -> &str {
    "failed to read one or more files"
  }

  fn cause(&self) -> Option<&Error> {
    let first = self.badfiles.first();

    // There's some weird casting that I have to do here
    // that I don't fully understand.
    match first {
      Some(err) => Some(err),
      None => None
    }
  }
}

impl From<Vec<FailReadFileError>> for InputError {
  fn from(err: Vec<FailReadFileError>) -> Self {
    InputError { badfiles: err }
  }
}

/// Add the attempt_map() function to all iterators.
trait TryIterator {
  type Item;
  type JIter: ExactSizeIterator<Item=Self::Item>;

  /// Attempt to map the function over the given iterator, which might fail.
  /// If all attempts succeed, give back all the success. Otherwise, give
  /// back all the errors.
  fn attempt_map<F, T, E>(self, mapper: F) -> Result<Vec<T>, Vec<E>> where
    F: Fn(Self::Item) -> Result<T, E>;
}

impl<I> TryIterator for I where
  I: ExactSizeIterator 
{
  type Item = I::Item;
  type JIter = I;

  fn attempt_map<F, T, E>(self, mapper: F) -> Result<Vec<T>, Vec<E>> where
    F: Fn(Self::Item) -> Result<T, E>
  {
    let mut any_failure = false;
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for obj in self {
      match mapper(obj) {
        Ok(output) => {
          if !any_failure {
            successes.push(output);
          }
        },
        Err(err) => {
          any_failure = true;
          failures.push(err);
        }
      };
    }

    if any_failure { Err(failures) } else { Ok(successes) }
  }
}

pub type Lines = io::Lines<BufReader<Box<Read>>>;

/// Act like [`input_lines()`](fn.input_lines.html), but automatically
/// pull arguments from the command line. 
///
/// See [`argf()`](fn.argf.html) for caveats.
pub fn argf_lines() -> Result<Lines, InputError> {
  let chained = argf()?;
  let buffered = BufReader::new(chained);

  Ok(buffered.lines())
}

/// Act like [`input()`](fn.input.html), but automatically pull arguments
/// from the command line.
///
/// Assumes that the command line arguments are undisturbed (i.e., the first
/// argument is the executable name) and that all other arguments should be
/// treated like file names. If this is not the case and you need more fine-grained
/// control (e.g. you're using `docopt` to parse command-line arguments instead),
/// use `input()`.
pub fn argf() -> Result<Box<Read>, InputError> {
  let args = args_os().skip(1);
  input(args)
}

/// Return an iterator over all lines of input. 
///
/// See [`input()`](fn.input.html) for how this handles its arguments/errors.
pub fn input_lines<I, J, S>(inputs: I) -> Result<Lines, InputError> where
  I: IntoIterator<Item=S, IntoIter=J>,
  J: ExactSizeIterator<Item=S>,
  S: AsRef<Path>
{
  let chained = input(inputs)?;
  let buffered = BufReader::new(chained);

  Ok(buffered.lines())
}

/// Return a `Read` instance with all the input files/`stdin` chained together.
///
/// If any of the files fail to open, returns a `Vec` of all the IO errors
/// instead.
///
/// If *no* files are specified as inputs, this reads solely from `stdin`.
/// Otherwise, ignores `stdin` and concatenates the contents of all files
/// specified as arguments.
/// The argument "-" is special, and is an alias for `stdin`; this can be
/// used to reinsert `stdin` into the contents returned, if so desired.
pub fn input<I, J, S>(inputs: I) -> Result<Box<Read>, InputError> where
  I: IntoIterator<Item=S, IntoIter=J>,
  J: ExactSizeIterator<Item=S>,
  S: AsRef<Path>
{
  let iter = inputs.into_iter();

  if iter.len() == 0 {
    Ok(Box::new(io::stdin()))
  } else {
    let reads = iter.attempt_map(|path| from_arg(path.as_ref()))?;

    Ok(chain_all_reads(reads))
  }
}

fn chain_all_reads<I>(reads: I) -> Box<Read> where
  I: IntoIterator<Item=Box<Read>>
{
  reads.into_iter().fold(Box::new(io::empty()), |read, next| {
    Box::new(read.chain(next))
  })
}

fn from_arg<'a>(arg: &'a Path) -> Result<Box<Read>, FailReadFileError> {
  let str_repr = arg.to_string_lossy();
  if str_repr == "-" {
    Ok(Box::new(io::stdin()))
  } else {
    let file = File::open(arg).map_err(|err| {
      FailReadFileError {
        inner: err,
        filename: arg.to_string_lossy().to_string()
      }
    })?;
    Ok(Box::new(file))
  }
}
