use std::iter::Iterator;

#[derive(Debug, Clone, Copy)]
pub struct StrInput<'a>(pub &'a str);

impl<'a> StrInput<'a> {
    pub fn as_str(self) -> &'a str {
       self.0
    }

    pub fn first(self) -> &'a str {
        let first_space = self.0.find(' ');
        match first_space {
            Some(position) => &self.0[..position],
            None => self.0,
        }
    }

    pub fn has_command(self, command: &str) -> bool {
        self.0 == command || self.0.starts_with(&format!("{} ", command))
    }

    pub fn has_commands(self, commands: &[&str]) -> bool {
        commands.iter()
            .find(|command| self.has_command(command))
            .is_some()
    }

    pub fn split(self) -> Vec<&'a str> {
        self.0.split_ascii_whitespace().collect()
    }

    pub fn parse_arguments(self) -> Vec<&'a str> {
        let mut parts = self.0.split_ascii_whitespace();
        // drop command
        parts.next();
        parts.collect()
    }

    /// returns empty string if have no extra arguments
    pub fn plain_arguments(self) -> &'a str {
        let first_space = self.0.find(' ');
        match first_space {
            Some(position) => &self.0[position + 1..],
            None => "",
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn has_command_test() {
        let input = StrInput("kill all the things");
        assert!(input.has_command("kill"));
        assert!(!input.has_command("all"));
        assert!(!input.has_command("kil"));
    }

    #[test]
    fn has_commands_test() {
        let input = StrInput("k all the things");
        assert!(input.has_commands(&["kill", "k"]));
        assert!(!input.has_commands(&["kill", "ki"]));
    }

    #[test]
    fn parse_arguments_test() {
        let input = StrInput("kill all the things");
        let vec = input.parse_arguments();
        assert_eq!(3, vec.len());
        assert_eq!(Some(&"all"), vec.get(0));
        assert_eq!(Some(&"the"), vec.get(1));
        assert_eq!(Some(&"things"), vec.get(2));
    }

    #[test]
    fn plain_arguments_test() {
        let input = StrInput("kill all the things");
        assert_eq!("all the things", input.plain_arguments());

        let input = StrInput("kill");
        assert!(input.plain_arguments().is_empty());
    }

    #[test]
    fn split_test() {
        let input = StrInput("kill all the things");
        let vec = input.split();
        assert_eq!(4, vec.len());
        assert_eq!(Some(&"kill"), vec.get(0));
        assert_eq!(Some(&"all"), vec.get(1));
        assert_eq!(Some(&"the"), vec.get(2));
        assert_eq!(Some(&"things"), vec.get(3));
    }

    #[test]
    fn first_test() {
        let input = StrInput("kill");
        assert_eq!("kill", input.first());

        let input = StrInput("kill all the things");
        assert_eq!("kill", input.first());
    }
}
