extern crate arg_input;

mod inputs;

use std::io::Read;

use inputs::{attach_input_dir, INPUTS, NONEXISTENT};

#[test]
fn test_input() {
  let filenames = INPUTS.iter().map(|str| {
    attach_input_dir(str)
  });

  let all_input = arg_input::input(filenames);

  assert!(all_input.is_ok());

  let mut all_input = all_input.unwrap();
  let mut result_string = String::new();

  let mut comparison_string = String::new();

  for contents in INPUTS.iter() {
    comparison_string += contents;
    comparison_string += "\n";
  }

  let result = all_input.read_to_string(&mut result_string);

  assert!(result.is_ok());
  assert_eq!(result_string, comparison_string);
}

#[test]
fn test_input_nonexistent() {
  let filenames = NONEXISTENT.iter().map(|str| {
    attach_input_dir(str)
  });

  let all_input = arg_input::input(filenames);

  assert!(all_input.is_err());
}
