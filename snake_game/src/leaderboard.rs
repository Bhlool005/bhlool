use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};

const SCORE_FILE: &str = "scores.txt";

pub struct Leaderboard {
    scores: HashMap<String, u32>,
}

impl Leaderboard {
    pub fn new() -> Self {
        Self {
            scores: Self::load_scores().unwrap_or_default(),
        }
    }

    pub fn add_score(&mut self, player: &str, score: u32) -> io::Result<()> {
        let entry = self.scores.entry(player.to_string()).or_insert(0);
        if score > *entry {
            *entry = score;
        }
        Self::save_scores(&self.scores)
    }

    pub fn sorted_scores(&self) -> Vec<(&str, u32)> {
        let mut leaderboard: Vec<(&str, u32)> = self
            .scores
            .iter()
            .map(|(player, score)| (player.as_str(), *score))
            .collect();
        leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
        leaderboard
    }

    fn load_scores() -> io::Result<HashMap<String, u32>> {
        let mut scores = HashMap::new();
        let Ok(mut file) = File::open(SCORE_FILE) else {
            return Ok(scores);
        };

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        for line in content.lines() {
            let mut parts = line.splitn(2, ':');
            let Some(player) = parts.next() else { continue };
            let Some(score_raw) = parts.next() else {
                continue;
            };
            if let Ok(score) = score_raw.trim().parse::<u32>() {
                scores.insert(player.to_string(), score);
            }
        }
        Ok(scores)
    }

    fn save_scores(scores: &HashMap<String, u32>) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(SCORE_FILE)?;

        for (player, score) in scores {
            writeln!(file, "{}: {}", player, score)?;
        }

        Ok(())
    }
}
