#! /usr/bin/env bash

TEST_DB_URL=$(grep DATABASE_URL_TEST .env | awk -F '=' '{ print $2 }')
OTHER_DB_URL=$(grep '^DATABASE_URL=' .env | awk -F '=' '{ print $2 }')

function drop_test_db() {
	psql "$OTHER_DB_URL" -c "DROP DATABASE tmt_test"
}

function create_test_db() {
	psql "$OTHER_DB_URL" -c "CREATE DATABASE tmt_test"
}

function run_migrations() {
	diesel migration run --database-url "$TEST_DB_URL"
}

function main() {
	drop_test_db && create_test_db && run_migrations
}

main
