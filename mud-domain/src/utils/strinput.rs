use std::iter::Iterator;

#[derive(Debug, Clone, Copy)]
pub struct StrInput<'a>(pub &'a str);

impl<'a> StrInput<'a> {
    pub fn as_str(self) -> &'a str {
       self.0
    }

    pub fn first(self) -> &'a str {
        self.parse_arguments().first().unwrap_or(self.0)
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
    pub fn plain_arguments(self) -> &'a &str {
        self.0.position(|ch| ch == ' ')
            .map(|index| &self.0[index..])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {

    }
}
