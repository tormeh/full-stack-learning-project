# full-stack-learning-project

Requires something a la this in postgres:
create database mydb;
create extension "uuid-ossp";
create table test (id uuid default uuid_generate_v4(), content text);


Use this to create new entries:
curl http://yoururl/item/ -X POST -H 'Content-Type: application/json' -d '{"content":"stuff I like to write into a database here"}'