# Auto Commit

## Description

A small CLI tool that commits and pushes any changes within a repository as long as the changes are more recent that the remote. Its purpose is to make the git add, git commit, git push into one command for convenience. Also adds the ability to load ssh key through the command without having to start the ssh-agent.

## Arguments

`--directory` (`-d`): The path of the repository

`--keyfile` (`-k`): The path to the SSH key file (by default: ~/.ssh/id_ed25519)

`--password` (`-p`): Password to access the SSH Key file. Alternatively, if not passed, reads the environment variable SSHPASS.

`--message` (`-m`): Passed to `$ git commit`
