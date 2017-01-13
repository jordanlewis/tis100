use std::io;
use std::io::Read;
use std::str::Lines;
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
  pos: usize,
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
  match s.ends_with(",") {
    true => Ok(&s[..s.len() - 1]),
    false => Err("invalid mov instruction with no comma".to_string()),
  }
}

pub fn parse_mov<'a>(words: &mut SplitWhitespace<'a>) -> Result<Instr<'a>, String> {
  let src = try!(words.next()
    .ok_or("invalid mov instruction without source".to_string())
    .and_then(eat_comma)
    .and_then(parse_source));
  let dest = try!(words.next()
    .ok_or("invalid mov instruction without dest".to_string())
    .and_then(parse_loc));
  Ok(Instr::Mov(src, dest))
}

pub fn parse_line(line: &str) -> Result<Instr, String> {
  let mut words = line.split_whitespace();
  match words.next() {
    Some("NOP") => Ok(Instr::Nop),
    Some("MOV") => parse_mov(&mut words),
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
      match s.starts_with("#") {
        true => Ok(Instr::Comment(words.fold(s.to_string(), |acc, s| acc + " " + s))),
        false => Err(format!("invalid instr '{}'", s)),
      }
    }
    None => Ok(Instr::Emptyline),
  }
}

pub fn get_label(line: &str) -> (&str, Option<&str>) {
  match line.find(":") {
    Some(i) => {
      if line[..i].contains(" ") {
        (&line, None)
      } else {
        (&line[i + 1..], Some(&line[..i]))
      }
    }
    None => (&line, None),
  }
}

fn parse_program<'a>(buf: Vec<&'a str>) -> Result<Program<'a>, String> {
  let mut instrs = Vec::new();
  let mut labels = HashMap::new();
  for (line_no, line) in buf.iter().enumerate() {
    let (line, label) = get_label(line);
    match label {
      Some(label) => {
        if labels.insert(label, instrs.len()) != None {
          return Err(format!("label '{}' already exists", label));
        }
      }
      None => {}
    }
    match try!(parse_line(line)) {
      Instr::Comment(_) |
      Instr::Emptyline => continue,
      instr => {
        instrs.push(InstrAndPos {
          instr: instr,
          pos: line_no,
        })
      }
    }
  }
  Ok(Program {
    instrs: instrs,
    labels: labels,
  })
}

fn parse_spec<'a>(buf: Lines<'a>) -> Result<Spec<'a>, String> {
  let mut programs = Vec::new();

  let mut buf = buf.into_iter();
  let first_line = try!(buf.next().ok_or("invalid empty spec"));
  if first_line != "@0" {
    return Err(format!("invalid spec header {}, expecting @0", first_line));
  }

  let mut next_section = 1;
  let mut raw_program: Vec<&str> = Vec::new();
  let mut raw_programs = Vec::new();
  for line in buf {
    if line.starts_with("@") {
      let sec = try!(line[1..].parse::<u8>().map_err(|e| e.to_string()));
      if sec != next_section {
        return Err(format!("expecting section {}, found section {}", next_section, sec));
      } else if sec > 11 {
        return Err(format!("section {} greater than maximum 11", sec));
      }
      next_section = next_section + 1;
      let last_line = try!(raw_program.pop().ok_or("invalid empty section".to_string()));
      if !last_line.is_empty() {
        return Err(format!("invalid section: didn't end with an empty line"));
      }
      print!("{:?}\n", raw_program);
      raw_programs.push(raw_program);
      raw_program = Vec::new();
    } else {
      raw_program.push(line);
    }
  }

  for raw_program in raw_programs {
    programs.push(try!(parse_program(raw_program)))
  }

  Ok(Spec { programs: programs })
}

fn main() {
  let mut buffer = String::new();
  let stdin = io::stdin();
  stdin.lock().read_to_string(&mut buffer).unwrap();
  let spec = parse_spec(buffer.lines()).unwrap();
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
               parse_mov(&mut "3, ACC".split_whitespace()));
    assert_eq!(Ok(Instr::Mov(Source::Loc(Loc::Acc), Loc::Acc)),
               parse_mov(&mut "ACC, ACC".split_whitespace()));

    assert!(parse_mov(&mut "f, ACC".split_whitespace()).is_err());
  }

  #[test]
  fn test_parse_line() {
    assert_eq!(Ok(Instr::Mov(Source::Val(3), Loc::Acc)),
               parse_line("MOV 3, ACC"));
    assert_eq!(Ok(Instr::Swp), parse_line("SWP"));
    assert_eq!(Ok(Instr::Sub(Source::Val(3))), parse_line("SUB 3"));
    assert_eq!(Ok(Instr::Sub(Source::Loc(Loc::Left))),
               parse_line("SUB LEFT"));
    assert_eq!(Ok(Instr::Jmp("FOO")), parse_line("JMP FOO"));
  }

  #[test]
  fn test_get_label() {
    assert_eq!(("label nop", None), get_label("label nop"));
    assert_eq!(("nop", Some("label")), get_label("label:nop"));
    assert_eq!((" nop", Some("label")), get_label("label: nop"));
    assert_eq!(("label : nop", None), get_label("label : nop"));
    assert_eq!(("3: nop", Some("label")), get_label("label:3: nop"));
  }
}
