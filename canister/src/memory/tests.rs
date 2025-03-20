mod request_counter_tests {
    use crate::memory::next_request_id;
    use std::collections::BTreeSet;
    use assert_matches::assert_matches;

    #[test]
    fn should_increment_request_id() {
        let request_ids = (0..10)
            .into_iter()
            .map(|_| next_request_id().to_string())
            .collect::<BTreeSet<_>>();

        assert_matches!(request_ids.len(), 10);
    }
}
