//! Markdown drawn as elements (`logic::markdown` reads it). Text only ever
//! becomes text nodes, and links open in a tab of their own. In a forum
//! post, the posts of the booru it names become links to them.

use leptos::prelude::*;

use super::mention::Mention;
use crate::logic::forum::{Segment, segments};
use crate::logic::markdown::{Block, Inline, parse};

/// `text` as Markdown. Its headings start a level under the card's title.
/// With `mentions`, `>>123` and `post #123` link to those posts.
#[component]
pub fn Markdown(#[prop(into)] text: String, #[prop(optional)] mentions: bool) -> impl IntoView {
    let blocks = parse(&text);
    view! {
        <div class="markdown">
            {blocks.into_iter().map(|b| block(b, mentions)).collect_view()}
        </div>
    }
}

fn block(b: Block, mentions: bool) -> AnyView {
    let runs = move |runs| runs_view(runs, mentions);
    match b {
        Block::Heading(1 | 2, r) => view! { <h3>{runs(r)}</h3> }.into_any(),
        Block::Heading(3, r) => view! { <h4>{runs(r)}</h4> }.into_any(),
        Block::Heading(_, r) => view! { <h5>{runs(r)}</h5> }.into_any(),
        Block::Paragraph(r) => view! { <p>{runs(r)}</p> }.into_any(),
        Block::List {
            ordered: true,
            start,
            items,
        } => view! { <ol start=start>{items_view(items, mentions)}</ol> }.into_any(),
        Block::List { items, .. } => view! { <ul>{items_view(items, mentions)}</ul> }.into_any(),
        Block::Quote(inner) => view! {
            <blockquote>{inner.into_iter().map(|b| block(b, mentions)).collect_view()}</blockquote>
        }
        .into_any(),
        Block::Code(code) => view! {
            <pre class="command">
                <code>{code}</code>
            </pre>
        }
        .into_any(),
        Block::Rule => view! { <hr /> }.into_any(),
    }
}

fn items_view(items: Vec<Vec<Block>>, mentions: bool) -> impl IntoView {
    items
        .into_iter()
        .map(|item| {
            view! { <li>{item.into_iter().map(|b| block(b, mentions)).collect_view()}</li> }
        })
        .collect_view()
}

fn runs_view(runs: Vec<Inline>, mentions: bool) -> AnyView {
    runs.into_iter()
        .map(|r| run(r, mentions))
        .collect_view()
        .into_any()
}

fn run(r: Inline, mentions: bool) -> AnyView {
    match r {
        Inline::Text(text) if mentions => segments(&text)
            .into_iter()
            .map(|s| match s {
                Segment::Text(t) => t.into_any(),
                Segment::Mention(id, written) => {
                    view! { <Mention id=id text=written /> }.into_any()
                }
            })
            .collect_view()
            .into_any(),
        Inline::Text(text) => text.into_any(),
        Inline::Strong(inner) => view! { <strong>{runs_view(inner, mentions)}</strong> }.into_any(),
        Inline::Emphasis(inner) => view! { <em>{runs_view(inner, mentions)}</em> }.into_any(),
        Inline::Code(code) => view! { <code>{code}</code> }.into_any(),
        // No link inside a link.
        Inline::Link { href, text } => view! {
            <a href=href target="_blank" rel="noopener noreferrer">
                {runs_view(text, false)}
            </a>
        }
        .into_any(),
        Inline::Break => view! { <br /> }.into_any(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn nothing_is_written_as_html() {
        let source = include_str!("markdown.rs");
        let code = source.split("#[cfg(test)]").next().unwrap_or_default();
        assert!(!code.contains(concat!("inner", "_html")));
        assert!(!code.contains(concat!("set_inner", "_html")));
    }
}
