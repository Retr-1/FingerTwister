#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Playing,
    Lost(&'static str),
    Won,
}

#[derive(Clone, Copy, Debug)]
pub struct Finger {
    pub id: u64,
    pub cell: usize,
}

pub struct Game {
    pub columns: usize,
    pub rows: usize,
    pub cells: Vec<usize>,
    pub fingers: Vec<Finger>,
    pub target: usize,
    pub outcome: Outcome,
    pub deadline: f64,
    pub seconds: f64,
    rng: u64,
}

impl Game {
    pub fn new(columns: usize, rows: usize, seconds: f64, seed: u64, now: f64) -> Self {
        assert!((3..=6).contains(&columns) && (3..=6).contains(&rows));
        let mut game = Self {
            columns,
            rows,
            cells: (0..columns * rows).map(|i| i % 4).collect(),
            fingers: Vec::new(),
            target: 0,
            outcome: Outcome::Playing,
            deadline: now + seconds,
            seconds,
            rng: seed.max(1),
        };
        for i in (1..game.cells.len()).rev() {
            let j = game.random(i + 1);
            game.cells.swap(i, j);
        }
        game.choose_target();
        game
    }

    fn random(&mut self, upper: usize) -> usize {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng % upper as u64) as usize
    }

    pub fn occupied(&self, cell: usize) -> bool {
        self.fingers.iter().any(|f| f.cell == cell)
    }

    fn choose_target(&mut self) {
        let available: Vec<_> = (0..4)
            .filter(|color| {
                self.cells
                    .iter()
                    .enumerate()
                    .any(|(i, c)| c == color && !self.occupied(i))
            })
            .collect();
        if available.is_empty() {
            self.outcome = Outcome::Won;
        } else {
            self.target = available[self.random(available.len())];
        }
    }

    // A complete snapshot, including contacts outside the board (cell = None).
    // Validate every held finger before accepting any new one.
    pub fn update(&mut self, contacts: &[(u64, Option<usize>)], now: f64) {
        if self.outcome != Outcome::Playing {
            return;
        }
        for finger in &self.fingers {
            if !contacts
                .iter()
                .any(|(id, cell)| *id == finger.id && *cell == Some(finger.cell))
            {
                self.outcome = Outcome::Lost("Prst sa zdvihol alebo opustil policko.");
                return;
            }
        }
        if now >= self.deadline {
            self.outcome = Outcome::Lost("Cas vyprsal.");
            return;
        }
        // One command accepts one new finger. Simultaneous extra fingers are invalid.
        let new: Vec<_> = contacts
            .iter()
            .filter(|(id, _)| !self.fingers.iter().any(|f| f.id == *id))
            .collect();
        if new.len() > 1 {
            self.outcome = Outcome::Lost("Pridavaj vzdy iba jeden prst.");
            return;
        }
        if let Some((id, cell)) = new.first() {
            match *cell {
                Some(c)
                    if c < self.cells.len()
                        && !self.occupied(c)
                        && self.cells[c] == self.target =>
                {
                    self.fingers.push(Finger { id: *id, cell: c });
                    self.choose_target();
                    self.deadline = now + self.seconds;
                }
                _ => self.outcome = Outcome::Lost("Treba volne policko zadanej farby."),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn game() -> Game {
        Game::new(3, 3, 5., 42, 0.)
    }
    fn target(g: &Game) -> usize {
        (0..g.cells.len())
            .find(|&i| g.cells[i] == g.target && !g.occupied(i))
            .unwrap()
    }
    #[test]
    fn accepts_new_finger_and_keeps_previous_contacts() {
        let mut g = game();
        let a = target(&g);
        g.update(&[(10, Some(a))], 1.);
        let b = target(&g);
        g.update(&[(10, Some(a)), (20, Some(b))], 2.);
        assert_eq!(g.fingers.len(), 2);
        assert_eq!(g.deadline, 7.);
        assert_eq!(g.outcome, Outcome::Playing);
    }
    #[test]
    fn release_slide_and_replacement_lose() {
        for kind in 0..3 {
            let mut g = game();
            let a = target(&g);
            g.update(&[(1, Some(a))], 1.);
            let contacts = match kind {
                0 => vec![],
                1 => vec![(1, Some((a + 1) % 9))],
                _ => vec![(2, Some(a))],
            };
            g.update(&contacts, 2.);
            assert!(matches!(g.outcome, Outcome::Lost(_)));
        }
    }
    #[test]
    fn wrong_outside_occupied_and_multiple_contacts_lose() {
        for kind in 0..4 {
            let mut g = game();
            let a = target(&g);
            let wrong = (0..9).find(|&i| g.cells[i] != g.target).unwrap();
            let contacts = match kind {
                0 => vec![(1, Some(wrong))],
                1 => vec![(1, None)],
                2 => {
                    g.update(&[(1, Some(a))], 1.);
                    vec![(1, Some(a)), (2, Some(a))]
                }
                _ => vec![(1, Some(a)), (2, Some(a))],
            };
            g.update(&contacts, 2.);
            assert!(matches!(g.outcome, Outcome::Lost(_)));
        }
    }
    #[test]
    fn deadline_and_terminal_state() {
        let mut g = game();
        let a = target(&g);
        g.update(&[(1, Some(a))], 5.);
        assert_eq!(g.outcome, Outcome::Lost("Cas vyprsal."));
        g.update(&[(1, Some(a))], 6.);
        assert!(g.fingers.is_empty());
    }
    #[test]
    fn every_target_has_a_free_cell_until_full() {
        for columns in 3..=6 {
            for rows in 3..=6 {
                for seed in 1..100 {
                    let mut g = Game::new(columns, rows, 5., seed, 0.);
                    let mut contacts = Vec::new();
                    for step in 0..columns * rows {
                        let cell = target(&g);
                        contacts.push((step as u64, Some(cell)));
                        g.update(&contacts, step as f64 + 1.);
                    }
                    assert_eq!(g.outcome, Outcome::Won);
                    assert_eq!(g.cells.len(), columns * rows);
                }
            }
        }
    }
}
