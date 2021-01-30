extern crate actor_ws;
use actor_ws::domain;

#[cfg(test)]
mod utility_testing {
    use super::*;

    #[test]
    fn regex_extractor_test() {
        let scanner = domain::utility::build_extract_backlash("register", 16);
        let result = scanner("register/apple");
        assert_eq!(result, Some("apple".to_string()));
    }

    #[test]
    fn sanizter_test_false() {
        // bad text sample
        let sample = "apple";
        let sanitzier = domain::utility::sanitize_text(sample);
        assert!(!sanitzier);
    }

    #[test]
    fn sanitize_text_true() {
        //[$/,#,?*[\\]]
        let prohibited = vec!["/", "*", "$", "?", "#", "\\"];
        let mut cramp_test = String::new();
        let sample_text = "apple".to_string();
        prohibited.into_iter().for_each(|bad_txt| {
            cramp_test.push_str(bad_txt);
            let sample = format!("{}{}", sample_text, bad_txt);
            let sanitzier = domain::utility::sanitize_text(&sample);
            let cramp_san = domain::utility::sanitize_text(&cramp_test);
            assert!(sanitzier);
            assert!(cramp_san);
        });
    }
}
