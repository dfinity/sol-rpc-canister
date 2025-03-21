mod request_counter_tests {
    use crate::memory::next_request_id;
    use std::collections::BTreeSet;

    #[test]
    fn should_increment_request_id() {
        let request_ids = (0..10)
            .map(|_| next_request_id().to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(request_ids.len(), 10);
    }
}
