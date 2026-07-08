use serde_json::{Map, Value};

pub fn render(template: &Value, context: &Value) -> Value {
    match template {
        Value::String(s) => Value::String(render_string(s, context)),
        Value::Array(arr) => Value::Array(arr.iter().map(|v| render(v, context)).collect()),
        Value::Object(obj) => {
            let mut out = Map::with_capacity(obj.len());
            for (k, v) in obj {
                out.insert(k.clone(), render(v, context));
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}

fn render_string(template: &str, context: &Value) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);

        let after_open = &rest[start + 2..];
        let Some(end_rel) = after_open.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };

        let raw_path = &after_open[..end_rel];
        let path = raw_path.trim();

        match navigate(context, path) {
            Some(value) => out.push_str(&stringify(value)),
            None => {
                out.push_str("{{");
                out.push_str(raw_path);
                out.push_str("}}");
            }
        }

        rest = &after_open[end_rel + 2..];
    }

    out.push_str(rest);
    out
}

fn navigate<'a>(context: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = context;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn stringify(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plain_string_passes_through() {
        let out = render(&json!("hello world"), &json!({}));
        assert_eq!(out, json!("hello world"));
    }

    #[test]
    fn non_string_values_pass_through() {
        assert_eq!(render(&json!(42), &json!({})), json!(42));
        assert_eq!(render(&json!(true), &json!({})), json!(true));
        assert_eq!(render(&json!(null), &json!({})), json!(null));
    }

    #[test]
    fn top_level_placeholder_substitutes() {
        let template = json!("Hello {{name}}!");
        let context = json!({ "name": "Alice" });
        assert_eq!(render(&template, &context), json!("Hello Alice!"));
    }

    #[test]
    fn nested_placeholder_substitutes() {
        let template = json!("CI broken on {{repository.name}}");
        let context = json!({ "repository": { "name": "my-repo" } });
        assert_eq!(render(&template, &context), json!("CI broken on my-repo"));
    }

    #[test]
    fn placeholder_with_spaces_substitutes() {
        // Whitespace inside `{{ ... }}` is trimmed.
        let template = json!("Hi {{ name }}");
        let context = json!({ "name": "Bob" });
        assert_eq!(render(&template, &context), json!("Hi Bob"));
    }

    #[test]
    fn multiple_placeholders_in_one_string() {
        let template = json!("Workflow {{workflow.name}} failed on {{repository.name}}");
        let context = json!({
            "workflow": { "name": "CI" },
            "repository": { "name": "my-repo" }
        });
        assert_eq!(
            render(&template, &context),
            json!("Workflow CI failed on my-repo")
        );
    }

    #[test]
    fn missing_path_leaves_placeholder_literal() {
        let template = json!("Hello {{name}}!");
        let context = json!({});
        assert_eq!(render(&template, &context), json!("Hello {{name}}!"));
    }

    #[test]
    fn missing_nested_path_leaves_placeholder_literal() {
        let template = json!("On {{repository.private}}");
        let context = json!({ "repository": { "name": "my-repo" } });
        assert_eq!(
            render(&template, &context),
            json!("On {{repository.private}}")
        );
    }

    #[test]
    fn number_context_stringifies() {
        let template = json!("Count: {{count}}");
        let context = json!({ "count": 42 });
        assert_eq!(render(&template, &context), json!("Count: 42"));
    }

    #[test]
    fn boolean_context_stringifies() {
        let template = json!("Draft: {{draft}}");
        let context = json!({ "draft": false });
        assert_eq!(render(&template, &context), json!("Draft: false"));
    }

    #[test]
    fn nested_object_template_is_walked() {
        let template = json!({
            "title": "CI broken on {{repository.name}}",
            "severity": "high",
            "body": "Workflow {{workflow.name}} failed"
        });
        let context = json!({
            "repository": { "name": "my-repo" },
            "workflow": { "name": "CI" }
        });

        assert_eq!(
            render(&template, &context),
            json!({
                "title": "CI broken on my-repo",
                "severity": "high",
                "body": "Workflow CI failed"
            })
        );
    }

    #[test]
    fn nested_array_template_is_walked() {
        let template = json!(["{{a}}", "{{b}}", "literal"]);
        let context = json!({ "a": "AAA", "b": "BBB" });
        assert_eq!(
            render(&template, &context),
            json!(["AAA", "BBB", "literal"])
        );
    }

    #[test]
    fn unclosed_placeholder_kept_literal() {
        let template = json!("Hello {{name");
        let context = json!({ "name": "Alice" });
        assert_eq!(render(&template, &context), json!("Hello {{name"));
    }

    #[test]
    fn empty_placeholder_treated_as_missing() {
        let template = json!("Odd: {{}}");
        let context = json!({ "name": "Alice" });
        assert_eq!(render(&template, &context), json!("Odd: {{}}"));
    }

    #[test]
    fn full_vigil_create_incident_reaction() {
        let template = json!({
            "title": "CI broken on {{repository.name}}",
            "severity": "high",
            "body": "Workflow {{workflow.name}} failed — [View run]({{run.url}})"
        });
        let context = json!({
            "repository": { "name": "my-repo", "id": 42 },
            "workflow": { "name": "CI" },
            "run": { "url": "https://github.com/org/repo/actions/runs/12345" }
        });

        assert_eq!(
            render(&template, &context),
            json!({
                "title": "CI broken on my-repo",
                "severity": "high",
                "body": "Workflow CI failed — [View run](https://github.com/org/repo/actions/runs/12345)"
            })
        );
    }
}
