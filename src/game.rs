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
    pub order: usize,
    pub removable: bool,
}

pub struct Game {
    pub columns: usize,
    pub rows: usize,
    pub cells: Vec<usize>,
    pub fingers: Vec<Finger>,
    pub target: usize,
    pub placed: usize,
    pub release_after: usize,
    pub outcome: Outcome,
    pub deadline: f64,
    pub seconds: f64,
    rng: u64,
}

impl Game {
    pub fn new(columns: usize, rows: usize, seconds: f64, seed: u64, now: f64) -> Self {
        assert!((3..=12).contains(&columns) && (3..=12).contains(&rows));
        let mut game = Self {
            columns,
            rows,
            cells: (0..columns * rows).map(|i| i % 4).collect(),
            fingers: Vec::new(),
            target: 0,
            placed: 0,
            release_after: usize::MAX,
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
                self.cells.iter().enumerate().any(|(i, c)| {
                    c == color && !self.fingers.iter().any(|f| f.cell == i && !f.removable)
                })
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
        // Optional contacts remain occupied until actually lifted. They never count
        // as a new placement merely because a new round has started.
        self.fingers
            .retain(|f| !f.removable || contacts.iter().any(|(id, _)| *id == f.id));
        for finger in &mut self.fingers {
            if finger.removable {
                finger.cell = contacts
                    .iter()
                    .find(|(id, _)| *id == finger.id)
                    .and_then(|(_, cell)| *cell)
                    .unwrap_or(usize::MAX);
            }
        }
        for finger in self.fingers.iter().filter(|f| !f.removable) {
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
                    self.placed += 1;
                    self.fingers.push(Finger {
                        id: *id,
                        cell: c,
                        order: self.placed,
                        removable: false,
                    });
                    if self.placed >= self.release_after {
                        if let Some(oldest) = self.fingers.iter_mut().find(|f| !f.removable) {
                            oldest.removable = true;
                        }
                    }
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
    #[test]
    fn rolling_modes_release_in_order_and_continue() {
        for limit in [5, 10] {
            let mut g = Game::new(4, 4, 5., 42, 0.);
            g.release_after = limit;
            let mut contacts = Vec::new();
            for step in 1..=50 {
                let cell = target(&g);
                contacts.push((step as u64, Some(cell)));
                g.update(&contacts, step as f64);
                assert_eq!(g.outcome, Outcome::Playing);
                assert_eq!(g.placed, step);
                let removable = g.fingers.iter().find(|f| f.removable).copied();
                if step < limit {
                    assert!(removable.is_none());
                } else {
                    let finger = removable.unwrap();
                    assert_eq!(finger.order, step - limit + 1);
                    // Keeping it down for another frame is allowed and does not score.
                    g.update(&contacts, step as f64 + 0.1);
                    assert_eq!(g.placed, step);
                    contacts.retain(|(id, _)| *id != finger.id);
                    g.update(&contacts, step as f64 + 0.2);
                    assert_eq!(g.outcome, Outcome::Playing);
                    assert!(!g.occupied(finger.cell));
                    assert_eq!(g.fingers.len(), limit - 1);
                }
            }
        }
    }

    #[test]
    fn rolling_modes_do_not_allow_releasing_early_or_out_of_order() {
        for limit in [5, 10] {
            for count in [limit - 1, limit] {
                let mut g = Game::new(4, 4, 5., 42, 0.);
                g.release_after = limit;
                let mut contacts = Vec::new();
                for step in 1..=count {
                    let cell = target(&g);
                    contacts.push((step as u64, Some(cell)));
                    g.update(&contacts, step as f64);
                }
                let wrong_id = if count < limit { 1 } else { 2 };
                contacts.retain(|(id, _)| *id != wrong_id);
                g.update(&contacts, count as f64 + 0.1);
                assert!(matches!(g.outcome, Outcome::Lost(_)));
            }
        }
    }

    #[test]
    fn larger_rectangular_board() {
        let g = Game::new(12, 11, 5., 42, 0.);
        assert_eq!(g.cells.len(), 132);
        assert!(g.cells.iter().all(|&color| color < 4));
    }
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
