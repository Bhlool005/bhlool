use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read};
use std::collections::HashMap;

const SCORE_FILE: &str = "scores.txt";

pub struct Leaderboard {
    scores: HashMap<String, u32>,
}

impl Leaderboard {
    pub fn new() -> Self {
        let scores = Leaderboard::load_scores();
        Leaderboard { scores }
    }

    pub fn add_score(&mut self, player: &str, score: u32) {
        let entry = self.scores.entry(player.to_string()).or_insert(0);
        if score > *entry {
            *entry = score;
        }
        Leaderboard::save_scores(&self.scores);
    }

    pub fn display(&self) {
        let mut leaderboard: Vec<(&String, &u32)> = self.scores.iter().collect();
        leaderboard.sort_by(|a, b| b.1.cmp(a.1)); // Sort scores in descending order

        println!("Leaderboard:");
        for (player, score) in leaderboard {
            println!("{}: {}", player, score);
        }
    }

    fn load_scores() -> HashMap<String, u32> {
        let mut scores = HashMap::new();
        let file = File::open(SCORE_FILE);
        if let Ok(mut file) = file {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            for line in content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() == 2 {
                    let player = parts[0].to_string();
                    let score = parts[1].trim().parse::<u32>().unwrap();
                    scores.insert(player, score);
                }
            }
        }
        scores
    }

    fn save_scores(scores: &HashMap<String, u32>) {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(SCORE_FILE).unwrap();
        for (player, score) in scores.iter() {
            writeln!(file, "{}: {}", player, score).unwrap();
        }
    }
}

// Example Usage:
// fn main() {
//     let mut leaderboard = Leaderboard::new();
//     leaderboard.add_score("Player1", 100);
//     leaderboard.add_score("Player2", 150);
//     leaderboard.display();
// }