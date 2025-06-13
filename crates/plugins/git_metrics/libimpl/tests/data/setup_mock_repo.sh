# Create repo
mkdir mock-repo
cd mock-repo
git init

# Create user switcher function
set_user() {
  git config user.name "$1"
  git config user.email "$2"
}

# Set user Alice
set_user "Alice" "alice@example.com"
echo "Initial content by Alice" > README.md
git add README.md
GIT_AUTHOR_DATE="2023-01-01T10:00:00" GIT_COMMITTER_DATE="2023-01-01T10:00:00" git commit -m "Initial commit by Alice"

# More work by Alice
echo "Alice adds LICENSE" > LICENSE
echo "Alice adds CONTRIBUTING" > CONTRIBUTING.md
echo "Alice adds CODE_OF_CONDUCT" > CODE_OF_CONDUCT.md
git add LICENSE CONTRIBUTING.md CODE_OF_CONDUCT.md
GIT_AUTHOR_DATE="2023-01-01T11:00:00" GIT_COMMITTER_DATE="2023-01-01T11:00:00" git commit -m "Add LICENSE, CONTRIBUTING, CODE_OF_CONDUCT by Alice"

# Switch to Bob
set_user "Bob" "bob@example.com"
echo "Bob adds main.rs" > main.rs
echo "Temporary file" > temp.txt
git add main.rs temp.txt
GIT_AUTHOR_DATE="2023-01-02T09:30:00" GIT_COMMITTER_DATE="2023-01-02T09:30:00" git commit -m "Add main.rs and temp.txt by Bob"

# More work by Bob
echo "fn main() {}" >> main.rs
echo "Bob's note" > note.txt
git add main.rs note.txt
git rm temp.txt
GIT_AUTHOR_DATE="2023-01-02T10:00:00" GIT_COMMITTER_DATE="2023-01-02T10:00:00" git commit -m "Implement empty main(), add note.txt, remove temp.txt by Bob"

# Switch to Charlie
set_user "Charlie" "charlie@example.com"
echo "Some documentation" > docs.md
echo "Setup instructions" > SETUP.md
echo "Changelog" > CHANGELOG.md
git add docs.md SETUP.md CHANGELOG.md
GIT_AUTHOR_DATE="2023-01-03T08:00:00" GIT_COMMITTER_DATE="2023-01-03T08:00:00" git commit -m "Add docs.md, SETUP.md, CHANGELOG.md by Charlie"

# Branch: feature-login (Alice)
git checkout -b feature-login
set_user "Alice" "alice@example.com"
echo "Login module started" > login.rs
git add login.rs
GIT_AUTHOR_DATE="2023-01-04T12:00:00" GIT_COMMITTER_DATE="2023-01-04T12:00:00" git commit -m "Start login module by Alice"

echo "fn login() {}" >> login.rs
git add login.rs
GIT_AUTHOR_DATE="2023-01-04T14:00:00" GIT_COMMITTER_DATE="2023-01-04T14:00:00" git commit -m "Add login() function by Alice"

# Switch back to main and create another branch: feature-logout (Bob)
git checkout main
git checkout -b feature-logout
set_user "Bob" "bob@example.com"
echo "Logout module started" > logout.rs
git add logout.rs
GIT_AUTHOR_DATE="2023-01-05T09:00:00" GIT_COMMITTER_DATE="2023-01-05T09:00:00" git commit -m "Start logout module by Bob"

# Merge branches into main
git checkout main
git merge feature-login --no-ff -m "Merge feature-login into main"
git merge feature-logout --no-ff -m "Merge feature-logout into main"

# Final cleanup
set_user "Charlie" "charlie@example.com"
echo "Final review by Charlie" >> README.md
git add README.md
GIT_AUTHOR_DATE="2023-01-06T15:00:00" GIT_COMMITTER_DATE="2023-01-06T15:00:00" git commit -m "Review and update README by Charlie"

# More work by Alice - bigger commit
set_user "Alice" "alice@example.com"
echo -e "Line 1\nLine 2\nLine 3\nLine 4\nLine 5" > CONTRIBUTING.md
echo -e "Update 1\nUpdate 2\nUpdate 3" >> README.md
echo -e "LICENSE update 1\nLICENSE update 2" >> LICENSE
git add CONTRIBUTING.md README.md LICENSE
GIT_AUTHOR_DATE="2023-01-02T12:00:00" GIT_COMMITTER_DATE="2023-01-02T12:00:00" git commit -m "Add CONTRIBUTING.md and update README, LICENSE by Alice"

# Bob makes a big refactor in main.rs
set_user "Bob" "bob@example.com"
echo -e "fn main() {\n    println!(\"Hello, world!\");\n    // TODO: add more logic\n    let x = 42;\n    let y = x * 2;\n    println!(\"{}\", y);\n}" > main.rs
echo -e "Helper function\nfn helper() -> i32 {\n    7\n}" >> main.rs
git add main.rs
GIT_AUTHOR_DATE="2023-01-03T13:00:00" GIT_COMMITTER_DATE="2023-01-03T13:00:00" git commit -m "Refactor main.rs and add helper by Bob"

# Charlie updates docs.md with more content and removes a line from README.md
set_user "Charlie" "charlie@example.com"
echo -e "Some documentation\nSection 1\nSection 2\nSection 3\nSection 4" > docs.md
sed -i '1d' README.md
echo -e "Added new doc sections" >> docs.md
git add docs.md README.md
GIT_AUTHOR_DATE="2023-01-04T09:00:00" GIT_COMMITTER_DATE="2023-01-04T09:00:00" git commit -m "Expand docs.md and remove line from README by Charlie"

# Alice makes a big commit in feature-login
git checkout feature-login
set_user "Alice" "alice@example.com"
echo -e "fn validate_user() {}\nfn reset_password() {}\nfn logout() {}\n// More login logic" >> login.rs
echo -e "Login README\nHow to use login module\nAPI docs\nExamples" > login_README.md
git add login.rs login_README.md
GIT_AUTHOR_DATE="2023-01-04T16:00:00" GIT_COMMITTER_DATE="2023-01-04T16:00:00" git commit -m "Add multiple functions to login.rs and new login_README.md by Alice"

# Bob makes a big commit in feature-logout
git checkout feature-logout
set_user "Bob" "bob@example.com"
echo -e "fn logout_user() {}\nfn session_end() {}\n// More logout logic" >> logout.rs
echo -e "Logout instructions\nAPI\nExamples" > logout_README.md
git add logout.rs logout_README.md
GIT_AUTHOR_DATE="2023-01-05T11:00:00" GIT_COMMITTER_DATE="2023-01-05T11:00:00" git commit -m "Add functions to logout.rs and new logout_README.md by Bob"

# Merge branches into main (as before)
git checkout main
git merge feature-login --no-ff -m "Merge feature-login into main"
git merge feature-logout --no-ff -m "Merge feature-logout into main"

# Charlie does a big review commit
set_user "Charlie" "charlie@example.com"
echo -e "Final review by Charlie\nExtra notes\nSummary\nChangelog\nKnown issues" >> README.md
echo -e "Reviewed by Charlie" >> CONTRIBUTING.md
echo -e "Reviewed by Charlie" >> docs.md
git add README.md CONTRIBUTING.md docs.md
GIT_AUTHOR_DATE="2023-01-06T17:00:00" GIT_COMMITTER_DATE="2023-01-06T17:00:00" git commit -m "Big review: update README, CONTRIBUTING, docs by Charlie"
