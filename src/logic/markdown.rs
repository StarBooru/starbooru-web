//! A small Markdown reader for release notes: headings, paragraphs, lists,
//! quotes, code and links. It makes a tree that `components::markdown` draws
//! as elements, so nothing in the text reaches the page as HTML. HTML in the
//! text is dropped, pictures become their description (the app loads nothing
//! from anywhere but its server), and only web and mail links stay links.

/// A block of text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// A heading, 1 to 6.
    Heading(u8, Vec<Inline>),
    Paragraph(Vec<Inline>),
    List {
        /// Numbered, from `start`.
        ordered: bool,
        start: u64,
        items: Vec<Vec<Block>>,
    },
    Quote(Vec<Block>),
    Code(String),
    Rule,
}

/// A run of text inside a block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Strong(Vec<Inline>),
    Emphasis(Vec<Inline>),
    Code(String),
    Link {
        href: String,
        text: Vec<Inline>,
    },
    /// A new line, as GitHub keeps them in release notes.
    Break,
}

/// The blocks in `text`.
pub fn parse(text: &str) -> Vec<Block> {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<String> = text.lines().map(|l| l.replace('\t', "    ")).collect();
    let lines: Vec<&str> = lines.iter().map(String::as_str).collect();
    blocks(&lines)
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// A code fence: its character and length, for the line that closes it.
fn fence(line: &str) -> Option<(char, usize)> {
    if indent(line) > 3 {
        return None;
    }
    let t = line.trim_start();
    let c = t.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let n = t.chars().take_while(|x| *x == c).count();
    // A backtick fence's info string has no backticks.
    let info = t.get(n..).unwrap_or_default();
    (n >= 3 && !(c == '`' && info.contains('`'))).then_some((c, n))
}

fn heading(line: &str) -> Option<(u8, &str)> {
    if indent(line) > 3 {
        return None;
    }
    let t = line.trim_start();
    let level = t.chars().take_while(|c| *c == '#').count();
    let rest = t.get(level..)?;
    if !(1..=6).contains(&level) || !(rest.is_empty() || rest.starts_with(' ')) {
        return None;
    }
    let rest = rest.trim();
    // A closing run of #s goes, when a space stands before it.
    let body = rest.trim_end_matches('#');
    let body = if body.is_empty() || body.ends_with(' ') {
        body.trim_end()
    } else {
        rest
    };
    Some((u8::try_from(level).unwrap_or(6), body))
}

fn rule(line: &str) -> bool {
    if indent(line) > 3 {
        return false;
    }
    let marks: Vec<char> = line.chars().filter(|c| *c != ' ').collect();
    marks.len() >= 3
        && marks
            .first()
            .is_some_and(|m| matches!(m, '-' | '*' | '_') && marks.iter().all(|c| c == m))
}

/// The heading level a line of `=` or `-` under a paragraph makes of it.
fn underline(line: &str) -> Option<u8> {
    let t = line.trim();
    if indent(line) > 3 || t.is_empty() {
        return None;
    }
    if t.chars().all(|c| c == '=') {
        Some(1)
    } else if t.chars().all(|c| c == '-') {
        Some(2)
    } else {
        None
    }
}

/// A list item's start: numbered or not, its number, and the column its
/// text starts at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Bullet {
    ordered: bool,
    number: u64,
    /// `-`, `*`, `+`, `.` or `)`: a different one starts another list.
    mark: char,
    content: usize,
}

fn bullet(line: &str) -> Option<Bullet> {
    let lead = indent(line);
    if lead > 3 {
        return None;
    }
    let t = line.get(lead..)?;
    let (ordered, number, mark, width) = match t.chars().next()? {
        c @ ('-' | '*' | '+') => (false, 0, c, 1),
        c if c.is_ascii_digit() => {
            let digits = t.chars().take_while(char::is_ascii_digit).count();
            let mark = t.chars().nth(digits).filter(|m| *m == '.' || *m == ')')?;
            if digits > 9 {
                return None;
            }
            let number = t.get(..digits)?.parse().ok()?;
            (true, number, mark, digits + 1)
        }
        _ => return None,
    };
    let rest = t.get(width..)?;
    if !rest.is_empty() && !rest.starts_with(' ') {
        return None;
    }
    let gap = indent(rest);
    // An item's text starts one space after its mark when it starts with
    // indented code, or there is none on this line.
    let gap = if rest.trim().is_empty() || gap > 4 {
        1
    } else {
        gap
    };
    Some(Bullet {
        ordered,
        number,
        mark,
        content: lead + width + gap,
    })
}

/// A line of HTML on its own, such as `<details>` or `<img …>`.
fn html_line(line: &str) -> bool {
    let t = line.trim_start();
    let mut chars = t.chars();
    chars.next() == Some('<')
        && chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '/' || c == '!')
}

/// `line` without its first `n` columns of spaces, or as many as it has.
fn dedent(line: &str, n: usize) -> &str {
    line.get(indent(line).min(n)..).unwrap_or_default()
}

fn blocks(lines: &[&str]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para: Vec<&str> = Vec::new();
    let mut i = 0;
    let flush = |para: &mut Vec<&str>, out: &mut Vec<Block>| {
        if !para.is_empty() {
            let text = para.iter().map(|l| l.trim()).collect::<Vec<_>>().join("\n");
            let inlines = inline(&text);
            if !inlines.is_empty() {
                out.push(Block::Paragraph(inlines));
            }
            para.clear();
        }
    };
    while let Some(&line) = lines.get(i) {
        if blank(line) {
            flush(&mut para, &mut out);
            i += 1;
            continue;
        }
        if let Some((c, n)) = fence(line) {
            flush(&mut para, &mut out);
            let lead = indent(line);
            let mut code = Vec::new();
            i += 1;
            while let Some(&l) = lines.get(i) {
                i += 1;
                if fence(l).is_some_and(|(c2, n2)| c2 == c && n2 >= n)
                    && l.trim().chars().all(|x| x == c)
                {
                    break;
                }
                code.push(dedent(l, lead));
            }
            out.push(Block::Code(code.join("\n")));
            continue;
        }
        if !para.is_empty()
            && let Some(level) = underline(line)
        {
            let text = para.iter().map(|l| l.trim()).collect::<Vec<_>>().join("\n");
            para.clear();
            out.push(Block::Heading(level, inline(&text)));
            i += 1;
            continue;
        }
        if let Some((level, text)) = heading(line) {
            flush(&mut para, &mut out);
            out.push(Block::Heading(level, inline(text)));
            i += 1;
            continue;
        }
        if rule(line) {
            flush(&mut para, &mut out);
            out.push(Block::Rule);
            i += 1;
            continue;
        }
        if indent(line) <= 3 && line.trim_start().starts_with('>') {
            flush(&mut para, &mut out);
            let mut inner = Vec::new();
            while let Some(&l) = lines.get(i) {
                let t = l.trim_start();
                if indent(l) > 3 || !t.starts_with('>') {
                    break;
                }
                let t = t.get(1..).unwrap_or_default();
                inner.push(t.strip_prefix(' ').unwrap_or(t));
                i += 1;
            }
            out.push(Block::Quote(blocks(&inner)));
            continue;
        }
        if let Some(first) = bullet(line)
            // A paragraph goes on over a number other than 1, or a bare mark.
            && (para.is_empty()
                || ((!first.ordered || first.number == 1)
                    && !blank(line.get(first.content..).unwrap_or_default())))
        {
            flush(&mut para, &mut out);
            let (list, next) = list(lines, i, first);
            out.push(list);
            i = next;
            continue;
        }
        if html_line(line) && para.is_empty() {
            let text = inline(line);
            if !text.is_empty() {
                out.push(Block::Paragraph(text));
            }
            i += 1;
            continue;
        }
        para.push(line);
        i += 1;
    }
    flush(&mut para, &mut out);
    out
}

/// The list that starts at line `at` with `first`, and the line after it.
fn list(lines: &[&str], at: usize, first: Bullet) -> (Block, usize) {
    let mut items = Vec::new();
    let mut i = at;
    let mut current = first;
    // Each item's lines, without its mark and indent.
    while let Some(&line) = lines.get(i) {
        let mut body: Vec<&str> = vec![line.get(current.content..).unwrap_or_default()];
        i += 1;
        let mut gap = false;
        while let Some(&l) = lines.get(i) {
            if blank(l) {
                gap = true;
                i += 1;
                continue;
            }
            if indent(l) >= current.content {
                if gap {
                    body.push("");
                }
                body.push(dedent(l, current.content));
                gap = false;
                i += 1;
                continue;
            }
            // A line that isn't indented goes on the item's paragraph, unless
            // it starts something of its own.
            if !gap
                && bullet(l).is_none()
                && heading(l).is_none()
                && fence(l).is_none()
                && !rule(l)
                && !l.trim_start().starts_with('>')
                && body.last().is_some_and(|b| !blank(b))
            {
                body.push(l.trim_start());
                i += 1;
                continue;
            }
            break;
        }
        items.push(blocks(&body));
        // The next item, of the same kind of list.
        match lines.get(i).and_then(|l| bullet(l)) {
            Some(next) if next.ordered == first.ordered && next.mark == first.mark => {
                current = next;
            }
            _ => {
                // Blank lines that ended the list don't belong to it.
                while i > at && lines.get(i - 1).is_some_and(|l| blank(l)) {
                    i -= 1;
                }
                break;
            }
        }
    }
    (
        Block::List {
            ordered: first.ordered,
            start: first.number,
            items,
        },
        i,
    )
}

// ---------- inline ----------

/// Links that may open: web pages and mail. Anything else, such as
/// `javascript:`, stays text.
pub fn safe_href(href: &str) -> Option<String> {
    let href = href.trim();
    let lower = href.to_ascii_lowercase();
    let safe = ["https://", "http://", "mailto:"]
        .iter()
        .any(|p| lower.starts_with(p) && lower.len() > p.len());
    (safe && !href.chars().any(char::is_whitespace)).then(|| href.to_owned())
}

/// The runs of text in `text`.
pub fn inline(text: &str) -> Vec<Inline> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    Reader {
        chars: &chars,
        out: &mut out,
        text: String::new(),
    }
    .run(0, chars.len(), true);
    tidy(out)
}

/// Joins neighbouring texts, and drops empty ones and the space and breaks
/// at the ends.
fn tidy(runs: Vec<Inline>) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();
    for run in runs {
        match run {
            Inline::Text(t) if t.is_empty() => {}
            Inline::Text(t) => {
                if let Some(Inline::Text(last)) = out.last_mut() {
                    last.push_str(&t);
                } else if out.is_empty() || out.last() == Some(&Inline::Break) {
                    let t = t.trim_start();
                    if !t.is_empty() {
                        out.push(Inline::Text(t.to_owned()));
                    }
                } else {
                    out.push(Inline::Text(t));
                }
            }
            Inline::Break if out.is_empty() || out.last() == Some(&Inline::Break) => {}
            other => out.push(other),
        }
    }
    while out.last() == Some(&Inline::Break) {
        out.pop();
    }
    if let Some(Inline::Text(last)) = out.last_mut() {
        let kept = last.trim_end().len();
        last.truncate(kept);
        if last.is_empty() {
            out.pop();
        }
    }
    out
}

struct Reader<'a> {
    chars: &'a [char],
    out: &'a mut Vec<Inline>,
    text: String,
}

fn find(chars: &[char], from: usize, end: usize, pattern: &[char]) -> Option<usize> {
    (from..end).find(|&i| chars.get(i..i + pattern.len()) == Some(pattern))
}

fn word(c: Option<&char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric())
}

fn space(c: Option<&char>) -> bool {
    c.is_none_or(|c| c.is_whitespace())
}

impl Reader<'_> {
    fn push(&mut self, run: Inline) {
        if !self.text.is_empty() {
            self.out.push(Inline::Text(std::mem::take(&mut self.text)));
        }
        self.out.push(run);
    }

    fn nested(&self, from: usize, to: usize, links: bool) -> Vec<Inline> {
        let mut out = Vec::new();
        Reader {
            chars: self.chars,
            out: &mut out,
            text: String::new(),
        }
        .run(from, to, links);
        tidy(out)
    }

    fn at(&self, i: usize) -> Option<&char> {
        self.chars.get(i)
    }

    /// Reads `from..to` into `out`. `links` is false inside a link's text.
    fn run(mut self, from: usize, to: usize, links: bool) {
        let mut i = from;
        while i < to {
            let Some(&c) = self.at(i) else {
                break;
            };
            let next = match c {
                '\\' => self.escape(i, to),
                '`' => self.code(i, to),
                '\n' => {
                    // A backslash or two spaces before it say the same.
                    let kept = self.text.trim_end_matches(' ').len();
                    self.text.truncate(kept);
                    self.push(Inline::Break);
                    Some(i + 1)
                }
                '<' => self.angle(i, to, links),
                '!' if self.at(i + 1) == Some(&'[') => self.picture(i, to),
                '[' if links => self.link(i, to),
                '*' | '_' => self.emphasis(i, to, links),
                'h' | 'H' if links => self.bare_link(i, to),
                _ => None,
            };
            match next {
                Some(n) => i = n,
                None => {
                    self.text.push(c);
                    i += 1;
                }
            }
        }
        if !self.text.is_empty() {
            self.out.push(Inline::Text(self.text));
        }
    }

    fn escape(&mut self, i: usize, to: usize) -> Option<usize> {
        let &c = self.at(i + 1).filter(|_| i + 1 < to)?;
        if c == '\n' {
            self.push(Inline::Break);
            return Some(i + 2);
        }
        c.is_ascii_punctuation().then(|| {
            self.text.push(c);
            i + 2
        })
    }

    fn code(&mut self, i: usize, to: usize) -> Option<usize> {
        let n = (i..to).take_while(|&j| self.at(j) == Some(&'`')).count();
        let mut j = i + n;
        while j < to {
            let run = (j..to).take_while(|&k| self.at(k) == Some(&'`')).count();
            if run == n {
                let raw: String = self.chars.get(i + n..j)?.iter().collect();
                let raw = raw.replace('\n', " ");
                let trimmed = match raw.strip_prefix(' ').and_then(|r| r.strip_suffix(' ')) {
                    Some(inner) if !inner.trim().is_empty() => inner.to_owned(),
                    _ => raw,
                };
                self.push(Inline::Code(trimmed));
                return Some(j + n);
            }
            j += run.max(1);
        }
        // No closing run: the backticks are text.
        self.text.extend(std::iter::repeat_n('`', n));
        Some(i + n)
    }

    /// `<https://…>` is a link; any other tag goes, with what a script or a
    /// style holds.
    fn angle(&mut self, i: usize, to: usize, links: bool) -> Option<usize> {
        let close = find(self.chars, i + 1, to, &['>'])?;
        let inner: String = self.chars.get(i + 1..close)?.iter().collect();
        if let Some(href) = safe_href(&inner).filter(|_| links) {
            self.push(Inline::Link {
                href: href.clone(),
                text: vec![Inline::Text(href)],
            });
            return Some(close + 1);
        }
        let first = inner.chars().next()?;
        if !(first.is_ascii_alphabetic() || first == '/' || first == '!') {
            return None;
        }
        if inner.starts_with("!--") {
            let end = find(self.chars, i + 4, to, &['-', '-', '>']).map_or(to, |e| e + 3);
            return Some(end);
        }
        let name: String = inner
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase();
        if name == "script" || name == "style" {
            let closing: Vec<char> = format!("</{name}").chars().collect();
            let lower: Vec<char> = self.chars.iter().map(char::to_ascii_lowercase).collect();
            let end = find(&lower, close + 1, to, &closing)
                .and_then(|e| find(self.chars, e, to, &['>']))
                .map_or(to, |e| e + 1);
            return Some(end);
        }
        if name == "br" {
            self.push(Inline::Break);
        }
        Some(close + 1)
    }

    /// `[text](href)`, the brackets and where they lead: `None` when it isn't
    /// one.
    fn target(&self, i: usize, to: usize) -> Option<(usize, usize, String, usize)> {
        // The text's closing bracket, nested ones counted.
        let mut depth = 0;
        let mut j = i;
        let close = loop {
            match self.at(j).filter(|_| j < to)? {
                '\\' => j += 1,
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        break j;
                    }
                }
                _ => {}
            }
            j += 1;
        };
        if self.at(close + 1) != Some(&'(') {
            return None;
        }
        let mut depth = 0;
        let mut k = close + 2;
        let end = loop {
            match self.at(k).filter(|_| k < to)? {
                '(' => depth += 1,
                ')' if depth == 0 => break k,
                ')' => depth -= 1,
                _ => {}
            }
            k += 1;
        };
        let inside: String = self.chars.get(close + 2..end)?.iter().collect();
        let inside = inside.trim();
        // A title after the address is left out.
        let href = inside
            .strip_prefix('<')
            .and_then(|h| h.split_once('>').map(|(h, _)| h))
            .unwrap_or_else(|| inside.split_whitespace().next().unwrap_or_default());
        Some((i + 1, close, href.to_owned(), end + 1))
    }

    fn link(&mut self, i: usize, to: usize) -> Option<usize> {
        let (start, close, href, next) = self.target(i, to)?;
        let text = self.nested(start, close, false);
        match safe_href(&href) {
            Some(href) => self.push(Inline::Link { href, text }),
            // Not a link one may open: its words alone.
            None => {
                for run in text {
                    self.push(run);
                }
            }
        }
        Some(next)
    }

    /// A picture: its description, as the app loads nothing from elsewhere.
    fn picture(&mut self, i: usize, to: usize) -> Option<usize> {
        let (start, close, _, next) = self.target(i + 1, to)?;
        for run in self.nested(start, close, false) {
            self.push(run);
        }
        Some(next)
    }

    /// `https://…` in the middle of text, without the full stop or the
    /// bracket after it.
    fn bare_link(&mut self, i: usize, to: usize) -> Option<usize> {
        if word(i.checked_sub(1).and_then(|p| self.at(p))) {
            return None;
        }
        let end = (i..to)
            .find(|&j| self.at(j).is_none_or(|c| c.is_whitespace() || *c == '<'))
            .unwrap_or(to);
        let mut url: String = self.chars.get(i..end)?.iter().collect();
        loop {
            let opened = url.matches('(').count();
            let closed = url.matches(')').count();
            match url.chars().last() {
                Some('.' | ',' | ':' | ';' | '!' | '?' | '"' | '\'' | '*' | '_') => {}
                Some(')') if closed > opened => {}
                _ => break,
            }
            url.pop();
        }
        let href = safe_href(&url).filter(|h| !h.to_ascii_lowercase().starts_with("mailto:"))?;
        let used = href.chars().count();
        self.push(Inline::Link {
            href: href.clone(),
            text: vec![Inline::Text(href)],
        });
        Some(i + used)
    }

    /// `**strong**`, `__strong__`, `*emphasis*` and `_emphasis_`. An
    /// underscore inside a word, as in `game_of_life`, is a letter.
    fn emphasis(&mut self, i: usize, to: usize, links: bool) -> Option<usize> {
        let &c = self.at(i)?;
        let double = self.at(i + 1) == Some(&c);
        let n = if double { 2 } else { 1 };
        let before = i.checked_sub(1).and_then(|p| self.at(p));
        let after = self.at(i + n);
        if space(after) || (c == '_' && word(before)) {
            return None;
        }
        let marks: Vec<char> = std::iter::repeat_n(c, n).collect();
        let mut j = i + n;
        while let Some(at) = find(self.chars, j, to, &marks) {
            let inner_last = at.checked_sub(1).and_then(|p| self.at(p));
            let following = self.at(at + n);
            let closes = at > i + n
                && !space(inner_last)
                && !(c == '_' && word(following))
                // `**` doesn't close a single `*`.
                && !(!double && following == Some(&c));
            if closes {
                let inner = self.nested(i + n, at, links);
                self.push(if double {
                    Inline::Strong(inner)
                } else {
                    Inline::Emphasis(inner)
                });
                return Some(at + n);
            }
            j = at + n;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(t: &str) -> Inline {
        Inline::Text(t.into())
    }

    /// The plain text of some runs.
    fn plain(runs: &[Inline]) -> String {
        runs.iter()
            .map(|r| match r {
                Inline::Text(t) | Inline::Code(t) => t.clone(),
                Inline::Strong(inner)
                | Inline::Emphasis(inner)
                | Inline::Link { text: inner, .. } => plain(inner),
                Inline::Break => "\n".to_owned(),
            })
            .collect()
    }

    fn link(href: &str, label: &str) -> Inline {
        Inline::Link {
            href: href.into(),
            text: vec![text(label)],
        }
    }

    fn para(runs: Vec<Inline>) -> Block {
        Block::Paragraph(runs)
    }

    #[test]
    fn headings_paragraphs_and_rules() {
        assert_eq!(
            parse("# Title #\n\n## What's new\nFirst line\nsecond line\n\n---\n\nEnd"),
            vec![
                Block::Heading(1, vec![text("Title")]),
                Block::Heading(2, vec![text("What's new")]),
                para(vec![text("First line"), Inline::Break, text("second line")]),
                Block::Rule,
                para(vec![text("End")]),
            ]
        );
        // Underlined headings, and a # without a space is text.
        assert_eq!(
            parse("Big\n===\nSmall\n---\n#5 is fixed"),
            vec![
                Block::Heading(1, vec![text("Big")]),
                Block::Heading(2, vec![text("Small")]),
                para(vec![text("#5 is fixed")]),
            ]
        );
        assert_eq!(parse("  \n\n"), vec![]);
    }

    #[test]
    fn lists_nest_and_count_from_their_first_number() {
        let notes = "- one\n- two\n  - two and a half\n    more of it\n- three\n\n3. c\n4. d";
        assert_eq!(
            parse(notes),
            vec![
                Block::List {
                    ordered: false,
                    start: 0,
                    items: vec![
                        vec![para(vec![text("one")])],
                        vec![
                            para(vec![text("two")]),
                            Block::List {
                                ordered: false,
                                start: 0,
                                items: vec![vec![para(vec![
                                    text("two and a half"),
                                    Inline::Break,
                                    text("more of it"),
                                ])]],
                            },
                        ],
                        vec![para(vec![text("three")])],
                    ],
                },
                Block::List {
                    ordered: true,
                    start: 3,
                    items: vec![vec![para(vec![text("c")])], vec![para(vec![text("d")])]],
                },
            ]
        );
        // `*` and `+` lists, and a line that carries on an item.
        assert_eq!(
            parse("* a\ncarried on\n+ b"),
            vec![
                Block::List {
                    ordered: false,
                    start: 0,
                    items: vec![vec![para(vec![
                        text("a"),
                        Inline::Break,
                        text("carried on")
                    ])]],
                },
                Block::List {
                    ordered: false,
                    start: 0,
                    items: vec![vec![para(vec![text("b")])]],
                },
            ]
        );
        // A paragraph that mentions a year isn't a list.
        assert_eq!(
            parse("Released in\n2026. Enjoy"),
            vec![para(vec![
                text("Released in"),
                Inline::Break,
                text("2026. Enjoy")
            ])]
        );
    }

    #[test]
    fn code_and_quotes() {
        assert_eq!(
            parse(
                "Run:\n\n```sh\ndocker compose pull\n  && up\n```\n> Back up *first*.\n> Always."
            ),
            vec![
                para(vec![text("Run:")]),
                Block::Code("docker compose pull\n  && up".into()),
                Block::Quote(vec![para(vec![
                    text("Back up "),
                    Inline::Emphasis(vec![text("first")]),
                    text("."),
                    Inline::Break,
                    text("Always."),
                ])]),
            ]
        );
        // A fence that never closes runs to the end; inside it nothing is read.
        assert_eq!(
            parse("~~~\n# not a heading\n<b>"),
            vec![Block::Code("# not a heading\n<b>".into())]
        );
        assert_eq!(
            inline("Use `order:random` or ``a ` b``."),
            vec![
                text("Use "),
                Inline::Code("order:random".into()),
                text(" or "),
                Inline::Code("a ` b".into()),
                text("."),
            ]
        );
        assert_eq!(inline("a ` b"), vec![text("a ` b")]);
    }

    #[test]
    fn emphasis_leaves_tag_names_alone() {
        assert_eq!(
            inline("**Public mode** and *twice* as __fast__, _really_"),
            vec![
                Inline::Strong(vec![text("Public mode")]),
                text(" and "),
                Inline::Emphasis(vec![text("twice")]),
                text(" as "),
                Inline::Strong(vec![text("fast")]),
                text(", "),
                Inline::Emphasis(vec![text("really")]),
            ]
        );
        for plain_text in [
            "game_of_life and test_pattern",
            "2 * 3 * 4",
            "a ** b",
            "_open but never closed",
            "snake_case_",
        ] {
            assert_eq!(inline(plain_text), vec![text(plain_text)], "{plain_text}");
        }
        assert_eq!(
            inline(r"\*not\* emphasis \_ \\"),
            vec![text(r"*not* emphasis _ \")]
        );
        assert_eq!(
            inline("**bold with `code` and [a link](https://x.org)**"),
            vec![Inline::Strong(vec![
                text("bold with "),
                Inline::Code("code".into()),
                text(" and "),
                link("https://x.org", "a link"),
            ])]
        );
    }

    #[test]
    fn only_web_and_mail_links_open() {
        assert_eq!(
            inline("See the [guide](https://example.org/docs \"Docs\")."),
            vec![
                text("See the "),
                link("https://example.org/docs", "guide"),
                text("."),
            ]
        );
        assert_eq!(
            inline(
                "[mail](mailto:a@b.org) [x](javascript:alert(1)) [y](data:text/html,hi) [z](/local)"
            ),
            vec![link("mailto:a@b.org", "mail"), text(" x y z"),]
        );
        // Bare addresses, without what closes the sentence around them.
        assert_eq!(
            inline(
                "Changes: https://github.com/a/b/compare/v0.2.0...v0.3.0. (see https://x.org/a_(b))"
            ),
            vec![
                text("Changes: "),
                link(
                    "https://github.com/a/b/compare/v0.2.0...v0.3.0",
                    "https://github.com/a/b/compare/v0.2.0...v0.3.0"
                ),
                text(". (see "),
                link("https://x.org/a_(b)", "https://x.org/a_(b)"),
                text(")"),
            ]
        );
        assert_eq!(
            inline("<https://x.org> and [not a link] (here)"),
            vec![
                link("https://x.org", "https://x.org"),
                text(" and [not a link] (here)")
            ]
        );
        // A link's text holds no other link.
        assert_eq!(
            inline("[https://a.org](https://b.org)"),
            vec![link("https://b.org", "https://a.org")]
        );
        assert_eq!(safe_href("HTTPS://X.ORG"), Some("HTTPS://X.ORG".into()));
        assert_eq!(safe_href("https://"), None);
        assert_eq!(safe_href("https://a b"), None);
    }

    #[test]
    fn html_and_pictures_never_reach_the_page() {
        assert_eq!(
            parse(
                "<img src=\"https://t.example/p.png\"> <script>alert(1)</script>\n\nok<br>next <b>bold</b><!-- note -->"
            ),
            vec![para(vec![text("ok"), Inline::Break, text("next bold"),])]
        );
        assert_eq!(
            parse("<details>\n<summary>More</summary>\n\nHidden\n</details>"),
            vec![para(vec![text("More")]), para(vec![text("Hidden")])]
        );
        assert_eq!(
            inline("![a chart](https://t.example/c.png) <SCRIPT type=x>bad()</Script>after"),
            vec![text("a chart after")]
        );
        assert_eq!(inline("1 < 2 > 0"), vec![text("1 < 2 > 0")]);
    }

    #[test]
    fn github_release_notes_read_as_written() {
        let notes = "## What's Changed\r\n* Public mode by @owner in https://github.com/StarBooru/starbooru-server/pull/12\r\n* Faster search\r\n\r\n**Full Changelog**: https://github.com/StarBooru/starbooru-server/compare/v0.2.0...v0.3.0";
        let blocks = parse(notes);
        assert_eq!(blocks.len(), 3);
        assert_eq!(
            blocks.first(),
            Some(&Block::Heading(2, vec![text("What's Changed")]))
        );
        let Some(Block::List { items, .. }) = blocks.get(1) else {
            panic!("{blocks:?}");
        };
        assert_eq!(
            items.first(),
            Some(&vec![para(vec![
                text("Public mode by @owner in "),
                link(
                    "https://github.com/StarBooru/starbooru-server/pull/12",
                    "https://github.com/StarBooru/starbooru-server/pull/12"
                ),
            ])])
        );
        let Some(Block::Paragraph(last)) = blocks.get(2) else {
            panic!("{blocks:?}");
        };
        assert_eq!(
            plain(last),
            "Full Changelog: https://github.com/StarBooru/starbooru-server/compare/v0.2.0...v0.3.0"
        );
    }
}
