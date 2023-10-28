mod spaced_repetition;

use std::{cell::RefCell, io::Write, rc::Rc};

use crate::discovered_word::DiscoveredWord;
use rusqlite::Connection;

use self::spaced_repetition::SpacedRepetition;

pub fn review(connection: &Connection) -> color_eyre::Result<()> {
    let words = DiscoveredWord::list(connection)?
        .into_iter()
        .map(|w| Rc::new(RefCell::new(w)))
        .collect::<Vec<_>>();
    loop {
        let mut answered_a_question_wrong = false;
        for word in SpacedRepetition::new(words.clone()) {
            let jmdict_entry = jmdict::entries()
                .find(|entry| {
                    entry
                        .kanji_elements()
                        .any(|e| e.text == word.borrow().word())
                })
                .ok_or_else(|| {
                    color_eyre::eyre::eyre!("Discovered Word not found in the dictionary")
                })?;
            let correct_answers = jmdict_entry
                .senses()
                .map(|s| s.glosses().map(|g| g.text.to_string()).collect::<Vec<_>>())
                .collect::<Vec<_>>();
            println!("{}Word: {}", termion::clear::All, word.borrow().word());
            println!("Type the meaning of this word:");
            print!("Answer: ");
            std::io::stdout().lock().flush()?;
            let mut answer = String::new();

            std::io::stdin().read_line(&mut answer)?;
            if correct_answers
                .iter()
                .any(|ca| ca.iter().any(|ca| ca == answer.trim()))
            {
                println!("{}✓ Nailed it{}", termion::color::Fg(termion::color::Green), termion::style::Reset);
                if correct_answers.len() > 1 {
                    println!("Other possible correct answers:");
                    for gloss in correct_answers {
                        print!("- {}", gloss[0]);
                        for correct_answer in &gloss[1..] {
                            print!(", {correct_answer}")
                        }
                        println!()
                    }
                }
                word.borrow_mut().reviewed(true, connection)?;
            } else {
                println!("{}✘ Wrong answer{}", termion::color::Fg(termion::color::Red), termion::style::Reset);
                println!("Correct answers:");
                for gloss in correct_answers {
                    print!("- {}", gloss[0]);
                    for correct_answer in &gloss[1..] {
                        print!(", {correct_answer}")
                    }
                    println!()
                }
                word.borrow_mut().reviewed(false, connection)?;
                answered_a_question_wrong = true;
            }
            println!("Press enter to go to the next word");
            let mut a = String::new();
            std::io::stdin().read_line(&mut a)?;
        }
        if !answered_a_question_wrong {
            break;
        }
    }
    println!("You finished reviewing all the ✨{}{}discovered words{}✨, go take a break, drink a coffee, do what you do best.", termion::style::Bold, termion::color::Fg(termion::color::Yellow), termion::style::Reset);
    Ok(())
}
