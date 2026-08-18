//! Answering the routine prompts for you — and refusing to answer the rest.
//!
//! Turbo is off until you turn it on, and even then it is deliberately timid.
//! The value of an attention queue is that the things in it are real; an
//! auto-answer that guesses wrong destroys that in one keystroke you never
//! saw. So two refusals are absolute:
//!
//! - a **guessed** prompt is never answered. We recognised the shape of a
//!   question on a screen; we do not know what the answers mean.
//! - a **destructive** choice is never taken, and an ambiguous prompt (no
//!   safe choice, or more than one) is left for a person.
//!
//! Protects `docs/guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse.md`.

use crate::session::{Choice, Prompt, PromptSource};

/// The choice turbo would take, if it may take one at all.
pub fn turbo_choice(prompt: &Prompt) -> Option<&Choice> {
    if prompt.source != PromptSource::Declared {
        return None;
    }
    let mut safe = prompt.choices.iter().filter(|c| !c.destructive);
    let only = safe.next()?;
    // A second safe answer means the program is asking something with more
    // than one harmless outcome — which is a decision, not a formality.
    safe.next().is_none().then_some(only)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(label: &str, destructive: bool) -> Choice {
        Choice {
            label: label.to_string(),
            send: "y\n".to_string(),
            destructive,
        }
    }

    fn declared(choices: Vec<Choice>) -> Prompt {
        Prompt {
            question: "Read package.json?".to_string(),
            choices,
            source: PromptSource::Declared,
        }
    }

    #[test]
    fn one_safe_choice_on_a_declared_prompt_is_answered() {
        let prompt = declared(vec![choice("Allow", false), choice("Delete it", true)]);
        assert_eq!(turbo_choice(&prompt).unwrap().label, "Allow");
    }

    #[test]
    fn a_guessed_prompt_is_never_answered() {
        let prompt = Prompt {
            source: PromptSource::Guessed,
            ..declared(vec![choice("Allow", false)])
        };
        assert!(turbo_choice(&prompt).is_none());
    }

    #[test]
    fn an_all_destructive_prompt_is_left_alone() {
        let prompt = declared(vec![choice("Force push", true), choice("Reset hard", true)]);
        assert!(turbo_choice(&prompt).is_none());
    }

    #[test]
    fn a_choice_between_two_harmless_outcomes_is_still_yours_to_make() {
        let prompt = declared(vec![choice("Python", false), choice("Rust", false)]);
        assert!(turbo_choice(&prompt).is_none());
    }

    #[test]
    fn a_prompt_with_no_choices_is_not_answerable() {
        assert!(turbo_choice(&declared(vec![])).is_none());
    }
}
