//! Markdown drawn as elements (`logic::markdown` reads it). Text only ever
//! becomes text nodes, and links open in a tab of their own.

use leptos::prelude::*;

use crate::logic::markdown::{Block, Inline, parse};

/// `text` as Markdown. Its headings start a level under the card's title.
#[component]
pub fn Markdown(#[prop(into)] text: String) -> impl IntoView {
    let blocks = parse(&text);
    view! { <div class="markdown">{blocks.into_iter().map(block).collect_view()}</div> }
}

fn block(b: Block) -> AnyView {
    match b {
        Block::Heading(1 | 2, runs) => view! { <h3>{runs_view(runs)}</h3> }.into_any(),
        Block::Heading(3, runs) => view! { <h4>{runs_view(runs)}</h4> }.into_any(),
        Block::Heading(_, runs) => view! { <h5>{runs_view(runs)}</h5> }.into_any(),
        Block::Paragraph(runs) => view! { <p>{runs_view(runs)}</p> }.into_any(),
        Block::List {
            ordered: true,
            start,
            items,
        } => view! { <ol start=start>{items_view(items)}</ol> }.into_any(),
        Block::List { items, .. } => view! { <ul>{items_view(items)}</ul> }.into_any(),
        Block::Quote(inner) => {
            view! { <blockquote>{inner.into_iter().map(block).collect_view()}</blockquote> }
                .into_any()
        }
        Block::Code(code) => view! {
            <pre class="command">
                <code>{code}</code>
            </pre>
        }
        .into_any(),
        Block::Rule => view! { <hr /> }.into_any(),
    }
}

fn items_view(items: Vec<Vec<Block>>) -> impl IntoView {
    items
        .into_iter()
        .map(|item| view! { <li>{item.into_iter().map(block).collect_view()}</li> })
        .collect_view()
}

fn runs_view(runs: Vec<Inline>) -> AnyView {
    runs.into_iter().map(run).collect_view().into_any()
}

fn run(r: Inline) -> AnyView {
    match r {
        Inline::Text(text) => text.into_any(),
        Inline::Strong(inner) => view! { <strong>{runs_view(inner)}</strong> }.into_any(),
        Inline::Emphasis(inner) => view! { <em>{runs_view(inner)}</em> }.into_any(),
        Inline::Code(code) => view! { <code>{code}</code> }.into_any(),
        Inline::Link { href, text } => view! {
            <a href=href target="_blank" rel="noopener noreferrer">
                {runs_view(text)}
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
