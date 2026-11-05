//! TagFile (RedPower's hierarchical config format), the `RenderModel` OBJ-style quad
//! model reader and `TexVertex`.
//!
//! Ports `com.eloraam.redpower.core.{TagFile, RenderModel, TexVertex}`. The Java reads
//! both formats through `java.io.StreamTokenizer`; `Tokenizer` below reproduces the
//! parts of its behaviour the two readers rely on (comment char `#`, significant EOL,
//! numbers, words, `"` quotes, ordinary characters).

use super::spatial::Vector3;
use std::collections::BTreeMap;
use std::fmt;

/// Parse failure ("Parse error", "Non-Quad Face", "Empty key name", ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl ParseError {
    fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

// ============================================================================
// StreamTokenizer subset
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Eof,
    Eol,
    Num(f64),
    Word(String),
    Quote(String),
    Char(char),
}

struct Tokenizer {
    chars: Vec<char>,
    pos: usize,
    /// `.` is a numeric character (model files) or ordinary (TagFile).
    dot_is_digit: bool,
}

impl Tokenizer {
    fn new(src: &str, dot_is_digit: bool) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            dot_is_digit,
        }
    }

    fn read(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn unread(&mut self) {
        self.pos -= 1;
    }

    fn is_alpha(c: char) -> bool {
        let n = c as u32;
        c.is_ascii_alphabetic() || n >= 128 + 32
    }

    fn is_digit(&self, c: char) -> bool {
        c.is_ascii_digit() || c == '-' || (self.dot_is_digit && c == '.')
    }

    fn next(&mut self) -> Tok {
        loop {
            let Some(c) = self.read() else {
                return Tok::Eof;
            };
            match c {
                '\r' => {
                    if self.chars.get(self.pos) == Some(&'\n') {
                        self.pos += 1;
                    }
                    return Tok::Eol;
                }
                '\n' => return Tok::Eol,
                c if (c as u32) <= 32 => continue,
                '#' => {
                    while let Some(d) = self.read() {
                        if d == '\n' || d == '\r' {
                            self.unread();
                            break;
                        }
                    }
                    continue;
                }
                '"' => return self.string(),
                c if self.is_digit(c) => return self.number(c),
                c if Self::is_alpha(c) => {
                    let mut s = String::new();
                    s.push(c);
                    while let Some(d) = self.read() {
                        if Self::is_alpha(d) || self.is_digit(d) {
                            s.push(d);
                        } else {
                            self.unread();
                            break;
                        }
                    }
                    return Tok::Word(s);
                }
                c => return Tok::Char(c),
            }
        }
    }

    fn number(&mut self, first: char) -> Tok {
        let mut c = first;
        let mut neg = false;
        if c == '-' {
            match self.read() {
                Some(d) if d == '.' || d.is_ascii_digit() => c = d,
                Some(_) => {
                    self.unread();
                    return Tok::Char('-');
                }
                None => return Tok::Char('-'),
            }
            neg = true;
        }
        let mut v = 0.0f64;
        let mut seen_dot = false;
        let mut frac = 0i32;
        loop {
            if c == '.' && !seen_dot {
                seen_dot = true;
            } else if c.is_ascii_digit() {
                v = v * 10.0 + f64::from(c as u8 - b'0');
                if seen_dot {
                    frac += 1;
                }
            } else {
                self.unread();
                break;
            }
            match self.read() {
                Some(d) => c = d,
                None => break,
            }
        }
        if frac > 0 {
            v /= 10f64.powi(frac);
        }
        Tok::Num(if neg { -v } else { v })
    }

    fn string(&mut self) -> Tok {
        let mut s = String::new();
        while let Some(c) = self.read() {
            match c {
                '"' | '\n' | '\r' => {
                    if c != '"' {
                        self.unread();
                    }
                    break;
                }
                '\\' => {
                    let Some(e) = self.read() else { break };
                    match e {
                        'a' => s.push('\u{7}'),
                        'b' => s.push('\u{8}'),
                        'f' => s.push('\u{c}'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'v' => s.push('\u{b}'),
                        '0'..='7' => {
                            let mut n = e as u32 - '0' as u32;
                            let limit = if e <= '3' { 2 } else { 1 };
                            for _ in 0..limit {
                                match self.read() {
                                    Some(d) if ('0'..='7').contains(&d) => {
                                        n = n * 8 + (d as u32 - '0' as u32)
                                    }
                                    Some(_) => {
                                        self.unread();
                                        break;
                                    }
                                    None => break,
                                }
                            }
                            s.push(char::from_u32(n).unwrap_or('\0'));
                        }
                        other => s.push(other),
                    }
                }
                c => s.push(c),
            }
        }
        Tok::Quote(s)
    }
}

// ============================================================================
// TagFile
// ============================================================================

/// A TagFile value: integer, string or nested dictionary (sorted by key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Int(i32),
    Str(String),
    Dict(BTreeMap<String, Tag>),
}

/// Hierarchical `key=value` / `key { ... }` config file with dotted-path access.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TagFile {
    pub contents: BTreeMap<String, Tag>,
    pub comments: BTreeMap<String, String>,
    pub file_comment: String,
}

impl TagFile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert `tag` at dotted path `name`, creating intermediate dictionaries.
    pub fn add_tag(&mut self, name: &str, tag: Tag) -> Result<(), ParseError> {
        let mut sub = &mut self.contents;
        let mut parts = name.split('.').peekable();
        while let Some(p) = parts.next() {
            if p.is_empty() {
                return Err(ParseError::new("Empty key name"));
            }
            if parts.peek().is_none() {
                sub.insert(p.to_string(), tag);
                return Ok(());
            }
            let entry = sub
                .entry(p.to_string())
                .or_insert_with(|| Tag::Dict(BTreeMap::new()));
            match entry {
                Tag::Dict(m) => sub = m,
                _ => return Err(ParseError::new("Key not a dictionary")),
            }
        }
        Ok(())
    }

    pub fn get_tag(&self, name: &str) -> Option<&Tag> {
        let mut sub = &self.contents;
        let mut parts = name.split('.').peekable();
        while let Some(p) = parts.next() {
            if parts.peek().is_none() {
                return sub.get(p);
            }
            match sub.get(p) {
                Some(Tag::Dict(m)) => sub = m,
                _ => return None,
            }
        }
        None
    }

    pub fn remove_tag(&mut self, name: &str) -> Option<Tag> {
        let mut sub = &mut self.contents;
        let mut parts = name.split('.').peekable();
        while let Some(p) = parts.next() {
            if parts.peek().is_none() {
                return sub.remove(p);
            }
            match sub.get_mut(p) {
                Some(Tag::Dict(m)) => sub = m,
                _ => return None,
            }
        }
        None
    }

    pub fn comment_tag(&mut self, key: &str, value: &str) {
        self.comments.insert(key.to_string(), value.to_string());
    }

    pub fn comment_file(&mut self, cmt: &str) {
        self.file_comment = cmt.to_string();
    }

    pub fn add_string(&mut self, name: &str, value: &str) -> Result<(), ParseError> {
        self.add_tag(name, Tag::Str(value.to_string()))
    }

    pub fn add_int(&mut self, name: &str, value: i32) -> Result<(), ParseError> {
        self.add_tag(name, Tag::Int(value))
    }

    pub fn get_string(&self, name: &str) -> Option<&str> {
        match self.get_tag(name) {
            Some(Tag::Str(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_string_or<'a>(&'a self, name: &str, def: &'a str) -> &'a str {
        self.get_string(name).unwrap_or(def)
    }

    /// Integer at `name`, or 0.
    pub fn get_int(&self, name: &str) -> i32 {
        self.get_int_or(name, 0)
    }

    pub fn get_int_or(&self, name: &str, def: i32) -> i32 {
        match self.get_tag(name) {
            Some(Tag::Int(i)) => *i,
            _ => def,
        }
    }

    /// Parse `src` into this file's contents (`readStream`).
    pub fn read_str(&mut self, src: &str) -> Result<(), ParseError> {
        let mut tok = Tokenizer::new(src, false);
        read_tag(&mut self.contents, &mut tok)
    }

    pub fn parse(src: &str) -> Result<Self, ParseError> {
        let mut t = Self::new();
        t.read_str(src)?;
        Ok(t)
    }

    /// Serialise to text (`saveFile`): file comment, then every tag.
    pub fn save_string(&self) -> String {
        let mut out = String::new();
        write_comment(&mut out, "", Some(&self.file_comment));
        self.save_tag(&mut out, &self.contents, None, "");
        out
    }

    fn collapsed_tag(&self, tag: &BTreeMap<String, Tag>, key: &str, ft: &str) -> Option<String> {
        let mut cn = key.to_string();
        let mut ob = tag.get(key)?;
        let mut ft = ft.to_string();
        while !self.comments.contains_key(&ft) {
            match ob {
                Tag::Str(s) => return Some(format!("{cn}=\"{}\"", s.replace('"', "\\\""))),
                Tag::Int(i) => return Some(format!("{cn}={i}")),
                Tag::Dict(m) => {
                    if m.len() != 1 {
                        return None;
                    }
                    let (k, v) = m.iter().next()?;
                    cn = format!("{cn}.{k}");
                    ft = format!("{ft}.{k}");
                    ob = v;
                }
            }
        }
        None
    }

    fn save_tag(
        &self,
        out: &mut String,
        tag: &BTreeMap<String, Tag>,
        name: Option<&str>,
        indent: &str,
    ) {
        for (k, ob) in tag {
            let ft = match name {
                Some(n) => format!("{n}.{k}"),
                None => k.clone(),
            };
            write_comment(out, indent, self.comments.get(&ft).map(String::as_str));
            match ob {
                Tag::Str(s) => {
                    out.push_str(&format!("{indent}{k}=\"{}\"\n", s.replace('"', "\\\"")))
                }
                Tag::Int(i) => out.push_str(&format!("{indent}{k}={i}\n")),
                Tag::Dict(m) => match self.collapsed_tag(tag, k, &ft) {
                    Some(ct) => out.push_str(&format!("{indent}{ct}\n")),
                    None => {
                        out.push_str(&format!("{indent}{k} {{\n"));
                        self.save_tag(out, m, Some(&ft), &format!("{indent}    "));
                        out.push_str(&format!("{indent}}}\n\n"));
                    }
                },
            }
        }
    }
}

/// `writecomment`: one `# line` per `\n`-separated line (Java `split` drops trailing empties).
fn write_comment(out: &mut String, indent: &str, cmt: Option<&str>) {
    let Some(cmt) = cmt else { return };
    let mut lines: Vec<&str> = cmt.split('\n').collect();
    if lines.len() > 1 {
        while lines.last() == Some(&"") {
            lines.pop();
        }
    }
    for l in lines {
        out.push_str(&format!("{indent}# {l}\n"));
    }
}

fn parse_err<T>() -> Result<T, ParseError> {
    Err(ParseError::new("Parse error"))
}

fn dict_at<'a>(map: &'a mut BTreeMap<String, Tag>, key: &str) -> &'a mut BTreeMap<String, Tag> {
    if !matches!(map.get(key), Some(Tag::Dict(_))) {
        map.insert(key.to_string(), Tag::Dict(BTreeMap::new()));
    }
    match map.get_mut(key) {
        Some(Tag::Dict(m)) => m,
        _ => unreachable!(),
    }
}

/// `readtag`: parse entries until `}` or end of input.
fn read_tag(tag: &mut BTreeMap<String, Tag>, tok: &mut Tokenizer) -> Result<(), ParseError> {
    'lines: loop {
        match tok.next() {
            Tok::Eof | Tok::Char('}') => return Ok(()),
            Tok::Eol => continue,
            Tok::Word(first) => {
                let mut key = first;
                let mut ltag: &mut BTreeMap<String, Tag> = tag;
                loop {
                    match tok.next() {
                        Tok::Char('.') => {
                            ltag = dict_at(ltag, &key);
                            match tok.next() {
                                Tok::Word(w) => key = w,
                                _ => return parse_err(),
                            }
                        }
                        Tok::Char('=') => {
                            match tok.next() {
                                // Java: (int) of a double saturates, as does `as`.
                                Tok::Num(n) => {
                                    ltag.insert(key, Tag::Int(n as i32));
                                }
                                Tok::Quote(s) => {
                                    ltag.insert(key, Tag::Str(s));
                                }
                                _ => return parse_err(),
                            }
                            if tok.next() == Tok::Eol {
                                continue 'lines;
                            }
                            return parse_err();
                        }
                        Tok::Char('{') => {
                            let sub = dict_at(ltag, &key);
                            read_tag(sub, tok)?;
                            if tok.next() == Tok::Eol {
                                continue 'lines;
                            }
                            return parse_err();
                        }
                        _ => return parse_err(),
                    }
                }
            }
            _ => return parse_err(),
        }
    }
}

// ============================================================================
// TexVertex / RenderModel
// ============================================================================

/// A textured, tinted model vertex reference (`vtx` indexes the model's vertex list).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TexVertex {
    pub vtx: i32,
    pub u: f64,
    pub v: f64,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub brtex: i32,
}

impl TexVertex {
    /// `TexVertex(vti, ui, vi)`: raw UVs, white tint.
    pub fn new(vtx: i32, u: f64, v: f64) -> Self {
        Self {
            vtx,
            u,
            v,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            brtex: 0,
        }
    }

    /// `TexVertex(vti, tn, ui, vi)`: UVs inside tile `tn` of a 16x16 atlas (1/16 per tile and per unit).
    pub fn with_tile(vtx: i32, tile: i32, u: f64, v: f64) -> Self {
        Self::new(
            vtx,
            f64::from(tile & 15) * 0.0625 + u * 0.0625,
            f64::from(tile >> 4) * 0.0625 + v * 0.0625,
        )
    }

    pub fn set_uv(&mut self, u: f64, v: f64) {
        self.u = u;
        self.v = v;
    }
}

/// A quad mesh: vertices, per-face texture vertices and `[start, end)` face ranges per group/subgroup.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderModel {
    pub vertex_list: Vec<Vector3>,
    pub tex_list: Vec<[TexVertex; 4]>,
    pub groups: Vec<Vec<[i32; 2]>>,
}

#[derive(Default)]
struct ModelReader {
    vertex: Vec<Vector3>,
    faceno: Vec<i32>,
    texvert: Vec<TexVertex>,
    groups: Vec<i32>,
    grcnt: Vec<i32>,
    fno: i32,
}

impl ModelReader {
    fn get_float(tok: &mut Tokenizer) -> Result<f64, ParseError> {
        match tok.next() {
            Tok::Num(n) => Ok(n),
            _ => parse_err(),
        }
    }

    fn get_int(tok: &mut Tokenizer) -> Result<i32, ParseError> {
        Self::get_float(tok).map(|n| n as i32)
    }

    fn end_line(tok: &mut Tokenizer) -> Result<(), ParseError> {
        if tok.next() == Tok::Eol {
            Ok(())
        } else {
            parse_err()
        }
    }

    fn parse_face(&mut self, tok: &mut Tokenizer) -> Result<(), ParseError> {
        loop {
            match tok.next() {
                Tok::Eof | Tok::Eol => {
                    self.faceno.push(-1);
                    self.fno += 1;
                    return Ok(());
                }
                Tok::Num(n) => {
                    if tok.next() != Tok::Char('/') {
                        return parse_err();
                    }
                    let n2 = Self::get_int(tok)?;
                    self.faceno.push(n as i32);
                    self.faceno.push(n2);
                }
                _ => return parse_err(),
            }
        }
    }

    fn set_group(&mut self, gr: i32, sub: i32) -> Result<(), ParseError> {
        self.groups.extend([gr, sub, self.fno]);
        if gr < 0 || sub < 0 || (self.grcnt.len() as i64) < i64::from(gr) {
            return parse_err();
        }
        let gr = gr as usize;
        if self.grcnt.len() == gr {
            self.grcnt.push(0);
        }
        self.grcnt[gr] = self.grcnt[gr].max(sub + 1);
        Ok(())
    }

    fn parse_group(&mut self, tok: &mut Tokenizer) -> Result<(), ParseError> {
        let n1 = Self::get_int(tok)?;
        let mut n2 = 0;
        let mut t = tok.next();
        if t == Tok::Char('_') {
            n2 = Self::get_int(tok)?;
            t = tok.next();
        }
        self.set_group(n1, n2)?;
        if t != Tok::Eol {
            return parse_err();
        }
        Ok(())
    }

    fn read_model(&mut self, src: &str) -> Result<(), ParseError> {
        let mut tok = Tokenizer::new(src, true);
        loop {
            match tok.next() {
                Tok::Eof => return Ok(()),
                Tok::Eol => {}
                Tok::Word(w) => match w.as_str() {
                    "v" => {
                        let x = Self::get_float(&mut tok)?;
                        let y = Self::get_float(&mut tok)?;
                        let z = Self::get_float(&mut tok)?;
                        self.vertex.push(Vector3::new(x, y, z));
                        Self::end_line(&mut tok)?;
                    }
                    "vt" => {
                        let u = Self::get_float(&mut tok)?;
                        let v = Self::get_float(&mut tok)?;
                        self.texvert.push(TexVertex::new(0, u, v));
                        Self::end_line(&mut tok)?;
                    }
                    "vtc" => {
                        let u = Self::get_float(&mut tok)?;
                        let v = Self::get_float(&mut tok)?;
                        let mut tv = TexVertex::new(0, u, v);
                        tv.r = Self::get_float(&mut tok)? as f32;
                        tv.g = Self::get_float(&mut tok)? as f32;
                        tv.b = Self::get_float(&mut tok)? as f32;
                        self.texvert.push(tv);
                        Self::end_line(&mut tok)?;
                    }
                    "f" => self.parse_face(&mut tok)?,
                    "g" => self.parse_group(&mut tok)?,
                    _ => loop {
                        match tok.next() {
                            Tok::Eol | Tok::Eof => break,
                            _ => {}
                        }
                    },
                },
                _ => return parse_err(),
            }
        }
    }
}

impl RenderModel {
    /// Parse model text (`loadModel`; resource lookup is the caller's). Every face must be a quad.
    pub fn load_model(src: &str) -> Result<Self, ParseError> {
        let mut ml = ModelReader::default();
        ml.read_model(src)?;
        let non_quad = || ParseError::new("Non-Quad Face");

        let mut tex_list = Vec::new();
        let mut i = 0;
        while i < ml.faceno.len() {
            let mut tv = [TexVertex::default(); 4];
            for slot in &mut tv {
                let n = ml.faceno[i];
                i += 1;
                if n < 0 {
                    return Err(non_quad());
                }
                let m = *ml.faceno.get(i).ok_or_else(non_quad)?;
                i += 1;
                let mut t = *usize::try_from(m - 1)
                    .ok()
                    .and_then(|k| ml.texvert.get(k))
                    .ok_or_else(|| ParseError::new("Bad texture vertex index"))?;
                t.vtx = n - 1;
                t.v = 1.0 - t.v;
                *slot = t;
            }
            if ml.faceno.get(i).copied().ok_or_else(non_quad)? >= 0 {
                return Err(non_quad());
            }
            i += 1;
            tex_list.push(tv);
        }

        let mut groups: Vec<Vec<[i32; 2]>> =
            ml.grcnt.iter().map(|&l| vec![[0, 0]; l as usize]).collect();
        let (mut lgs, mut lgmn, mut lgsn) = (-1i32, 0usize, 0usize);
        for t in ml.groups.as_chunks::<3>().0 {
            if lgs >= 0 {
                groups[lgmn][lgsn] = [lgs, t[2]];
            }
            lgmn = t[0] as usize;
            lgsn = t[1] as usize;
            lgs = t[2];
        }
        if lgs >= 0 {
            groups[lgmn][lgsn] = [lgs, ml.fno];
        }

        Ok(Self {
            vertex_list: ml.vertex,
            tex_list,
            groups,
        })
    }

    /// Multiply every vertex by `sf`.
    pub fn scale(&mut self, sf: f64) {
        for v in &mut self.vertex_list {
            v.multiply(sf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagfile_round_trip() {
        let src =
            "# header\nname=\"Bob\"\nsize=12\nserver {\n    port=25565\n    deep.er.key=-3\n}\n";
        let tf = TagFile::parse(src).unwrap();
        assert_eq!(tf.get_string("name"), Some("Bob"));
        assert_eq!(tf.get_int("server.port"), 25565);
        assert_eq!(tf.get_int("server.deep.er.key"), -3);
        assert_eq!(tf.get_int_or("missing", 7), 7);
        let out = tf.save_string();
        assert_eq!(TagFile::parse(&out).unwrap(), tf);
        assert!(out.contains("server {\n    deep.er.key=-3\n    port=25565\n}\n\n"));
    }

    #[test]
    fn tagfile_comments_and_errors() {
        let mut tf = TagFile::new();
        tf.comment_file("top\nline2");
        tf.add_int("a.b.c", 5).unwrap();
        tf.add_string("a.s", "q\"x").unwrap();
        tf.comment_tag("a.s", "note");
        let out = tf.save_string();
        assert_eq!(
            out,
            "# top\n# line2\na {\n    b.c=5\n    # note\n    s=\"q\\\"x\"\n}\n\n"
        );
        assert_eq!(
            tf.add_int("a.b.c.d", 1),
            Err(ParseError::new("Key not a dictionary"))
        );
        assert_eq!(
            tf.add_int("a..b", 1),
            Err(ParseError::new("Empty key name"))
        );
        assert_eq!(tf.remove_tag("a.b.c"), Some(Tag::Int(5)));
        assert!(TagFile::parse("x=1 2\n").is_err());
        assert!(TagFile::parse("x=1").is_err());
    }

    #[test]
    fn tex_vertex_primitives() {
        let t = TexVertex::with_tile(2, 0x23, 8.0, 16.0);
        assert_eq!(t.u, 3.0 * 0.0625 + 0.5);
        assert_eq!(t.v, 2.0 * 0.0625 + 1.0);
        let mut c = t;
        c.set_uv(0.25, 0.5);
        assert_eq!((c.u, c.v, c.r), (0.25, 0.5, 1.0));
    }

    const QUAD: &str = "# model\nv 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 0.25\nvtc 0 1 0.5 0.5 0.5\n\
g 0\nf 1/1 2/2 3/3 4/4\ng 1_2\nf 4/4 3/3 2/2 1/1\nusemtl foo\n";

    #[test]
    fn model_loads_quads_and_groups() {
        let mut m = RenderModel::load_model(QUAD).unwrap();
        assert_eq!(m.vertex_list.len(), 4);
        assert_eq!(m.tex_list.len(), 2);
        assert_eq!(m.tex_list[0][2].vtx, 2);
        assert_eq!(m.tex_list[0][2].v, 0.75);
        assert_eq!(m.tex_list[0][3].g, 0.5);
        assert_eq!(m.groups, vec![vec![[0, 1]], vec![[0, 0], [0, 0], [1, 2]]]);
        m.scale(0.5);
        assert_eq!(m.vertex_list[2], Vector3::new(0.5, 0.5, 0.0));
    }

    #[test]
    fn model_rejects_non_quads() {
        let tri = "v 0 0 0\nvt 0 0\nf 1/1 1/1 1/1\n";
        assert_eq!(
            RenderModel::load_model(tri),
            Err(ParseError::new("Non-Quad Face"))
        );
        assert!(RenderModel::load_model("v 0 0\n").is_err());
    }
}
