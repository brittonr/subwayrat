//! Priority cookie cycling.

pub struct PriorityCycler {
    pub cycle: Vec<char>,
}

impl PriorityCycler {
    pub fn new(cycle: Vec<char>) -> Self { Self { cycle } }

    pub fn default_cycle() -> Self { Self::new(vec!['A', 'B', 'C']) }

    /// Cycle to next priority. None → first, last → None.
    pub fn cycle(&self, current: Option<char>) -> Option<char> {
        match current {
            None => self.cycle.first().copied(),
            Some(c) => {
                let idx = self.cycle.iter().position(|&x| x == c);
                match idx {
                    Some(i) if i + 1 < self.cycle.len() => Some(self.cycle[i + 1]),
                    _ => None,
                }
            }
        }
    }
}

pub fn format_priority(p: Option<char>) -> String {
    match p { Some(c) => format!("[#{}]", c), None => String::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_none_to_a() {
        let c = PriorityCycler::default_cycle();
        assert_eq!(c.cycle(None), Some('A'));
    }

    #[test]
    fn cycle_a_to_b() {
        let c = PriorityCycler::default_cycle();
        assert_eq!(c.cycle(Some('A')), Some('B'));
    }

    #[test]
    fn cycle_c_to_none() {
        let c = PriorityCycler::default_cycle();
        assert_eq!(c.cycle(Some('C')), None);
    }

    #[test]
    fn format() {
        assert_eq!(format_priority(Some('A')), "[#A]");
        assert_eq!(format_priority(None), "");
    }
}
