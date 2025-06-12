version="1.0.1"

awk -v ver="$version" '
  $0 ~ "^## \\["ver"\\]" { found = 1; print; next }
  found && /^---$/ { exit }
  found { print }
' CHANGELOG.md
