#![allow(dead_code)]

use std::{collections::HashMap, fs::File, io::{self, BufRead, BufReader}, path::Path};
use nom::{
    Finish, IResult, Parser, branch::alt, bytes::take_while1, character::satisfy, combinator::{eof, opt}, multi::many_m_n, sequence::delimited, character::complete::char,
};
use log::{warn, debug};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IDC(char);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Vert,
    Hort,
    Other,
}

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

    pub fn arity(self) -> usize {
        idc_arity(self.0)
    }

    pub fn reduce(self) -> Option<IDC> {
        match self {
            IDC('⿲') => Some(IDC('⿰')),
            IDC('⿳') => Some(IDC('⿱')),
            _ => None,
        }
    }

    pub fn direction(self) -> Direction {
        match self.0 {
            '⿰' | '⿲' => Direction::Hort,
            '⿱' | '⿳' => Direction::Vert,
            _ => Direction::Other,
        }
    }

    pub fn is_same_direction(self, other: IDC) -> bool {
        return self.direction() == other.direction()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IDS {
    Char(char),
    Special(String),
    Composition {
        idc: IDC,
        children: Vec<IDS>,
    }
}

impl std::fmt::Display for IDS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IDS::Char(k) => write!(f, "{}", k),
            IDS::Special(s) => write!(f, "{{{}}}", s),
            IDS::Composition { idc, children } => {
                write!(f, "{}", idc.0)?;
                for c in children {
                    write!(f, "{}", c)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedIDS {
    pub ids: IDS,
    pub tag: Tag,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Tag {
    Variant(String),
    Anon(usize),
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tag::Variant(s) => write!(f, "{}", s),
            Tag::Anon(_) => Ok(()),
        }
    }
}

impl From<String> for Tag {
    fn from(val: String) -> Tag {
        Tag::Variant(val)
    }
}

#[derive(Default, Debug, Clone)]
pub struct IDSTable {
    table: HashMap<(char, Tag), IDS>,
    tags: HashMap<char, Vec<Tag>>,
}

impl IDSTable {
    pub fn load_file<P: AsRef<Path>>(path: P) -> io::Result<IDSTable> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut table: HashMap<(char, Tag), IDS> = HashMap::new();
        let mut tags: HashMap<char, Vec<Tag>> = HashMap::new();
        for (_i, line) in reader.lines().enumerate() {
            let line = line.expect("valid line");
            let parts = line.split_whitespace().collect::<Vec<_>>();
            let Some(char) = parts[1].chars().next() else {
                warn!("Malformed line {}", line);
                continue;
            };
            for ids_str in parts.iter().skip(2) {
                let Ok(tids) = parse_tagged(ids_str) else {
                    warn!("Cannot parse IDS on line {}", line);
                    continue;
                };
                let key = (char, tids.tag.clone());
                if table.contains_key(&key) {
                    let tag = Tag::Anon(tags.get(&char).unwrap().len());
                    let key = (char, tag.clone());
                    table.insert(key, tids.ids);
                    tags.entry(char)
                        .and_modify(|v| v.push(tag.clone()))
                        .or_insert_with(|| vec![tag.clone()]);
                } else {
                    tags.entry(char).and_modify(|v| v.push(tids.tag.clone())).or_insert(vec![tids.tag.clone()]);
                    table.insert(key, tids.ids);
                }
            }
        }
        Ok(IDSTable {
            table,
            tags,
        })
    }

    pub fn load_from_string(content: &str) -> io::Result<IDSTable> {
        let mut table = HashMap::new();
        let mut tags: HashMap<char, Vec<Tag>> = HashMap::new();
        for line in content.lines() {
            let parts = line.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 3 {
                continue;
            }
            let Some(char) = parts[1].chars().next() else {
                continue;
            };
            for ids_str in parts.iter().skip(2) {
                let Ok(tids) = parse_tagged(ids_str) else {
                    warn!("Cannot parse IDS on line {}", line);
                    continue;
                };
                let key = (char, tids.tag.clone());
                if table.contains_key(&key) {
                    let tag = Tag::Anon(tags.get(&char).unwrap().len());
                    let key = (char, tag.clone());
                    table.insert(key, tids.ids);
                    tags.entry(char)
                        .and_modify(|v| v.push(tag.clone()))
                        .or_insert_with(|| vec![tag.clone()]);
                } else {
                    tags.entry(char).and_modify(|v| v.push(tids.tag.clone())).or_insert(vec![tids.tag.clone()]);
                    table.insert(key, tids.ids);
                }
            }
        }
        Ok(IDSTable { table, tags })
    }

    pub fn ids_match(&self, a: &IDS, b: &IDS, wildcard_k: char) -> bool {
        use IDS::*;
        match (a, b) {
            (Char(a), _) if a == &wildcard_k => true,
            (_, Char(b)) if b == &wildcard_k => true,
            (Special(a), Special(b)) => a == b,
            (Char(a), Char(b)) => a == b,
            (Char(k), Composition { .. }) => {
                let Some(k_tags) = self.tags.get(k) else {
                    return false;
                };
                for k_tag in k_tags {
                    if let Some(k_components) = self.table.get(&(*k, k_tag.clone())) {
                        if k_components != &IDS::Char(*k) {
                            return self.ids_match(k_components, b, wildcard_k);
                        }
                    }
                }
                false
            }
            (Composition { .. }, Char(_)) => {
                return self.ids_match(b, a, wildcard_k);
            }
            (x @ Composition { idc: xc, children: xs, .. }, y @ Composition { idc: yc, children: ys, .. }) => {
                if xc == yc {
                    for (x, y) in xs.iter().zip(ys.iter())  {
                        if !self.ids_match(x, y, wildcard_k) {
                            return false;
                        }
                    }
                    return true;
                } else if xc.arity() == 3 && yc.arity() == 2 && xc.is_same_direction(*yc) {
                    // try to match ⿳abc with ⿱de
                    let a = xs[0].clone();
                    let b = xs[1].clone();
                    let c = xs[2].clone();
                    let d = ys[0].clone();
                    let e = ys[1].clone();
                    let ab = Composition { idc: xc.reduce().unwrap(), children: vec![a.clone(), b.clone()] };
                    let bc = Composition { idc: xc.reduce().unwrap(), children: vec![b.clone(), c.clone()] };
                    return (self.ids_match(&ab, &d, wildcard_k) && self.ids_match(&c, &e, wildcard_k)) ||
                        (self.ids_match(&a, &d, wildcard_k) && self.ids_match(&bc, &e, wildcard_k));
                } else if xc.arity() == 2 && yc.arity() == 3 {
                    return self.ids_match(y, x, wildcard_k);
                }
                false
            }
            _ => false,
        }
    }

    pub fn ids_has_matching_subcomponent(&self, a: &IDS, b: &IDS, wildcard_k: char) -> bool {
        use IDS::*;
        if self.ids_match(a, b, wildcard_k) {
            return true;
        }
        match (a, b) {
            (Char(a), _) if a == &wildcard_k => true,
            (_, Char(b)) if b == &wildcard_k => true,
            (Special(a), Special(b)) => a == b,
            (Special(_), _) => false,
            (Char(a), Char(b)) => a == b,
            (Char(_), Special(_)) => false,
            (Char(a), Composition { .. }) => {
                let Some(a_tags) = self.tags.get(a) else {
                    return false;
                };
                for a_tag in a_tags {
                    if let Some(a_components) = self.table.get(&(*a, a_tag.clone())) {
                        if a_components != &IDS::Char(*a) {
                            return self.ids_has_matching_subcomponent(a_components, b, wildcard_k);
                        }
                    }
                }
                false
            }
            (Composition { children: xs, .. }, b) => {
                for x in xs {
                    if self.ids_has_matching_subcomponent(x, b, wildcard_k) {
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
            (Char(a), Char(b)) if a != b && !self.tags.contains_key(a) => false,
            (Char(a), _) => {
                let Some(tags) = self.tags.get(a) else {
                    return false;
                };
                for tag in tags {
                    if let Some(a_components) = self.table.get(&(*a, tag.clone())) {
                        if a_components != &IDS::Char(*a) {
                            if self.ids_has_subcomponent(a_components, needle) {
                                return true;
                            }
                        }
                    }
                }
                false
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

    pub fn iter(&self) -> impl Iterator<Item = (&(char, Tag), &IDS)> {
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
            tag: Tag::Variant(tag.unwrap_or_default()),
        })
        .parse(input)
}

pub fn parse(input: &str) -> Result<IDS, String> {
    match parser_ids(input).finish() {
        Ok((input, ids)) if input.is_empty() => Ok(ids),
        Ok(_) => Err("Input is not parsed completely".to_string()),
        Err(e) => Err(e.to_string())
    }
}

pub fn parse_tagged(input: &str) -> Result<TaggedIDS, String> {
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
            tag: Tag::Variant("".to_string()),
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
            tag: Tag::Variant("G".to_string())
        });
    }
}
