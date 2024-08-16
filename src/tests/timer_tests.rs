// Furtherance - Track your time without being tracked
// Copyright (C) 2024  Ricky Kresslein <rk@unobserved.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
        assert_eq!(split_task_input(input), expected);
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
        assert_eq!(split_task_input(input), expected);
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
        assert_eq!(split_task_input(input), expected);
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
        assert_eq!(split_task_input(input), expected);
    }

    #[test]
    fn test_split_task_input_project_at_end() {
        let input = "Task with project at end #tag1 #tag2 @A New Project";
        let expected = (
            "Task with project at end".to_string(),
            "A New Project".to_string(),
            "tag1 #tag2".to_string(),
            0.0,
        );
        assert_eq!(split_task_input(input), expected);
    }

    #[test]
    fn test_split_task_input_project_space_before() {
        let input = "Project space before @  The proj #tag1 #tag2";
        let expected = (
            "Project space before".to_string(),
            "The proj".to_string(),
            "tag1 #tag2".to_string(),
            0.0,
        );
        assert_eq!(split_task_input(input), expected);
    }
}
