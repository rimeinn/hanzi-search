#![allow(dead_code)]

use std::{collections::HashMap, fs::File, io::{self, BufRead, BufReader}, path::Path};
use nom::{
    Finish, IResult, Parser, branch::alt, bytes::take_while1, character::satisfy, combinator::{eof, opt}, multi::many_m_n, sequence::delimited, character::complete::char,
};
use log::{warn, debug};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IDC(char);

const ENCODED_IDC: &str = "⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻⿼⿽⿾⿿㇯";

fn idc_arity(c: char) -> usize {
    match c {
        '⿾' | '⿿' => 1,
        '⿲' | '⿳' => 3,
        _ => 2,
    }
}

fn is_idc(c: char) -> bool {
    ENCODED_IDC.contains(c)
}

impl IDC {
    pub fn new(idc: char) -> Option<IDC> {
        if ENCODED_IDC.find(idc).is_some() {
            return Some(IDC(idc));
        } else {
            return None;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IDS {
    Char(char),
    Special(String),
    Composition {
        idc: IDC,
        children: Vec<IDS>,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedIDS {
    pub ids: IDS,
    pub tag: String,
}

#[derive(Default, Debug, Clone)]
pub struct IDSTable {
    table: HashMap<char, TaggedIDS>,
}

impl IDSTable {
    pub fn load_file<P: AsRef<Path>>(path: P) -> io::Result<IDSTable> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut table = HashMap::new();
        for (_i, line) in reader.lines().enumerate() {
            let line = line.expect("valid line");
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let Some(char) = parts[1].chars().next() else {
                warn!("Malformed line {}", line);
                continue;
            };
            if table.contains_key(&char) {
                warn!("Duplicated key {} ignored", char);
                continue;
            }
            let Ok(ids) = parse(parts[2]) else {
                warn!("Cannot parse IDS on line {}", line);
                continue;
            };
            // debug!("{} -> {:?}", char, ids);
            table.insert(char, ids);
        }
        Ok(IDSTable { table })
    }

    pub fn load_from_string(content: &str) -> io::Result<IDSTable> {
        let mut table = HashMap::new();
        for line in content.lines() {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 3 {
                continue;
            }
            let Some(char) = parts[1].chars().next() else {
                continue;
            };
            if table.contains_key(&char) {
                continue;
            }
            let Ok(ids) = parse(parts[2]) else {
                continue;
            };
            table.insert(char, ids);
        }
        Ok(IDSTable { table })
    }

    pub fn ids_match(&self, a: &IDS, b: &IDS, wildcard_k: char) -> bool {
        use IDS::*;
        match (a, b) {
            (Special(a), Special(b)) => a == b,
            (Char(a), Char(b)) => a == &wildcard_k || b == &wildcard_k || a == b,
            (Composition { idc: xc, children: xs, .. }, Composition { idc: yc, children: ys, .. }) if xc == yc && xs.len() == ys.len() => {
                for (x, y) in xs.iter().zip(ys.iter())  {
                    if !self.ids_match(x, y, '.') {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    pub fn ids_has_matching_subcomponent(&self, a: &IDS, b: &IDS, wildcard_k: char) -> bool {
        use IDS::*;
        if a == b {
            return true;
        }
        match (a, b) {
            (Special(a), Special(b)) => a == b,
            (Special(_), _) => false,
            (Char(a), Char(b)) => a == &wildcard_k || b == &wildcard_k || a == b,
            (Char(_), Special(_)) => false,
            (Char(a), Composition { .. }) => {
                let Some(a_components) = self.table.get(a) else {
                    return false
                };
                if let TaggedIDS { ids: IDS::Char(a_char), .. } = a_components {
                    if a_char == a {
                        return false;
                    }
                }
                self.ids_has_matching_subcomponent(&a_components.ids, b, wildcard_k)
            }
            (Composition { idc: xc, children: xs, .. }, b) => {
                // Children match
                for x in xs {
                    if self.ids_has_matching_subcomponent(x, b, wildcard_k) {
                        return true;
                    }
                }
                // Structural match
                if let IDS::Composition { idc: yc, children: ys } = b {
                    if xc == yc && xs.len() == ys.len() {
                        for (x, y) in xs.iter().zip(ys.iter()) {
                            if !self.ids_match(x, y, wildcard_k) {
                                return false;
                            }
                        }
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn ids_has_subcomponent(&self, haystack: &IDS, needle: &IDS) -> bool {
        debug!("has_subcomponent haystack={:?} needle={:?}", haystack, needle);
        if haystack == needle {
            return true;
        }
        use IDS::*;
        match (haystack, needle) {
            (Special(a), Special(b)) => a == b,
            (Special(_), _) => false,
            (Char(a), Char(b)) if a == b => true,
            (Char(a), Char(b)) if a != b && !self.table.contains_key(a) => false,
            (Char(a), _) => {
                let Some(a_components) = self.table.get(a) else {
                    return false;
                };
                // Avoid circular definition.
                match (&a_components.ids, needle) {
                    (Char(a), Char(b)) => return a == b,
                    (Char(_), _) => return false,
                    _ => {},
                }
                self.ids_has_subcomponent(&a_components.ids, needle)
            },
            (Composition { children, .. }, _) => {
                for c in children {
                    if self.ids_has_subcomponent(c, needle) {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn char_has_subcomponent(&self, k: char, needle: &IDS) -> bool {
        let ids = IDS::Char(k);
        self.ids_has_subcomponent(&ids, needle)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&char, &TaggedIDS)> {
        self.table.iter()
    }
}

fn parser_tag(input: &str) -> IResult<&str, String> {
    delimited(
        char('['),
        take_while1(|c| c != ']'),
        char(']'),
    )
        .map(String::from)
        .parse(input)
}

fn parser_special(input: &str) -> IResult<&str, IDS> {
    delimited(
        satisfy(|c| c == '{'),
        take_while1(|c| c != '}'),
        satisfy(|c| c == '}')
    )
        .map(|s: &str| IDS::Special(s.to_string()))
        .parse(input)
}

fn parser_char(input: &str) -> IResult<&str, IDS> {
    satisfy(|c| !is_idc(c) && !"{[".contains(c))
        .map(IDS::Char)
        .parse(input)
}

fn parser_composition(input: &str) -> IResult<&str, IDS> {
    let (input, idc_char) = satisfy(is_idc).parse(input)?;
    let arity = idc_arity(idc_char);
    let (input, children) = many_m_n(arity, arity, parser_ids).parse(input)?;
    Ok((input, IDS::Composition {
        idc: IDC::new(idc_char).unwrap(),
        children,
    }))
}

fn parser_ids(input: &str) -> IResult<&str, IDS> {
    alt((parser_composition, parser_special, parser_char)).parse(input)
}

fn parser_tagged_ids(input: &str) -> IResult<&str, TaggedIDS> {
    (parser_ids, opt(parser_tag), eof)
        .map(|(ids, tag, _)| TaggedIDS {
            ids,
            tag: tag.unwrap_or_default(),
        })
        .parse(input)
}

pub fn parse(input: &str) -> Result<TaggedIDS, String> {
    match parser_tagged_ids(input).finish() {
        Ok((input, tids)) if input.is_empty() => Ok(tids),
        Ok(_) => Err("Input is not parsed completely".to_string()),
        Err(e) => Err(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ids_special() {
        let input = "{柬中}";
        let (_, ids) = parser_ids(input).unwrap();
        assert_eq!(ids, IDS::Special("柬中".to_string()));
    }

    #[test]
    fn parse_ids_char() {
        let input = "啊";
        let (_, ids) = parser_ids(input).unwrap();
        assert_eq!(ids, IDS::Char('啊'));
    }

    #[test]
    fn parse_tagged_ids_char() {
        let input = "啊";
        let (_, ids) = parser_tagged_ids(input).unwrap();
        assert_eq!(ids, TaggedIDS {
            tag: "".to_string(),
            ids: IDS::Char('啊'),
        });
    }

    #[test]
    fn parse_tagged_ids_composition() {
        let input = "⿱亽{⻞下}[G]";
        let (_, ids) = parser_tagged_ids(input).unwrap();
        assert_eq!(ids, TaggedIDS {
            ids: IDS::Composition {
                idc: IDC::new('⿱').unwrap(),
                children: vec![
                    IDS::Char('亽'),
                    IDS::Special("⻞下".to_string()),
                ],
            },
            tag: "G".to_string()
        });
    }
}
