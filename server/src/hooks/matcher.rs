use serde_json::Value;

pub fn matches(payload: &Value, filters: &Value) -> bool {
    let Some(filters_map) = filters.as_object() else {
        return true;
    };

    if filters_map.is_empty() {
        return true;
    }

    filters_map
        .iter()
        .all(|(path, expected)| match navigate(payload, path) {
            Some(actual) => values_match(actual, expected),
            None => false,
        })
}

fn navigate<'a>(payload: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = payload;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn values_match(actual: &Value, expected: &Value) -> bool {
    actual == expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_filters_match_anything() {
        assert!(matches(&json!({"a": 1}), &json!({})));
        assert!(matches(&json!(null), &json!({})));
        assert!(matches(&json!({"nested": {"deep": true}}), &json!({})));
    }

    #[test]
    fn non_object_filters_are_treated_as_empty() {
        assert!(matches(&json!({"a": 1}), &json!(null)));
        assert!(matches(&json!({"a": 1}), &json!("weird")));
    }

    #[test]
    fn top_level_string_match() {
        let payload = json!({ "conclusion": "failure" });
        let filters = json!({ "conclusion": "failure" });
        assert!(matches(&payload, &filters));
    }

    #[test]
    fn top_level_string_mismatch() {
        let payload = json!({ "conclusion": "success" });
        let filters = json!({ "conclusion": "failure" });
        assert!(!matches(&payload, &filters));
    }

    #[test]
    fn missing_key_does_not_match() {
        let payload = json!({ "action": "completed" });
        let filters = json!({ "conclusion": "failure" });
        assert!(!matches(&payload, &filters));
    }

    #[test]
    fn nested_path_match() {
        let payload = json!({
            "workflow_run": { "conclusion": "failure" }
        });
        let filters = json!({ "workflow_run.conclusion": "failure" });
        assert!(matches(&payload, &filters));
    }

    #[test]
    fn nested_path_mismatch() {
        let payload = json!({
            "workflow_run": { "conclusion": "success" }
        });
        let filters = json!({ "workflow_run.conclusion": "failure" });
        assert!(!matches(&payload, &filters));
    }

    #[test]
    fn nested_path_missing_intermediate() {
        let payload = json!({ "action": "completed" });
        let filters = json!({ "workflow_run.conclusion": "failure" });
        assert!(!matches(&payload, &filters));
    }

    #[test]
    fn deep_nesting_works() {
        let payload = json!({
            "a": { "b": { "c": { "d": "found" } } }
        });
        let filters = json!({ "a.b.c.d": "found" });
        assert!(matches(&payload, &filters));
    }

    #[test]
    fn all_filters_must_match() {
        let payload = json!({
            "workflow_run": { "conclusion": "failure" },
            "repository": { "full_name": "org/repo" }
        });
        let filters = json!({
            "workflow_run.conclusion": "failure",
            "repository.full_name": "org/repo"
        });
        assert!(matches(&payload, &filters));
    }

    #[test]
    fn one_bad_filter_fails_the_whole_match() {
        let payload = json!({
            "workflow_run": { "conclusion": "failure" },
            "repository": { "full_name": "wrong/repo" }
        });
        let filters = json!({
            "workflow_run.conclusion": "failure",
            "repository.full_name": "org/repo"
        });
        assert!(!matches(&payload, &filters));
    }

    #[test]
    fn boolean_values_match() {
        let payload = json!({ "draft": false });
        let filters = json!({ "draft": false });
        assert!(matches(&payload, &filters));
    }

    #[test]
    fn number_values_match() {
        let payload = json!({ "attempt": 3 });
        let filters = json!({ "attempt": 3 });
        assert!(matches(&payload, &filters));
    }

    #[test]
    fn type_mismatch_does_not_match() {
        let payload = json!({ "attempt": 3 });
        let filters = json!({ "attempt": "3" });
        assert!(!matches(&payload, &filters));
    }

    #[test]
    fn full_github_workflow_run_scenario() {
        let payload = json!({
            "action": "completed",
            "workflow_run": {
                "id": 12345,
                "name": "CI",
                "conclusion": "failure",
                "html_url": "https://github.com/org/repo/actions/runs/12345"
            },
            "repository": {
                "full_name": "org/repo",
                "name": "repo"
            }
        });

        let filters = json!({
            "action": "completed",
            "workflow_run.conclusion": "failure",
            "repository.full_name": "org/repo"
        });

        assert!(matches(&payload, &filters));
    }

    #[test]
    fn full_github_workflow_run_wrong_repo_does_not_match() {
        let payload = json!({
            "action": "completed",
            "workflow_run": { "conclusion": "failure" },
            "repository": { "full_name": "someone-else/their-repo" }
        });

        let filters = json!({
            "action": "completed",
            "workflow_run.conclusion": "failure",
            "repository.full_name": "org/repo"
        });

        assert!(!matches(&payload, &filters));
    }
}
