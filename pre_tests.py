import os
import subprocess
from dotenv import load_dotenv

load_dotenv()

DATABASE_URL = os.getenv("DATABASE_URL")
ADMIN_DB_URL = DATABASE_URL.replace('/tmt', '/postgres')


def drop_db():
    args = ['psql', ADMIN_DB_URL, '-c', 'DROP DATABASE tmt;']
    subprocess.run(args, check=True)


def create_db():
    args = ['psql', ADMIN_DB_URL, '-c', 'CREATE DATABASE tmt']
    subprocess.run(args, check=True)


def run_migrations():
    args = ['diesel', 'migration', 'run']
    subprocess.run(args, check=True)


def create_user():
    args = ['psql', DATABASE_URL, '-c',
            "INSERT INTO users (email, password, confirmed) VALUES ('sam.raker+1@gmail.com', 'password1', true)"]
    subprocess.run(args, check=True)


def main():
    drop_db()
    create_db()
    run_migrations()
    create_user()
