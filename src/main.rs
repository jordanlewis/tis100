use std::io;
use std::io::BufRead;
use std::str::SplitWhitespace;
use std::collections::HashMap;

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

#[derive(Debug, PartialEq)]
struct InstrAndPos<'a> {
  instr: Instr<'a>,
  pos: i8,
}

#[derive(Debug, PartialEq)]
pub enum Instr<'a> {
  Nop,
  Mov(Source, Loc),
  Swp,
  Sav,
  Add(Source),
  Sub(Source),
  Neg,
  Jmp(&'a str),
  Jez(&'a str),
  Jnz(&'a str),
  Jgz(&'a str),
  Jlz(&'a str),
  Jro(Source),
  Comment(String),
  Emptyline,
  // This is a section switch marker and won't be included in a program.
  Section(u8),
}

#[derive(Debug)]
struct Program<'a> {
  instrs: Vec<InstrAndPos<'a>>,
  labels: HashMap<&'a str, usize>,
}

#[derive(Debug)]
struct Spec<'a> {
  programs: Vec<Program<'a>>,
}

pub fn parse_loc(s: &str) -> Result<Loc, String> {
  match s {
    "LEFT" => Ok(Loc::Left),
    "RIGHT" => Ok(Loc::Right),
    "UP" => Ok(Loc::Up),
    "DOWN" => Ok(Loc::Down),
    "ACC" => Ok(Loc::Acc),
    "LAST" => Ok(Loc::Last),
    s => Err(format!("Invalid loc {}", s)),
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

pub fn parse_mov<'a>(mut words: SplitWhitespace) -> Result<Instr<'a>, String> {
  let src = try!(words.next()
    .ok_or("invalid mov instruction without source".to_string())
    .and_then(eat_comma)
    .and_then(parse_source));
  let dest = try!(words.next()
    .ok_or("invalid mov instruction without dest".to_string())
    .and_then(parse_loc));
  Ok(Instr::Mov(src, dest))
}

pub fn parse_line<'a>(line: &'a str) -> Result<Instr, String> {
  let mut words = line.split_whitespace();
  match words.next() {
    Some("NOP") => Ok(Instr::Nop),
    Some("MOV") => parse_mov(words),
    Some("SWP") => Ok(Instr::Swp),
    Some("SAV") => Ok(Instr::Sav),
    Some("ADD") => {
      words.next()
        .ok_or("invalid add instr without source".to_string())
        .and_then(parse_source)
        .map(Instr::Add)
    }
    Some("SUB") => {
      words.next()
        .ok_or("invalid sub instr without source".to_string())
        .and_then(parse_source)
        .map(Instr::Sub)
    }
    Some("NEG") => Ok(Instr::Neg),
    Some("JMP") => {
      words.next()
        .ok_or("invalid jmp instr without label".to_string())
        .map(Instr::Jmp)
    }
    Some("JEZ") => {
      words.next()
        .ok_or("invalid jez instr without label".to_string())
        .map(Instr::Jez)
    }
    Some("JNZ") => {
      words.next()
        .ok_or("invalid jnz instr without label".to_string())
        .map(Instr::Jnz)
    }
    Some("JGZ") => {
      words.next()
        .ok_or("invalid jgz instr without label".to_string())
        .map(Instr::Jgz)
    }
    Some("JLZ") => {
      words.next()
        .ok_or("invalid jlz instr without label".to_string())
        .map(Instr::Jlz)
    }
    Some("JRO") => {
      words.next()
        .ok_or("invalid jro instr without source".to_string())
        .and_then(parse_source)
        .map(Instr::Jro)
    }
    Some(s) => {
      if s.starts_with("#") {
        Ok(Instr::Comment(words.fold(s.to_string(), |acc, s| acc + " " + s)))
      } else if s.starts_with("@") {
        s[1..]
          .parse::<u8>()
          .map_err(|e| e.to_string())
          .map(Instr::Section)
      } else {
        Err(format!("invalid instr '{}'", s))
      }
    }
    None => Ok(Instr::Emptyline),
  }
}

fn get_label<'a>(line: &'a str) -> (&'a str, Option<&str>) {
  match line.find(":") {
    Some(i) => (&line[i+1..], Some(&line[..i-1])),
    None => (&line, None),
  }
}

fn parse_spec(buf: &mut BufRead) -> Result<Spec, String> {
  let mut next_section = 0;
  let mut last_empty = false;
  let mut programs = Vec::new();
  let mut instrs = Vec::new();
  let mut labels = HashMap::new();
  let mut line_no = 0;
  for line in buf.lines() {
    line_no = line_no + 1;
    let line = &line.unwrap();
    let (line, label) = get_label(line);
    match label {
      Some(label) => {
        if labels.insert(label, instrs.len()) != None {
          return Err(format!("label {} already exists", label));
        }
      }
      None => {}
    }
    let instr = try!(parse_line(line));
    match instr {
      Instr::Section(i) => {
        if i != next_section {
          return Err(format!("expecting section {}, found section {}", next_section, i));
        } else if i > 11 {
          return Err(format!("section {} greater than maximum 11", i));
        }
        if next_section > 0 {
          // sections must end with an empty line
          if !last_empty {
            return Err(format!("invalid section: didn't end with an empty line"));
          }
          last_empty = false;
          programs.push(Program { instrs: instrs, labels: labels });
          instrs = Vec::new();
          labels = HashMap::new()
        }
        line_no = -1;
        next_section = next_section + 1;
      }
      Instr::Emptyline => { last_empty = true }
      i => {
        last_empty = false;
        if next_section == 0 {
          return Err(format!("missing @0 header, found {:?}", i));
        }
        instrs.push(InstrAndPos{instr: i, pos: line_no})
      }
    }
  }
  Ok(Spec { programs: programs })
}

fn main() {
  let stdin = io::stdin();
  let stdin = &mut stdin.lock();

  let spec = parse_spec(stdin).unwrap();
  println!("{:?}", spec);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_lit() {
    assert_eq!(Ok(3), parse_lit("3"));
    assert!(parse_lit("f").is_err());
  }

  #[test]
  fn test_parse_loc() {
    assert_eq!(Ok(Loc::Acc), parse_loc("ACC"));
    assert!(parse_loc("Aflkj").is_err());
  }

  #[test]
  fn test_parse_source() {
    assert_eq!(Ok(Source::Loc(Loc::Acc)), parse_source("ACC"));
    assert_eq!(Ok(Source::Val(3)), parse_source("3"));
    assert!(parse_source("Aflkj").is_err());
  }

  #[test]
  fn test_parse_mov() {
    assert_eq!(Ok(Instr::Mov(Source::Val(3), Loc::Acc)),
               parse_mov("3, ACC".split_whitespace()));
    assert_eq!(Ok(Instr::Mov(Source::Loc(Loc::Acc), Loc::Acc)),
               parse_mov("ACC, ACC".split_whitespace()));

    assert!(parse_mov("f, ACC".split_whitespace()).is_err());
  }

  #[test]
  fn test_parse_line() {
    assert_eq!(Ok(Instr::Mov(Source::Val(3), Loc::Acc)),
               parse_line("MOV 3, ACC"));
    assert_eq!(Ok(Instr::Swp), parse_line("SWP"));
    assert_eq!(Ok(Instr::Sub(Source::Val(3))),
               parse_line("SUB 3"));
    assert_eq!(Ok(Instr::Sub(Source::Loc(Loc::Left))),
               parse_line("SUB LEFT"));
    assert_eq!(Ok(Instr::Jmp("FOO")),
               parse_line("JMP FOO"));
  }
}
