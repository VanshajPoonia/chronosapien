//! Retro terminal RPG quests derived from compiled ChronoOS capabilities.

const QUEST_USAGE: &str = "Usage: quest list|status";

pub fn run(command: &str) -> bool {
    let command = command.trim();

    if command != "quest" && !command.starts_with("quest ") {
        return false;
    }

    crate::println!("{}", QUEST_USAGE);
    true
}
