use std::io;
use std::io::BufRead;
use std::io::StdinLock;

enum Loc {
  Left,
  Right,
  Up,
  Down,
  Acc,
  Last,
}

enum Source {
  Val(u16),
  Loc(Loc),
}

type Label = String;

#[derive(Debug)]
enum Instr {
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
  Jro(Label),
  Label(String)
  Comment(String),
}

struct Program =

struct Spec {
  nodes: [;]
}

fn parse_spec(buf: &mut BufRead) -> Result<i32, String> {
  let mut section = -1;
  for line in buf.lines() {
    let line = line.unwrap();
    let mut words = line.split_whitespace();
    let instr = match words.next() {
      Some("Loc")
      None => { }
    }
    word = word.next()
    if line.len() == 0 {
    }
    if line.nth(0)
    let mut chars = line.chars();
    if chars.next() != Some('@') {
        return Err(format!("invalid header: expecting '@', got ".to_string())
    }
    if line.len() != 2 || line[0] != "@" {
    }
    if let Ok(n) = line[1].parse::<i8>() {
        section = n
    } else {
        return Err(format!("invalid header: {} is not a digit", line[1]))
    }

    println!("{}", line.unwrap());
  }
  Ok(1)
}

fn main() {
  let stdin = io::stdin();
  let stdin = &mut stdin.lock();

  let spec = parse_spec(stdin).unwrap();
}
