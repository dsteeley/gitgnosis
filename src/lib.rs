use std::collections::HashMap;

use anyhow::Result;
use dotenvy::dotenv;
use openai_api_rust::chat::*;
// use openai_api_rust::completions::*;
use openai_api_rust::*;
// TODO make llm a trait and crate feature
use palm_api::palm::create_client;
use palm_api::palm::new_text_body;

const PROMPT_PREFIX: &str = r#"I'm going to provide a list of recent commits to a codebase and three example changelog sections to learn from.
Please provide me with a Changelog unreleased section with a human readable summary of the changes from the commit messages.
The following is a list of commits in this repository:\n"#;

const PROMPT_EXAMPLE_CHANGELOG: &str = r#"
Here is the first example of an unreleased changelog section.
## [Unreleased]
### Breaking Changes
- Removed the ability to query the database.

### Added
- Added support for the querying of commits in the repository.
- Added support for querying the Palm API for text generation.

### Fixed
- Resolved bug in where the program would crash when the user pressed the 'X' button.


Here is the second unreleased changelog section.

## [Unreleased]
### Breaking Changes

### Added
- Added support for bumping the project version automatically based on the changelog
- Handled the scenario where users attempted to downgrade the project version.

### Fixed
- Handled the case where the user would input an empty string.

Here is a third example of an unreleased changelog section.

## [Unreleased]
### Breaking Changes

### Added

### Fixed
- Fixup failing CI pipeline.


"#;

// const PROMPT_SUFFIX: &str = "Please summarise each commit in a single sentence in the format of a CHANGELOG unreleased section categorising each commit into breaking change, added or fixed.";

// Todo investigate other auth mechanisms for apis
pub fn auth() -> Auth {
    // Dot in a .env file
    // dotenv().expect(".env file not found");
    Auth::from_env().unwrap()
}

// TODO better return type
// TODO split into one object and have trait per model
pub fn query_openapi(content: String) -> Result<String> {
    let auth = auth();
    // TODO add support for other llms
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    // TODO other models
    let body = ChatBody {
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(20),
        temperature: Some(0_f32),
        top_p: Some(0_f32),
        n: Some(2),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![Message {
            role: Role::User,
            content,
        }],
    };
    let rs = openai.chat_completion_create(&body);
    let choice = rs.unwrap().choices;
    // TODO Error handling
    let message = &choice[0].message.as_ref().unwrap().content;
    Ok(message.to_string())
}

pub fn query_palm(content: String) -> Result<String> {
    dotenv().expect(".env file not found");
    let palm_api_key = std::env::var("PALM_API_KEY").unwrap();
    let client = create_client(palm_api_key.to_string());
    let mut textbody = new_text_body();
    textbody.set_text_prompt(content);
    let response = client
        .generate_text("text-bison-001".to_string(), textbody)
        .expect("An error has occured.");
    Ok(response.candidates.unwrap()[0].output.clone())
}

fn list_commits() -> Result<HashMap<String, String>> {
    // Run git log --oneline to get all commits
    // Put all the strings into a HashMap of Commit ID, String
    // Return the HashMap
    let mut commits = HashMap::new();
    // Run the git log command
    let cmd = std::process::Command::new("git")
        .arg("log")
        .arg("--oneline")
        .output()
        .expect("Failed to execute command");
    cmd.stdout.split(|&x| x == b'\n').for_each(|line| {
        let line = String::from_utf8(line.to_vec()).unwrap();
        if line.len() < 9 {
            // Return early as the line is too short
            return;
        }
        let commit_id = line[0..7].to_string();
        let message = line[8..].to_string();
        commits.insert(commit_id, message);
    });
    Ok(commits)
}

fn commits_to_prompt_string(commits: HashMap<String, String>) -> String {
    let mut prompt = String::new();
    for (commit_id, message) in commits {
        prompt.push_str(&format!("- {}:{}\n", commit_id, message));
    }
    prompt
}

pub fn generate_changelog_section() -> Result<String> {
    let commits = list_commits()?;
    let commit_prompt = commits_to_prompt_string(commits);
    let prompt = PROMPT_PREFIX.to_string() + &commit_prompt + PROMPT_EXAMPLE_CHANGELOG;
    let result = query_palm(prompt)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_world() {
        let content = "Hello".to_string();
        // Run the test on Palm as that's currently free to use for some developers.
        let result = query_palm(content).unwrap();
        println!("{}", result);
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_generate_changelog_section() {
        let result = generate_changelog_section().unwrap();
        println!("{}", result);
    }
}
