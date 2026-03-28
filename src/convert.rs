//! Markdown <-> Tiptap JSON conversion.
//!
//! - [`markdown_to_tiptap_json`]: parses Markdown via `pulldown_cmark` and
//!   builds a Tiptap-compatible JSON document tree.
//! - [`tiptap_json_to_markdown`]: walks a Tiptap JSON tree and emits Markdown.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// Markdown -> Tiptap JSON
// ---------------------------------------------------------------------------

/// Convert a Markdown string into a Tiptap-compatible JSON document.
///
/// The returned value has the shape:
/// ```json
/// { "type": "doc", "content": [ ... ] }
/// ```
pub fn markdown_to_tiptap_json(md: &str) -> Value {
    let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TABLES;
    let parser = Parser::new_ext(md, opts);

    // Stack of in-progress nodes.  Each entry is a JSON object being built.
    let mut stack: Vec<Value> = vec![json!({"type": "doc", "content": []})];

    // Active inline marks (bold, italic, strike, link, ...).
    let mut marks: Vec<Value> = Vec::new();

    for event in parser {
        match event {
            // ----- block-level start tags -----
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let level_num = heading_level_to_u8(level);
                    stack.push(json!({
                        "type": "heading",
                        "attrs": { "level": level_num },
                        "content": []
                    }));
                }
                Tag::Paragraph => {
                    stack.push(json!({"type": "paragraph", "content": []}));
                }
                Tag::CodeBlock(kind) => {
                    let lang = match &kind {
                        CodeBlockKind::Fenced(info) => {
                            let s = info.split_whitespace().next().unwrap_or("");
                            s.to_string()
                        }
                        CodeBlockKind::Indented => String::new(),
                    };
                    stack.push(json!({
                        "type": "codeBlock",
                        "attrs": { "language": lang },
                        "content": []
                    }));
                }
                Tag::List(start) => {
                    if let Some(n) = start {
                        stack.push(json!({
                            "type": "orderedList",
                            "attrs": { "start": n },
                            "content": []
                        }));
                    } else {
                        stack.push(json!({"type": "bulletList", "content": []}));
                    }
                }
                Tag::Item => {
                    stack.push(json!({"type": "listItem", "content": []}));
                }
                Tag::BlockQuote(_) => {
                    stack.push(json!({"type": "blockquote", "content": []}));
                }
                Tag::Table(_) => {
                    stack.push(json!({"type": "table", "content": []}));
                }
                Tag::TableHead => {
                    stack.push(json!({"type": "tableRow", "content": []}));
                }
                Tag::TableRow => {
                    stack.push(json!({"type": "tableRow", "content": []}));
                }
                Tag::TableCell => {
                    stack.push(json!({"type": "tableCell", "content": []}));
                }
                // ----- inline marks -----
                Tag::Strong => {
                    marks.push(json!({"type": "bold"}));
                }
                Tag::Emphasis => {
                    marks.push(json!({"type": "italic"}));
                }
                Tag::Strikethrough => {
                    marks.push(json!({"type": "strike"}));
                }
                Tag::Link { dest_url, .. } => {
                    marks.push(json!({
                        "type": "link",
                        "attrs": { "href": dest_url.as_ref() }
                    }));
                }
                Tag::Image { dest_url, title, .. } => {
                    // Tiptap images are block-level nodes, not inline marks.
                    // We push a temporary container; on End we will emit the
                    // image node with the collected alt text.
                    stack.push(json!({
                        "type": "__image__",
                        "attrs": {
                            "src": dest_url.as_ref(),
                            "title": title.as_ref()
                        },
                        "content": []
                    }));
                }
                _ => {
                    // Unknown block tag -- push a generic wrapper so
                    // Start/End stays balanced.
                    stack.push(json!({"type": "paragraph", "content": []}));
                }
            },

            // ----- block-level end tags -----
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_)
                | TagEnd::Paragraph
                | TagEnd::CodeBlock
                | TagEnd::List(_)
                | TagEnd::Item
                | TagEnd::BlockQuote(_)
                | TagEnd::Table
                | TagEnd::TableHead
                | TagEnd::TableRow
                | TagEnd::TableCell => {
                    pop_and_append(&mut stack);
                }
                TagEnd::Strong => {
                    pop_mark(&mut marks, "bold");
                }
                TagEnd::Emphasis => {
                    pop_mark(&mut marks, "italic");
                }
                TagEnd::Strikethrough => {
                    pop_mark(&mut marks, "strike");
                }
                TagEnd::Link => {
                    pop_mark(&mut marks, "link");
                }
                TagEnd::Image => {
                    // Convert the temporary __image__ container into a real
                    // Tiptap image node.
                    if let Some(mut img) = stack.pop() {
                        // Collect the alt text from any text children.
                        let alt = img["content"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|c| c["text"].as_str())
                                    .collect::<Vec<_>>()
                                    .join("")
                            })
                            .unwrap_or_default();
                        img["type"] = json!("image");
                        img["attrs"]["alt"] = json!(alt);
                        // Images have no content array in Tiptap.
                        img.as_object_mut()
                            .map(|m| m.remove("content"));
                        append_to_parent(&mut stack, img);
                    }
                }
                _ => {
                    pop_and_append(&mut stack);
                }
            },

            // ----- leaf events -----
            Event::Text(s) => {
                let text_node = make_text_node(&s, &marks);
                append_to_parent(&mut stack, text_node);
            }
            Event::Code(s) => {
                let mut code_marks = marks.clone();
                code_marks.push(json!({"type": "code"}));
                let text_node = make_text_node(&s, &code_marks);
                append_to_parent(&mut stack, text_node);
            }
            Event::SoftBreak => {
                // Treat soft breaks as a space character.
                let text_node = make_text_node(" ", &marks);
                append_to_parent(&mut stack, text_node);
            }
            Event::HardBreak => {
                append_to_parent(&mut stack, json!({"type": "hardBreak"}));
            }
            Event::Rule => {
                append_to_parent(&mut stack, json!({"type": "horizontalRule"}));
            }
            Event::Html(html) => {
                // Preserve raw HTML as a paragraph with the raw text.
                let node = json!({
                    "type": "paragraph",
                    "content": [{"type": "text", "text": html.as_ref()}]
                });
                append_to_parent(&mut stack, node);
            }
            _ => {}
        }
    }

    // The root "doc" node should be the only item remaining.
    stack.into_iter().next().unwrap_or(json!({"type": "doc", "content": []}))
}

/// Pop the top node off the stack and append it to the parent's `content` array.
fn pop_and_append(stack: &mut Vec<Value>) {
    if let Some(node) = stack.pop() {
        append_to_parent(stack, node);
    }
}

/// Append `child` to the `content` array of the node on top of `stack`.
fn append_to_parent(stack: &mut [Value], child: Value) {
    if let Some(parent) = stack.last_mut() {
        if let Some(arr) = parent.get_mut("content").and_then(Value::as_array_mut) {
            arr.push(child);
        }
    }
}

/// Remove the most recent mark whose `"type"` matches `mark_type`.
fn pop_mark(marks: &mut Vec<Value>, mark_type: &str) {
    if let Some(pos) = marks
        .iter()
        .rposition(|m| m["type"].as_str() == Some(mark_type))
    {
        marks.remove(pos);
    }
}

/// Build a `{"type":"text", "text":"...", "marks":[...]}` node.
fn make_text_node(text: &str, marks: &[Value]) -> Value {
    let mut node = json!({"type": "text", "text": text});
    if !marks.is_empty() {
        node["marks"] = json!(marks);
    }
    node
}

/// Map `pulldown_cmark::HeadingLevel` to a plain u8.
fn heading_level_to_u8(level: pulldown_cmark::HeadingLevel) -> u8 {
    use pulldown_cmark::HeadingLevel::*;
    match level {
        H1 => 1,
        H2 => 2,
        H3 => 3,
        H4 => 4,
        H5 => 5,
        H6 => 6,
    }
}

// ---------------------------------------------------------------------------
// Tiptap JSON -> Markdown
// ---------------------------------------------------------------------------

/// Convert a Tiptap JSON document tree back into Markdown text.
pub fn tiptap_json_to_markdown(json: &Value) -> String {
    node_to_md(json, "")
}

fn node_to_md(node: &Value, indent: &str) -> String {
    match node["type"].as_str() {
        Some("doc") => children_to_md(node, indent),

        Some("heading") => {
            let level = node["attrs"]["level"].as_u64().unwrap_or(1) as usize;
            let hashes = "#".repeat(level);
            format!("{}{} {}\n\n", indent, hashes, inline_content(node))
        }

        Some("paragraph") => {
            let text = inline_content(node);
            if text.is_empty() {
                // Empty paragraph -- just a blank line.
                "\n".to_string()
            } else {
                format!("{}{}\n\n", indent, text)
            }
        }

        Some("codeBlock") => {
            let lang = node["attrs"]["language"].as_str().unwrap_or("");
            let text = code_block_text(node);
            format!("{}```{}\n{}\n{}```\n\n", indent, lang, text, indent)
        }

        Some("bulletList") => {
            let items = node["content"].as_array();
            let mut out = String::new();
            if let Some(items) = items {
                for item in items {
                    out.push_str(&list_item_to_md(item, indent, "- "));
                }
            }
            out.push('\n');
            out
        }

        Some("orderedList") => {
            let start = node["attrs"]["start"].as_u64().unwrap_or(1);
            let items = node["content"].as_array();
            let mut out = String::new();
            if let Some(items) = items {
                for (i, item) in items.iter().enumerate() {
                    let prefix = format!("{}. ", start + i as u64);
                    out.push_str(&list_item_to_md(item, indent, &prefix));
                }
            }
            out.push('\n');
            out
        }

        Some("listItem") => {
            // Normally rendered by bulletList / orderedList, but handle
            // standalone for robustness.
            children_to_md(node, indent)
        }

        Some("blockquote") => {
            let inner = children_to_md(node, "");
            let mut out = String::new();
            for line in inner.lines() {
                out.push_str(indent);
                out.push_str("> ");
                out.push_str(line);
                out.push('\n');
            }
            out.push('\n');
            out
        }

        Some("horizontalRule") => format!("{}---\n\n", indent),

        Some("hardBreak") => "  \n".to_string(),

        Some("image") => {
            let src = node["attrs"]["src"].as_str().unwrap_or("");
            let alt = node["attrs"]["alt"].as_str().unwrap_or("");
            format!("{}![{}]({})\n\n", indent, alt, src)
        }

        Some("table") => table_to_md(node, indent),

        Some("text") => render_text(node),

        _ => children_to_md(node, indent),
    }
}

/// Render all children of a node, concatenated.
fn children_to_md(node: &Value, indent: &str) -> String {
    let mut out = String::new();
    if let Some(children) = node["content"].as_array() {
        for child in children {
            out.push_str(&node_to_md(child, indent));
        }
    }
    out
}

/// Render the inline content of a block node (heading, paragraph, etc.).
fn inline_content(node: &Value) -> String {
    let mut out = String::new();
    if let Some(children) = node["content"].as_array() {
        for child in children {
            match child["type"].as_str() {
                Some("text") => out.push_str(&render_text(child)),
                Some("hardBreak") => out.push_str("  \n"),
                _ => out.push_str(&inline_content(child)),
            }
        }
    }
    out
}

/// Extract the plain text from a codeBlock's children.
fn code_block_text(node: &Value) -> String {
    node["content"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c["text"].as_str())
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

/// Render a single text node, wrapping it with any active marks.
fn render_text(node: &Value) -> String {
    let text = node["text"].as_str().unwrap_or("");
    let marks = node["marks"].as_array();
    let mut result = text.to_string();

    if let Some(marks) = marks {
        // Apply marks from innermost to outermost (reverse order).
        for mark in marks.iter().rev() {
            match mark["type"].as_str() {
                Some("bold") => result = format!("**{}**", result),
                Some("italic") => result = format!("*{}*", result),
                Some("code") => result = format!("`{}`", result),
                Some("strike") => result = format!("~~{}~~", result),
                Some("link") => {
                    let href = mark["attrs"]["href"].as_str().unwrap_or("");
                    result = format!("[{}]({})", result, href);
                }
                _ => {}
            }
        }
    }
    result
}

/// Render a single list item with the given prefix ("- " or "1. ").
fn list_item_to_md(item: &Value, indent: &str, prefix: &str) -> String {
    let mut out = String::new();
    let child_indent = format!("{}{}", indent, " ".repeat(prefix.len()));

    if let Some(children) = item["content"].as_array() {
        for (i, child) in children.iter().enumerate() {
            if i == 0 {
                // First child gets the bullet/number prefix.
                let inner = inline_or_block(child, &child_indent);
                let trimmed = inner.trim_end_matches('\n');
                out.push_str(indent);
                out.push_str(prefix);
                out.push_str(trimmed);
                out.push('\n');
            } else {
                // Subsequent children are indented continuation.
                out.push_str(&node_to_md(child, &child_indent));
            }
        }
    }
    out
}

/// Render a child node that may be a paragraph (inline) or a nested block.
fn inline_or_block(node: &Value, indent: &str) -> String {
    match node["type"].as_str() {
        Some("paragraph") => inline_content(node),
        _ => node_to_md(node, indent),
    }
}

/// Render a Tiptap table as a GitHub-flavored Markdown table.
fn table_to_md(node: &Value, indent: &str) -> String {
    let rows = match node["content"].as_array() {
        Some(r) => r,
        None => return String::new(),
    };

    let mut out = String::new();

    for (row_idx, row) in rows.iter().enumerate() {
        let cells = match row["content"].as_array() {
            Some(c) => c,
            None => continue,
        };

        out.push_str(indent);
        out.push('|');
        for cell in cells {
            let text = inline_content(cell);
            out.push_str(&format!(" {} |", text));
        }
        out.push('\n');

        // After the first (header) row, emit the separator line.
        if row_idx == 0 {
            out.push_str(indent);
            out.push('|');
            for _ in cells {
                out.push_str(" --- |");
            }
            out.push('\n');
        }
    }

    out.push('\n');
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_heading() {
        let md = "# Hello World\n";
        let json = markdown_to_tiptap_json(md);
        assert_eq!(json["type"], "doc");
        let first = &json["content"][0];
        assert_eq!(first["type"], "heading");
        assert_eq!(first["attrs"]["level"], 1);
        assert_eq!(first["content"][0]["text"], "Hello World");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("# Hello World"));
    }

    #[test]
    fn round_trip_paragraph_with_bold() {
        let md = "Hello **bold** world\n";
        let json = markdown_to_tiptap_json(md);
        let para = &json["content"][0];
        assert_eq!(para["type"], "paragraph");

        // Find the bold text node.
        let bold_node = para["content"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["text"] == "bold")
            .expect("should have bold text node");
        assert_eq!(bold_node["marks"][0]["type"], "bold");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("**bold**"));
    }

    #[test]
    fn round_trip_code_block() {
        let md = "```rust\nfn main() {}\n```\n";
        let json = markdown_to_tiptap_json(md);
        let cb = &json["content"][0];
        assert_eq!(cb["type"], "codeBlock");
        assert_eq!(cb["attrs"]["language"], "rust");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("```rust"));
        assert!(back.contains("fn main() {}"));
    }

    #[test]
    fn round_trip_bullet_list() {
        let md = "- Item 1\n- Item 2\n";
        let json = markdown_to_tiptap_json(md);
        let list = &json["content"][0];
        assert_eq!(list["type"], "bulletList");
        assert_eq!(list["content"].as_array().unwrap().len(), 2);

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("- Item 1"));
        assert!(back.contains("- Item 2"));
    }

    #[test]
    fn round_trip_ordered_list() {
        let md = "1. First\n2. Second\n";
        let json = markdown_to_tiptap_json(md);
        let list = &json["content"][0];
        assert_eq!(list["type"], "orderedList");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("1. First"));
        assert!(back.contains("2. Second"));
    }

    #[test]
    fn round_trip_blockquote() {
        let md = "> quoted text\n";
        let json = markdown_to_tiptap_json(md);
        let bq = &json["content"][0];
        assert_eq!(bq["type"], "blockquote");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("> quoted text"));
    }

    #[test]
    fn round_trip_horizontal_rule() {
        let md = "---\n";
        let json = markdown_to_tiptap_json(md);
        let hr = &json["content"][0];
        assert_eq!(hr["type"], "horizontalRule");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("---"));
    }

    #[test]
    fn inline_code() {
        let md = "Use `println!` here\n";
        let json = markdown_to_tiptap_json(md);
        let para = &json["content"][0];
        let code_node = para["content"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["text"] == "println!")
            .expect("should have code text node");
        assert_eq!(code_node["marks"][0]["type"], "code");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("`println!`"));
    }

    #[test]
    fn link_mark() {
        let md = "[click here](https://example.com)\n";
        let json = markdown_to_tiptap_json(md);
        let para = &json["content"][0];
        let link_node = para["content"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["text"] == "click here")
            .expect("should have link text node");
        assert_eq!(link_node["marks"][0]["type"], "link");
        assert_eq!(
            link_node["marks"][0]["attrs"]["href"],
            "https://example.com"
        );

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("[click here](https://example.com)"));
    }

    #[test]
    fn strikethrough() {
        let md = "~~deleted~~\n";
        let json = markdown_to_tiptap_json(md);
        let para = &json["content"][0];
        let strike_node = para["content"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["text"] == "deleted")
            .expect("should have strike text node");
        assert_eq!(strike_node["marks"][0]["type"], "strike");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("~~deleted~~"));
    }

    #[test]
    fn complex_document() {
        let md = "\
# Title

Some **bold** and *italic* text.

- Item 1
- Item 2

```python
print('hello')
```

---

> A quote
";
        let json = markdown_to_tiptap_json(md);
        assert_eq!(json["type"], "doc");

        let back = tiptap_json_to_markdown(&json);
        assert!(back.contains("# Title"));
        assert!(back.contains("**bold**"));
        assert!(back.contains("*italic*"));
        assert!(back.contains("- Item 1"));
        assert!(back.contains("```python"));
        assert!(back.contains("print('hello')"));
        assert!(back.contains("---"));
        assert!(back.contains("> A quote"));
    }

    #[test]
    fn tiptap_json_to_md_standalone() {
        let doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": { "level": 2 },
                    "content": [{ "type": "text", "text": "Section" }]
                },
                {
                    "type": "paragraph",
                    "content": [
                        { "type": "text", "text": "Normal " },
                        {
                            "type": "text",
                            "text": "bold",
                            "marks": [{ "type": "bold" }]
                        }
                    ]
                }
            ]
        });

        let md = tiptap_json_to_markdown(&doc);
        assert!(md.contains("## Section"));
        assert!(md.contains("Normal **bold**"));
    }
}
