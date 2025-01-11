default:
    just --list

# set up precommit
pre-commit-install:
    pre-commit install

# update to latest versions in pre-commit
pre-commit-update:
    pre-commit autoupdate


