use crate::config::Category;

pub fn categorizer(categories: &Vec<Category>, description: &str) -> String {
    let mut result = "Unknown".to_string();
    let mut last_prio_level = 0;

    for category in categories {
        for keyword in category.keywords() {
            if description
                .to_lowercase()
                .contains(keyword.to_lowercase().as_str())
                && category.priority() > last_prio_level
            {
                result = category.title().to_string();
                last_prio_level = category.priority();
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Vec<Category> {
        serde_yaml::from_str(
            "
            - title: groceries
              keywords: [woolworths, coles, pearl]
              priority: 60
            - title: fuel
              keywords: [ampol, pearl]
              priority: 70
            ",
        )
        .unwrap()
    }

    #[test]
    fn assigns_category_on_keyword_match() {
        let desc = "WOOLWORTHS 2764 NERANG QL AUS Card xx8935".to_string();
        assert_eq!(categorizer(&rules(), &desc), "groceries");
    }

    #[test]
    fn higher_priority_wins_when_multiple_keywords_match() {
        let desc = "PEARL SOUTHPORT SOUTHPORT QL AUS Card xx5231".to_string();
        assert_eq!(categorizer(&rules(), &desc), "fuel");
    }

    #[test]
    fn returns_unknown_when_no_match() {
        assert_eq!(categorizer(&rules(), &"EasyPark"), "Unknown");
    }
}
