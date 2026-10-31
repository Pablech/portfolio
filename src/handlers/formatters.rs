use crate::handlers::models::TerminalRenderable;

fn format_block(title: &str, content: &str) -> String {
    if content.trim().is_empty() {
        return String::new();
    }

    if title == "@RAW" {
        return content.lines()
            .map(|line| format!("  {}", line))
            .collect::<Vec<_>>().join("\n");
    }

    let indent = if title.trim().is_empty() { "    " } else { "      " };

    let body = content.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n");

    if title.trim().is_empty() {
        body
    } else {
        format!("    {}:\n{}", title, body)
    }
}

pub fn format_section<T: TerminalRenderable>(items: &[T]) -> String {
    items.iter()
        .map(|item| {
            let mut block = format!("{}:\n", item.header());

            for (title, content) in item.sections() {
                let formated = format_block(&title, &content);
                if !formated.is_empty() {
                    block.push_str(&formated);
                    block.push('\n');
                }
            }
            block
        })
        .collect::<Vec<_>>().join("\n")
}
