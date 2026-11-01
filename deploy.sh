#!/bin/bash
set -e

if [ -f .env ]; then
  export $(grep -v '^#' .env | xargs)
else
  echo "❌ Erro: Ficheiro .env não encontrado!"
  exit 1
fi

cargo build --release

rsync -avz --progress --delete --exclude-from='.rsyncignore' \
  .env assets templates static ${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/

rsync -avz --progress target/release/site ${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}/target/release/

ssh -t ${DEPLOY_USER}@${DEPLOY_HOST} "sudo systemctl restart portfolio"
