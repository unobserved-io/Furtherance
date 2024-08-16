#[cfg(test)]
mod timer_tests {
    use crate::app::*;

    #[test]
    fn test_split_task_input_basic() {
        let input = "Check @Proj $10.0 #Bond james bond #007";
        let expected = (
            "Check".to_string(),
            "Proj".to_string(),
            "Bond james bond #007".to_string(),
            10.0,
        );
        assert_eq!(split_task_input(input), Some(expected));
    }

    #[test]
    fn test_split_task_input_no_tags() {
        let input = "Task without tags @Proj $5";
        let expected = (
            "Task without tags".to_string(),
            "Proj".to_string(),
            "".to_string(),
            5.0,
        );
        assert_eq!(split_task_input(input), Some(expected));
    }

    #[test]
    fn test_split_task_input_no_project() {
        let input = "Task with no project $15.00 #tag1 #tag2";
        let expected = (
            "Task with no project".to_string(),
            "".to_string(),
            "tag1 #tag2".to_string(),
            15.0,
        );
        assert_eq!(split_task_input(input), Some(expected));
    }

    #[test]
    fn test_split_task_input_no_rate() {
        let input = "Task with no rate @A New project #tag1 #tag2";
        let expected = (
            "Task with no rate".to_string(),
            "A New project".to_string(),
            "tag1 #tag2".to_string(),
            0.0,
        );
        assert_eq!(split_task_input(input), Some(expected));
    }

    #[test]
    fn test_split_task_input_project_at_end() {
        let input = "Task with no rate #tag1 #tag2 @A New Project";
        let expected = (
            "Task with no rate".to_string(),
            "A New Project".to_string(),
            "tag1 #tag2".to_string(),
            0.0,
        );
        assert_eq!(split_task_input(input), Some(expected));
    }

    #[test]
    fn test_split_task_input_empty() {
        let input = "";
        assert_eq!(split_task_input(input), None);
    }
}
