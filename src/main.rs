use inquire::{InquireError, Select};
use std::{
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

    return reader.lines().map(|line| line.unwrap()).collect();
}

fn amend_to_selected_commit(hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create a temporary commit with the changes to be amended
    Command::new("git")
        .args(&["commit", "-m", "__amend__"])
        .output()?;

    // Step 2: Get the current branch name
    let branch_output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;
    let current_branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

    // Step 3: Start an interactive rebase
    let rebase_output = Command::new("git")
        .args(&["rebase", "-i", &format!("{}~1", hash)])
        .output()?;

    if !rebase_output.status.success() {
        return Err(format!(
            "Rebase failed: {}",
            String::from_utf8(rebase_output.stderr)?
        )
        .into());
    }

    // Step 4: Modify the rebase todo file
    let todo_path = ".git/rebase-merge/git-rebase-todo";
    let mut content = fs::read_to_string(todo_path)?;

    // Find the line with the target commit and the line with "__amend__"
    let lines: Vec<&str> = content.lines().collect();
    let target_index = lines.iter().position(|&line| line.contains(hash)).unwrap();
    let amend_index = lines
        .iter()
        .position(|&line| line.contains("__amend__"))
        .unwrap();

    // Move the "__amend__" line just after the target commit and change it to "fixup"
    let mut new_lines = lines.clone();
    let amend_line = new_lines.remove(amend_index);
    new_lines.insert(target_index + 1, &amend_line.replace("pick", "fixup"));

    // Write the modified content back to the todo file
    let new_content = new_lines.join("\n");
    fs::write(todo_path, new_content)?;

    // Step 5: Continue the rebase
    Command::new("git")
        .args(&["rebase", "--continue"])
        .output()?;

    println!("Successfully amended changes to commit {}", hash);

    Ok(())
}
fn main() {
    let commits = get_commits();

    let ans: Result<String, InquireError> =
        Select::new("What's your favorite fruit?", commits).prompt();

    match ans {
        Ok(choice) => println!("{}! That's mine too!", choice),
        Err(_) => println!("There was an error, please try again"),
    }

    // Create a browsable list of commits.
    // Should only be the commmits that are new after main, i.e. the same tha will show up for git rebase --interactive main

    // Get the list of commits that are new after main
    // git log --oneline main..HEAD
}
