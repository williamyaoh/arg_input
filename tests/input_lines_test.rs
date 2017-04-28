extern crate arg_input;

mod inputs;

use inputs::{attach_input_dir, INPUTS, NONEXISTENT};

#[test]
fn test_input() {
  let filenames = INPUTS.iter().map(|str| {
    attach_input_dir(str)
  });

  let all_input = arg_input::input_lines(filenames);

  assert!(all_input.is_ok());

  let all_input = all_input.unwrap();

  for (i, line) in all_input.enumerate() {
    assert!(line.is_ok());

    let line_text = line.unwrap();
    
    assert_eq!(&line_text, INPUTS[i]);
  }
}

#[test]
fn test_input_lines_nonexistent() {
  let filenames = NONEXISTENT.iter().map(|str| {
    attach_input_dir(str)
  });

  let all_input = arg_input::input_lines(filenames);

  match all_input {
    Ok(_) => panic!("input_lines() should not have found these files"),
    Err(errs) => assert_eq!(errs.len(), NONEXISTENT.len())
  }
}
