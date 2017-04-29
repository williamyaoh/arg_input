use std::path::PathBuf;

pub static INPUTS: [&'static str; 5] = ["A", "B", "C", "D", "E"];
pub static NONEXISTENT: [&'static str; 3] = ["Z", "Y", "X"];

pub fn attach_input_dir<'a>(input_name: &'a str) -> PathBuf {
  let mut fullpath = PathBuf::new();

  fullpath.push(".");
  fullpath.push("tests");
  fullpath.push("inputs");
  fullpath.push(input_name);

  fullpath
}
