#[macro_use]
extern crate nom;

//mod parser;

use std::io;
use std::io::BufRead;
use std::io::StdinLock;
use std::str::SplitWhitespace;

#[derive(Debug, PartialEq)]
pub enum Loc {
  Left,
  Right,
  Up,
  Down,
  Acc,
  Last,
}

#[derive(Debug, PartialEq)]
pub enum Source {
  Val(i16),
  Loc(Loc),
}

type Label = String;

#[derive(Debug, PartialEq)]
pub enum Instr {
  Nop,
  Mov(Source, Loc),
  Swp,
  Sav,
  Add(Source),
  Sub(Source),
  Neg,
  Jmp(Label),
  Jez(Label),
  Jnz(Label),
  Jgz(Label),
  Jlz(Label),
  Jro(Source),
  Label(String),
  Comment(String),
}

struct Program {
  instrs: Vec<Instr>
}

struct Spec {
  programs: Vec<Program>
}

pub fn parse_loc(s: &str) -> Result<Loc, String> {
  match s {
    "LEFT" => { Ok(Loc::Left) }
    "RIGHT" => { Ok(Loc::Right) }
    "UP" => { Ok(Loc::Up) }
    "DOWN" => { Ok(Loc::Down) }
    "ACC" => { Ok(Loc::Acc) }
    "LAST" => { Ok(Loc::Last) }
    s => { Err(format!("Invalid loc {}", s)) }
  }
}

pub fn parse_lit(s: &str) -> Result<i16, String> {
  s.parse::<i16>().map_err(|e| e.to_string())
}

pub fn parse_source(s: &str) -> Result<Source, String> {
  parse_loc(s)
      .map(Source::Loc)
      .or_else(|_| parse_lit(s).map(Source::Val))
}

fn eat_comma(s: &str) -> Result<&str, String> {
  if !s.ends_with(",") {
    Err("invalid mov instruction with no comma".to_string())
  } else {
    Ok(&s[..s.len() - 1])
  }
}

pub fn parse_mov(words: &mut SplitWhitespace) -> Result<Instr, String> {
  let src = try!(
    words.next()
         .ok_or("invalid mov instruction without source".to_string())
         .and_then(eat_comma)
         .and_then(parse_source));
  let dest = try!(
    words.next()
         .ok_or("invalid mov instruction without dest".to_string())
         .and_then(parse_loc));
  Ok(Instr::Mov(src, dest))
}

fn parse_spec(buf: &mut BufRead) -> Result<i32, String> {
  let mut section = 0;
  for line in buf.lines() {
    let line = line.unwrap();
    let mut words = line.split_whitespace();
    let instr = match words.next() {
      Some("NOP") => { Ok(Instr::Nop) }
      Some("MOV") => { parse_mov(&mut words) }
      Some("SWP") => { Ok(Instr::Swp) }
      Some("SAV") => { Ok(Instr::Sav) }
      Some("ADD") => { words.next()
                            .ok_or("invalid add instr without source".to_string())
                            .and_then(parse_source)
                            .map(Instr::Add) }
      Some("SUB") => { words.next()
                            .ok_or("invalid sub instr without source".to_string())
                            .and_then(parse_source)
                            .map(Instr::Sub) }
      Some("NEG") => { Ok(Instr::Neg) }
      Some("JMP") => { words.next()
                            .ok_or("invalid jmp instr without label".to_string())
                            .map(str::to_string)
                            .map(Instr::Jmp) }
      Some("JEZ") => { words.next()
                            .ok_or("invalid jez instr without label".to_string())
                            .map(str::to_string)
                            .map(Instr::Jez) }
      Some("JNZ") => { words.next()
                            .ok_or("invalid jnz instr without label".to_string())
                            .map(str::to_string)
                            .map(Instr::Jnz) }
      Some("JGZ") => { words.next()
                            .ok_or("invalid jgz instr without label".to_string())
                            .map(str::to_string)
                            .map(Instr::Jgz) }
      Some("JLZ") => { words.next()
                            .ok_or("invalid jlz instr without label".to_string())
                            .map(str::to_string)
                            .map(Instr::Jlz) }
      Some("JRO") => { words.next()
                            .ok_or("invalid jro instr without source".to_string())
                            .and_then(parse_source)
                            .map(Instr::Jro) }
      Some(s) => {
        if s.starts_with("#") {
          Ok(Instr::Comment(words.fold(s.to_string(), |acc, s| acc + " " + s)))
        } else if s.starts_with("@") {
          // Handle errors here - fail if the line is @3 fz
          section = s[1..].parse::<u8>().unwrap();
          continue;
        } else if s.ends_with(":") {
          Ok(Instr::Label(s[..s.len() - 1].to_string()))
        } else {
          Err(format!("invalid instr {}", s))
        }
      }
      None => { Ok(Instr::Comment("".to_string())) }
    };
  }
  Ok(1)
}

fn main() {
  let stdin = io::stdin();
  let stdin = &mut stdin.lock();

  let spec = parse_spec(stdin).unwrap();
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_lit() {
    assert_eq!(Ok(3),  parse_lit("3"));
    assert!(parse_lit("f").is_err());
  }

  #[test]
  fn test_parse_loc() {
    assert_eq!(Ok(Loc::Acc),  parse_loc("ACC"));
    assert!(parse_loc("Aflkj").is_err());
  }

  #[test]
  fn test_parse_source() {
    assert_eq!(Ok(Source::Loc(Loc::Acc)),  parse_source("ACC"));
    assert_eq!(Ok(Source::Val(3)), parse_source("3"));
    assert!(parse_source("Aflkj").is_err());
  }

  #[test]
  fn test_parse_mov() {
    assert_eq!(Ok(Instr::Mov(Source::Val(3), Loc::Acc)),  parse_mov(&mut "3, ACC".split_whitespace()));
  }
}
