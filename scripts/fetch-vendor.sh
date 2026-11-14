#!/usr/bin/env bash
# Зеркала C-оригиналов (ТЗ 3.8, 5.1): shallow-клоны всех кодовых репозиториев организации.
# ≈300–400 МБ (полные .git всех 46 реп — 1.36 ГБ, из них 599 МБ — сайт). Живут в .gitignore.
set -euo pipefail
cd "$(dirname "$0")/.."
ORG=${ORG:-mate-desktop}
DEST=vendor/mate
SKIP="mate-desktop.org mate-wiki .github"
mkdir -p "$DEST"
curl -fsS "https://api.github.com/orgs/$ORG/repos?per_page=100&type=public" \
  | python3 -c 'import json,sys; [print(r["name"]) for r in json.load(sys.stdin)]' | sort > /tmp/svetlitsa-repos.txt
echo "репозиториев в организации: $(wc -l < /tmp/svetlitsa-repos.txt)"
while read -r repo; do
  case " $SKIP " in *" $repo "*) echo "пропуск $repo (не код)"; continue;; esac
  if [ -d "$DEST/$repo/.git" ]; then
    git -C "$DEST/$repo" fetch --depth 1 -q origin && git -C "$DEST/$repo" reset -q --hard "@{u}"
  else
    git clone --depth 1 -q "https://github.com/$ORG/$repo" "$DEST/$repo"
  fi
  echo "готово: $repo"
done < /tmp/svetlitsa-repos.txt
du -sh "$DEST"
