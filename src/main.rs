use std::{env, net::TcpStream, path::PathBuf};

use chrono::Utc;
use clap::{value_parser, Parser};
use clap_stdin::MaybeStdin;
use git2::{
    BranchType, Cred, IndexAddOption, PushOptions, RemoteCallbacks, Repository, StatusOptions,
};

#[derive(Parser)]
#[command(version="0.1", about="Automatically commits all changes in a repo after using connecting to ssh-key", long_about=None)]
struct Cli {
    #[arg(short='d', long="directory", value_parser=value_parser!(PathBuf))]
    repo_path: PathBuf,

    #[arg(short='k', long="keyfile", value_parser=value_parser!(PathBuf), default_value=format!("{}/.ssh/id_ed25519", home::home_dir().unwrap().display()))]
    key_file: PathBuf,

    #[arg(short = 'p', long = "password", env = "SSHPASS")]
    password: Option<String>,

    #[arg(short = 'm', long = "message")]
    commit_message: Option<MaybeStdin<String>>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if TcpStream::connect(("google.com", 80)).is_err() {
        println!("Not connected to internet!");
        return Ok(());
    }

    println!("Connected to internet!");

    let repo = Repository::open(cli.repo_path.clone()).unwrap_or_else(|_| {
        panic!(
            "No repo in {}",
            cli.repo_path
                .into_os_string()
                .into_string()
                .expect("Valid path string")
        )
    });

    let mut status_opts = StatusOptions::new();
    let statuses = repo
        .statuses(Some(&mut status_opts))
        .expect("Could not get status of repo");

    // If there are no changes, skip commit and push
    if !statuses.is_empty() {
        commit(&repo);
    } else {
        println!("No changes detected, skipping commit and proceeding to push.");
    };

    let head = repo.head()?.resolve()?; // Get head
    let local_oid = head.peel_to_commit()?.id(); // Local Object ID
    let branch_name = head
        .shorthand()
        .ok_or_else(|| git2::Error::from_str("Not on a branch!"))?; // Return error if detached
    let local_branch = repo.find_branch(branch_name, BranchType::Local)?; // Get local branch to
                                                                          // get upstream version
    let remote_oid = local_branch.upstream()?.get().peel_to_commit()?.id(); // Remote Object ID

    // If local is ahead of remote
    if repo.graph_descendant_of(local_oid, remote_oid)? {
        push(&repo);
    } else {
        println!("No difference between local and remote branch, not pushing...");
    }

    println!("Done");
    Ok(())
}

fn commit(repo: &Repository) {
    let cli = Cli::parse();

    let mut index = repo.index().expect("No index");
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .expect("Could not add stage all files");
    index.write().expect("Could not write staged files");

    let tree_oid = index.write_tree().expect("Could not get write tree");
    let tree = repo
        .find_tree(tree_oid)
        .expect("Could not find tree with given write tree");

    let head = repo
        .head()
        .expect("Could not get HEAD")
        .target()
        .expect("HEAD has no target");
    let parent_commit = repo.find_commit(head).expect("Could find HEAD commit");

    let sig = repo.signature().expect("Repo has no associated signature");

    let commit_message = cli
        .commit_message
        .map(|val| val.into_inner())
        .unwrap_or_else(|| {
            let var_name = format!(
                "From {}: {}",
                env::consts::OS,
                Utc::now().format("%a, %b %d %Y %T")
            );
            var_name
        });

    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &commit_message,
        &tree,
        &[&parent_commit],
    )
    .expect("Could not generate commit");
}

fn push(repo: &Repository) {
    let cli = Cli::parse();

    let mut remote = repo
        .find_remote("origin")
        .expect("Could not find remote origin");

    let mut push_options = PushOptions::new();

    push_options.remote_callbacks({
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username, _allowed_types| {
            println!("Username: {}", username.unwrap());
            println!("Password: {:?}", cli.password.as_deref());
            Cred::ssh_key(
                username.unwrap(),
                None,
                &cli.key_file,
                cli.password.as_deref(),
            )
        });
        callbacks
    });

    remote
        .push(
            &["refs/heads/main:refs/heads/main"],
            Some(&mut push_options),
        )
        .expect("Could not push to remote");
}
