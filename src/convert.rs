// Placeholder to check API
use pulldown_cmark::{Event, Tag, TagEnd, Options, Parser, CodeBlockKind, HeadingLevel, BlockQuoteKind};

fn _check_api() {
    let parser = Parser::new_ext("# Hello", Options::empty());
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => { let _: HeadingLevel = level; }
                Tag::Paragraph => {}
                Tag::CodeBlock(kind) => match kind {
                    CodeBlockKind::Indented => {}
                    CodeBlockKind::Fenced(info) => { let _: &str = &info; }
                },
                Tag::List(start) => { let _: Option<u64> = start; }
                Tag::Item => {}
                Tag::BlockQuote(kind) => { let _: Option<BlockQuoteKind> = kind; }
                Tag::Emphasis => {}
                Tag::Strong => {}
                Tag::Strikethrough => {}
                Tag::Link { dest_url, .. } => { let _: &str = &dest_url; }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_level) => {}
                TagEnd::Paragraph => {}
                TagEnd::CodeBlock => {}
                TagEnd::List(_ordered) => {}
                TagEnd::Item => {}
                TagEnd::BlockQuote(_) => {}
                TagEnd::Emphasis => {}
                TagEnd::Strong => {}
                TagEnd::Strikethrough => {}
                TagEnd::Link => {}
                _ => {}
            },
            Event::Text(s) => { let _: &str = &s; }
            Event::Code(s) => { let _: &str = &s; }
            Event::SoftBreak => {}
            Event::HardBreak => {}
            Event::Rule => {}
            _ => {}
        }
    }
}
