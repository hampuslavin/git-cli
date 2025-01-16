use inquire::{InquireError, Select};
use std::{
    env::{self},
    fs,
    io::{BufRead, BufReader},
    process::Command,
};

fn get_commits() -> Vec<String> {
    let output = Command::new("git")
        .arg("log")
        .arg("--oneline")
        .arg("main..HEAD")
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        panic!();
    }

    let reader = BufReader::new(&output.stdout[..]);

    return reader
        .lines()
        .map(|line| line.unwrap())
        .filter(|line| !line.ends_with("_amend"))
        .collect();
}

fn amend_to_selected_commit(
    hash: &str,
    todo_file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(todo_file_path)?;

    let lines: Vec<&str> = content.lines().collect();
    let target_index = lines.iter().position(|&line| line.contains(hash)).unwrap();
    let amend_index = lines
        .iter()
        .position(|&line| line.contains("_amend"))
        .unwrap();

    let mut new_lines = lines.clone();
    let amend_line = new_lines.remove(amend_index).replace("pick", "fixup");
    new_lines.insert(target_index + 1, amend_line.as_str());

    let new_content = new_lines.join("\n");
    fs::write(todo_file_path, new_content)?;

    println!("Successfully modified the rebase todo file");

    Ok(())
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 3 {
        // Recursive call from git rebase
        let commit_hash = &args[1];
        let todo_file_path = &args[2];

        amend_to_selected_commit(&commit_hash, &todo_file_path).unwrap();
        return;
    }

    let staged_files = Command::new("git")
        .args(&["diff", "--name-only", "--cached"])
        .output()
        .unwrap();
    if staged_files.stdout.is_empty() {
        println!("No staged files to amend");
        return;
    }

    let commits = get_commits();

    let ans: Result<String, InquireError> =
        Select::new("Select commit to amend to", commits).prompt();

    match ans {
        Ok(commit_to_ammend_to) => {
            call_command_recursively(commit_to_ammend_to.split_once(' ').unwrap().0.to_string())
                .unwrap()
        }
        Err(e) => println!("There was an error, please try again: {}", e),
    }
}

fn call_command_recursively(commit_to_ammend_to: String) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("git")
        .args(&["commit", "-m", "_amend"])
        .output()?;

    //git diff --name-only
    let unstaged_files = Command::new("git")
        .args(&["diff", "--name-only"])
        .output()?;
    let should_stash = !unstaged_files.stdout.is_empty();
    if should_stash {
        Command::new("git").args(&["stash"]).output()?;
    }

    // Step 3: Start an interactive rebase
    let rebase_output = Command::new("git")
        .env(
            "GIT_SEQUENCE_EDITOR",
            &format!("gitcli {}", commit_to_ammend_to),
        )
        .args(&["rebase", "-i", &format!("{}~1", commit_to_ammend_to)])
        .output()?;

    if !rebase_output.status.success() {
        return Err(format!(
            "Rebase failed: {}",
            String::from_utf8(rebase_output.stderr)?
        )
        .into());
    }

    if should_stash {
        Command::new("git").args(&["stash", "pop"]).output()?;
    }

    Ok(())
}
