use std::collections::HashMap;

use openai_api_rust::*;
use openai_api_rust::chat::*;
use openai_api_rust::completions::*;
use anyhow::Result;
use dotenvy::dotenv;
// TODO make llm a trait and crate feature
use palm_api::palm::create_client;
use palm_api::palm::new_text_body;


// Todo investigate other auth mechanisms
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
            content
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

pub fn list_commits() -> Result<HashMap<git2::Oid, String>> {
    // Run git log --oneline to get all commits
    // Put all the strings into a HashMap of Commit ID, String
    // Return the HashMap
    let mut commits = HashMap::new();
    // TODO support specifying the repo path
    let repo = git2::Repository::open(".")?;
    repo.references()?
        .for_each(|reference| {
            let reference: git2::Reference = reference.unwrap();
            println!("Reference: {:?}", reference.name());
            match reference.target() {
                Some(target) => {
                    let commit = repo.find_commit(target).unwrap();
                    let message = commit.message().unwrap();
                    commits.insert(commit.id(), message.to_string());
                }
                None => {
                    let resolved_ref = reference.resolve().unwrap();
                    println!("Resolved ref: {:?}", resolved_ref.target());
                    let commit = repo.find_commit(resolved_ref.target().unwrap()).unwrap();
                    let message = commit.message().unwrap();
                    commits.insert(commit.id(), message.to_string());
                }
            }
        });
    let mut walker = repo.revwalk()?;
    walker.push_head()?;

    Ok(commits)
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
    fn test_commits() {
        let commits = list_commits().unwrap();
        for (commit_id, message) in commits {
            println!("{}: {}", commit_id, message);
        }
    }
}
